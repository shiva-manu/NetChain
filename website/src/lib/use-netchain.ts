import { useEffect, useState } from "react";
import {
  DEFAULT_MONITORING_URL,
  DEFAULT_RPC_URL,
  NetChainClient,
} from "@/lib/netchain-client";
import type {
  AccountInfo,
  BlockDetails,
  BlockSummary,
  ChainInfo,
  ChainParams,
  HealthSnapshot,
  MetricsSnapshot,
  NetChainEndpoints,
  ProposalInfo,
  StakingInfo,
  TransactionFeedItem,
  ValidatorSlashedEvent,
  WsEvent,
} from "@/lib/types";

const DEFAULT_ENDPOINTS: NetChainEndpoints = {
  rpcUrl: DEFAULT_RPC_URL,
  monitoringUrl: DEFAULT_MONITORING_URL,
  wsUrl: "ws://127.0.0.1:8546",
};

const STORAGE_KEY = "netchain-dashboard-endpoints";
const MAX_RECENT_BLOCKS = 24;
const MAX_RECENT_TRANSACTIONS = 24;
const MAX_RECENT_SLASHES = 12;
const REFRESH_INTERVAL_MS = 30_000;

function readStoredEndpoints(): NetChainEndpoints {
  if (typeof window === "undefined") {
    return DEFAULT_ENDPOINTS;
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return DEFAULT_ENDPOINTS;
    }

    const parsed = JSON.parse(raw) as Partial<NetChainEndpoints>;
    return {
      rpcUrl: parsed.rpcUrl?.trim() || DEFAULT_ENDPOINTS.rpcUrl,
      monitoringUrl:
        parsed.monitoringUrl?.trim() || DEFAULT_ENDPOINTS.monitoringUrl,
      wsUrl: parsed.wsUrl?.trim() || DEFAULT_ENDPOINTS.wsUrl,
    };
  } catch {
    return DEFAULT_ENDPOINTS;
  }
}

function toErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  return typeof error === "string" ? error : "Unknown network error";
}

function mergeBlocks(current: BlockSummary[], incoming: BlockSummary[]) {
  const byIndex = new Map<number, BlockSummary>();
  for (const block of [...current, ...incoming]) {
    byIndex.set(block.index, block);
  }

  return [...byIndex.values()]
    .sort((left, right) => right.index - left.index)
    .slice(0, MAX_RECENT_BLOCKS);
}

function mergeTransactions(
  current: TransactionFeedItem[],
  incoming: TransactionFeedItem[]
) {
  const byHash = new Map<string, TransactionFeedItem>();
  for (const tx of [...incoming, ...current]) {
    byHash.set(tx.tx_hash, tx);
  }

  return [...byHash.values()].slice(0, MAX_RECENT_TRANSACTIONS);
}

function mergeProposals(current: ProposalInfo[], incoming: ProposalInfo[]) {
  const byId = new Map<number, ProposalInfo>();
  for (const proposal of [...current, ...incoming]) {
    const existing = byId.get(proposal.id);
    byId.set(proposal.id, {
      id: proposal.id,
      proposer: proposal.proposer ?? existing?.proposer,
      title: proposal.title ?? existing?.title ?? `Proposal #${proposal.id}`,
      description: proposal.description ?? existing?.description,
      created_at: proposal.created_at ?? existing?.created_at,
      expires_at: proposal.expires_at ?? existing?.expires_at,
      yes_votes: proposal.yes_votes,
      no_votes: proposal.no_votes,
      status: proposal.status ?? existing?.status ?? "Unknown",
      voter_count: proposal.voter_count ?? existing?.voter_count,
    });
  }

  return [...byId.values()].sort((left, right) => {
    const leftRank = left.status === "Active" ? 0 : 1;
    const rightRank = right.status === "Active" ? 0 : 1;

    if (leftRank !== rightRank) {
      return leftRank - rightRank;
    }

    const leftCreated = left.created_at ?? 0;
    const rightCreated = right.created_at ?? 0;
    if (leftCreated !== rightCreated) {
      return rightCreated - leftCreated;
    }

    return right.id - left.id;
  });
}

export interface NetChainWalletView {
  account: AccountInfo | null;
  staking: StakingInfo | null;
  selectedAddress: string | null;
}

export interface UseNetChainResult {
  settings: NetChainEndpoints;
  setSettings: (next: NetChainEndpoints) => void;
  client: NetChainClient;
  isConnected: boolean;
  wsError: string | null;
  rpcError: string | null;
  monitoringError: string | null;
  snapshotStatus: "idle" | "loading" | "ready" | "degraded" | "error";
  lastSyncedAt: string | null;
  health: HealthSnapshot | null;
  metrics: MetricsSnapshot | null;
  chainInfo: ChainInfo | null;
  chainParams: ChainParams | null;
  recentBlocks: BlockSummary[];
  recentTransactions: TransactionFeedItem[];
  proposals: ProposalInfo[];
  slashEvents: ValidatorSlashedEvent[];
  selectedBlock: BlockDetails | null;
  selectedProposal: ProposalInfo | null;
  walletView: NetChainWalletView;
  refreshSnapshot: () => Promise<void>;
  lookupBlock: (index: number) => Promise<BlockDetails | null>;
  lookupProposal: (proposalId: number) => Promise<ProposalInfo | null>;
  lookupAccount: (address: string) => Promise<NetChainWalletView>;
  clearSelections: () => void;
}

export function useNetChain(): UseNetChainResult {
  const [settings, setSettingsState] =
    useState<NetChainEndpoints>(readStoredEndpoints);
  const [isConnected, setIsConnected] = useState(false);
  const [wsError, setWsError] = useState<string | null>(null);
  const [rpcError, setRpcError] = useState<string | null>(null);
  const [monitoringError, setMonitoringError] = useState<string | null>(null);
  const [snapshotStatus, setSnapshotStatus] = useState<
    "idle" | "loading" | "ready" | "degraded" | "error"
  >("idle");
  const [lastSyncedAt, setLastSyncedAt] = useState<string | null>(null);

  const [health, setHealth] = useState<HealthSnapshot | null>(null);
  const [metrics, setMetrics] = useState<MetricsSnapshot | null>(null);
  const [chainInfo, setChainInfo] = useState<ChainInfo | null>(null);
  const [chainParams, setChainParams] = useState<ChainParams | null>(null);
  const [recentBlocks, setRecentBlocks] = useState<BlockSummary[]>([]);
  const [recentTransactions, setRecentTransactions] = useState<
    TransactionFeedItem[]
  >([]);
  const [proposals, setProposals] = useState<ProposalInfo[]>([]);
  const [slashEvents, setSlashEvents] = useState<ValidatorSlashedEvent[]>([]);
  const [selectedBlock, setSelectedBlock] = useState<BlockDetails | null>(null);
  const [selectedProposal, setSelectedProposal] = useState<ProposalInfo | null>(
    null
  );
  const [walletView, setWalletView] = useState<NetChainWalletView>({
    account: null,
    staking: null,
    selectedAddress: null,
  });

  const client = new NetChainClient(settings.rpcUrl, settings.monitoringUrl);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }, [settings]);

  async function refreshSnapshot() {
    setSnapshotStatus("loading");
    setRpcError(null);
    setMonitoringError(null);

    const [healthResult, metricsResult] = await Promise.allSettled([
      client.getHealth(),
      client.getMetrics(),
    ]);

    const [chainInfoResult, chainParamsResult, proposalsResult, blocksResult] =
      await Promise.allSettled([
        client.getChainInfo(),
        client.getChainParams(),
        client.getProposals(),
        client.getBlocks(0, MAX_RECENT_BLOCKS),
      ]);

    let rpcSucceeded = false;
    let monitoringSucceeded = false;
    const rpcErrors: string[] = [];
    const monitoringErrors: string[] = [];

    if (healthResult.status === "fulfilled") {
      setHealth(healthResult.value);
      monitoringSucceeded = true;
    } else {
      monitoringErrors.push(toErrorMessage(healthResult.reason));
    }

    if (metricsResult.status === "fulfilled") {
      setMetrics(metricsResult.value);
      monitoringSucceeded = true;
    } else {
      monitoringErrors.push(toErrorMessage(metricsResult.reason));
    }

    if (chainInfoResult.status === "fulfilled") {
      setChainInfo(chainInfoResult.value);
      rpcSucceeded = true;
    } else {
      rpcErrors.push(toErrorMessage(chainInfoResult.reason));
    }

    if (chainParamsResult.status === "fulfilled") {
      setChainParams(chainParamsResult.value);
      rpcSucceeded = true;
    } else {
      rpcErrors.push(toErrorMessage(chainParamsResult.reason));
    }

    if (proposalsResult.status === "fulfilled") {
      setProposals((current) =>
        mergeProposals(current, proposalsResult.value).slice(0, 12)
      );
      rpcSucceeded = true;
    } else {
      rpcErrors.push(toErrorMessage(proposalsResult.reason));
    }

    if (blocksResult.status === "fulfilled") {
      const incomingBlocks = blocksResult.value.map((block) => ({
        index: block.index,
        hash: block.hash,
        validator: block.validator,
        tx_count: block.transactions.length,
        timestamp: block.timestamp,
      }));

      setRecentBlocks((current) => mergeBlocks(current, incomingBlocks));
      rpcSucceeded = true;
    } else {
      rpcErrors.push(toErrorMessage(blocksResult.reason));
    }

    setRpcError(rpcErrors.length > 0 ? rpcErrors[0] : null);
    setMonitoringError(monitoringErrors.length > 0 ? monitoringErrors[0] : null);

    if (rpcSucceeded && monitoringSucceeded) {
      setSnapshotStatus("ready");
      setLastSyncedAt(new Date().toISOString());
    } else if (rpcSucceeded || monitoringSucceeded) {
      setSnapshotStatus("degraded");
      setLastSyncedAt(new Date().toISOString());
    } else {
      setSnapshotStatus("error");
    }
  }

  async function lookupBlock(index: number) {
    if (!Number.isFinite(index) || index < 0) {
      throw new Error("Block height must be a non-negative number");
    }

    try {
      const block = await client.getBlock(index);
      setSelectedBlock(block);
      setRecentBlocks((current) =>
        mergeBlocks(current, [
          {
            index: block.index,
            hash: block.hash,
            validator: block.validator,
            tx_count: block.transactions.length,
            timestamp: block.timestamp,
          },
        ])
      );
      return block;
    } catch (error) {
      const message = toErrorMessage(error);
      setRpcError(message);
      throw new Error(message);
    }
  }

  async function lookupProposal(proposalId: number) {
    if (!Number.isFinite(proposalId) || proposalId < 0) {
      throw new Error("Proposal id must be a non-negative number");
    }

    try {
      const proposal = await client.getProposal(proposalId);
      setSelectedProposal(proposal);
      setProposals((current) => mergeProposals(current, [proposal]));
      return proposal;
    } catch (error) {
      const message = toErrorMessage(error);
      setRpcError(message);
      throw new Error(message);
    }
  }

  async function lookupAccount(address: string) {
    const normalizedAddress = address.trim();
    if (!normalizedAddress) {
      throw new Error("Address cannot be empty");
    }

    try {
      const [account, staking] = await Promise.all([
        client.getAccount(normalizedAddress),
        client.getStakingInfo(normalizedAddress),
      ]);

      const nextView = {
        account,
        staking,
        selectedAddress: normalizedAddress,
      };
      setWalletView(nextView);
      return nextView;
    } catch (error) {
      const message = toErrorMessage(error);
      setRpcError(message);
      throw new Error(message);
    }
  }

  function clearSelections() {
    setSelectedBlock(null);
    setSelectedProposal(null);
    setWalletView({
      account: null,
      staking: null,
      selectedAddress: null,
    });
  }

  useEffect(() => {
    let active = true;
    let reconnectTimer: number | null = null;
    let socket: WebSocket | null = null;

    const connect = () => {
      if (!active) {
        return;
      }

      setIsConnected(false);

      try {
        socket = new WebSocket(settings.wsUrl);
      } catch (error) {
        setWsError(toErrorMessage(error));
        reconnectTimer = window.setTimeout(connect, 3000);
        return;
      }

      socket.onopen = () => {
        if (!active) {
          return;
        }

        setIsConnected(true);
        setWsError(null);
        socket?.send(
          JSON.stringify({
            action: "subscribe",
            topics: [
              "new_blocks",
              "new_transactions",
              "proposals",
              "slashing",
            ],
          })
        );
      };

      socket.onmessage = (event) => {
        try {
          const payload = JSON.parse(event.data as string) as
            | { type: string; message?: string; topics?: string[] }
            | WsEvent;

          if ("type" in payload) {
            if (payload.type === "error") {
              setWsError(payload.message ?? "WebSocket error");
            }
            return;
          }

          switch (payload.event) {
            case "new_block":
              setRecentBlocks((current) =>
                mergeBlocks(current, [payload.data])
              );
              setChainInfo((current) =>
                current
                  ? {
                      ...current,
                      height: Math.max(current.height, payload.data.index),
                      latest_block_hash: payload.data.hash,
                    }
                  : current
              );
              setHealth((current) =>
                current
                  ? {
                      ...current,
                      chain_height: Math.max(
                        current.chain_height,
                        payload.data.index
                      ),
                    }
                  : current
              );
              setMetrics((current) =>
                current
                  ? {
                      ...current,
                      netchain_chain_height: Math.max(
                        current.netchain_chain_height ?? 0,
                        payload.data.index
                      ),
                    }
                  : current
              );
              break;
            case "new_transaction":
              setRecentTransactions((current) =>
                mergeTransactions(current, [payload.data])
              );
              setHealth((current) =>
                current
                  ? {
                      ...current,
                      mempool_size: current.mempool_size + 1,
                    }
                  : current
              );
              setMetrics((current) =>
                current
                  ? {
                      ...current,
                      netchain_mempool_size:
                        (current.netchain_mempool_size ?? 0) + 1,
                    }
                  : current
              );
              break;
            case "proposal_update": {
              const proposal = payload.data;
              setProposals((current) => mergeProposals(current, [proposal]));
              setSelectedProposal((current) =>
                current && current.id === proposal.id
                  ? {
                      ...current,
                      ...proposal,
                    }
                  : current
              );
              break;
            }
            case "validator_slashed":
              setSlashEvents((current) => [
                payload.data,
                ...current,
              ].slice(0, MAX_RECENT_SLASHES));
              break;
            default:
              break;
          }
        } catch (error) {
          setWsError(toErrorMessage(error));
        }
      };

      socket.onerror = () => {
        if (!active) {
          return;
        }

        setWsError("WebSocket connection failed");
        setIsConnected(false);
      };

      socket.onclose = () => {
        if (!active) {
          return;
        }

        setIsConnected(false);
        if (!wsError) {
          setWsError("WebSocket disconnected");
        }
        reconnectTimer = window.setTimeout(connect, 3000);
      };
    };

    connect();

    return () => {
      active = false;
      if (reconnectTimer !== null) {
        window.clearTimeout(reconnectTimer);
      }
      socket?.close();
    };
    // wsUrl changes should restart the connection.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings.wsUrl]);

  useEffect(() => {
    void refreshSnapshot();
    const interval = window.setInterval(() => {
      void refreshSnapshot();
    }, REFRESH_INTERVAL_MS);

    return () => {
      window.clearInterval(interval);
    };
    // refreshSnapshot is intentionally re-evaluated with the current settings.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings.rpcUrl, settings.monitoringUrl]);

  return {
    settings,
    setSettings: setSettingsState,
    client,
    isConnected,
    wsError,
    rpcError,
    monitoringError,
    snapshotStatus,
    lastSyncedAt,
    health,
    metrics,
    chainInfo,
    chainParams,
    recentBlocks,
    recentTransactions,
    proposals,
    slashEvents,
    selectedBlock,
    selectedProposal,
    walletView,
    refreshSnapshot,
    lookupBlock,
    lookupProposal,
    lookupAccount,
    clearSelections,
  };
}
