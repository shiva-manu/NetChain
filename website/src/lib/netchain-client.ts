import type {
  AccountInfo,
  BlockDetails,
  ChainInfo,
  ChainParams,
  HealthSnapshot,
  MetricsSnapshot,
  ProposalInfo,
  StakingInfo,
} from "@/lib/types";

type RpcEnvelope<T> =
  | { status: "success"; data: T }
  | { status: "error"; message: string };

type RpcBody = {
  method: string;
  params?: unknown;
};

// Live NetChain node endpoints
const DEFAULT_RPC_URL = "https://api.netchain.me";
const DEFAULT_MONITORING_URL = "https://api.netchain.me";

function trimTrailingSlash(url: string) {
  return url.replace(/\/+$/, "");
}

function resolveRpcUrl(baseUrl: string) {
  const trimmed = trimTrailingSlash(baseUrl);
  return trimmed.endsWith("/rpc") ? trimmed : `${trimmed}/rpc`;
}

function resolveMonitoringUrl(baseUrl: string, path: "health" | "metrics") {
  return `${trimTrailingSlash(baseUrl)}/${path}`;
}

export function parseMetrics(text: string): MetricsSnapshot {
  const metrics: MetricsSnapshot = {};

  for (const line of text.split("\n")) {
    if (!line || line.startsWith("#")) continue;

    const match = line.match(/^(\w+)(?:\{[^}]*\})?\s+([0-9.]+)/);
    if (!match) continue;

    const [, key, value] = match;
    metrics[key as keyof MetricsSnapshot] = Number.parseFloat(value);
  }

  return metrics;
}

export class NetChainClient {
  public readonly rpcUrl: string;
  public readonly monitoringUrl: string;

  constructor(
    rpcUrl: string = DEFAULT_RPC_URL,
    monitoringUrl: string = DEFAULT_MONITORING_URL
  ) {
    this.rpcUrl = rpcUrl;
    this.monitoringUrl = monitoringUrl;
  }

  async request<T>(method: string, params?: unknown): Promise<T> {
    const body: RpcBody =
      params === undefined ? { method } : { method, params };

    const response = await fetch(resolveRpcUrl(this.rpcUrl), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      throw new Error(`RPC request failed with status ${response.status}`);
    }

    const envelope = (await response.json()) as RpcEnvelope<T>;
    if (envelope.status === "error") {
      throw new Error(envelope.message);
    }

    return envelope.data;
  }

  async getChainInfo(): Promise<ChainInfo> {
    return this.request<ChainInfo>("get_chain_info");
  }

  async getMempoolSize(): Promise<{ size: number }> {
    return this.request<{ size: number }>("get_mempool_size");
  }

  async getBlocks(startHeight = 0, limit = 25): Promise<BlockDetails[]> {
    return this.request<BlockDetails[]>("get_blocks", {
      start_height: startHeight,
      limit,
    });
  }

  async getBlock(index: number): Promise<BlockDetails> {
    return this.request<BlockDetails>("get_block", { index });
  }

  async getProposals(): Promise<ProposalInfo[]> {
    return this.request<ProposalInfo[]>("get_proposals");
  }

  async getProposal(proposalId: number): Promise<ProposalInfo> {
    return this.request<ProposalInfo>("get_proposal", { proposal_id: proposalId });
  }

  async getAccount(address: string): Promise<AccountInfo> {
    return this.request<AccountInfo>("get_account", { address });
  }

  async getStakingInfo(address: string): Promise<StakingInfo> {
    return this.request<StakingInfo>("get_staking_info", { address });
  }

  async getChainParams(): Promise<ChainParams> {
    return this.request<ChainParams>("get_chain_params");
  }

  async getMetrics(): Promise<MetricsSnapshot> {
    const response = await fetch(resolveMonitoringUrl(this.monitoringUrl, "metrics"));
    if (!response.ok) {
      throw new Error(`Metrics request failed with status ${response.status}`);
    }

    return parseMetrics(await response.text());
  }

  async getHealth(): Promise<HealthSnapshot> {
    const response = await fetch(resolveMonitoringUrl(this.monitoringUrl, "health"));
    if (!response.ok) {
      throw new Error(`Health request failed with status ${response.status}`);
    }

    return response.json();
  }

  async requestTokens(address: string): Promise<string> {
    if (!address) {
      throw new Error("Invalid address");
    }

    const result = await this.request<{ tx_hash: string }>("faucet_tokens", { address });
    return result.tx_hash;
  }
}

export { DEFAULT_MONITORING_URL, DEFAULT_RPC_URL };
