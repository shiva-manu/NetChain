import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import {
  Activity,
  AlertTriangle,
  ArrowDown,
  ArrowUp,
  Blocks,
  Clock3,
  Copy,
  FileText,
  Gauge,
  Globe,
  Layers3,
  Loader2,
  Plus,
  Radar,
  RefreshCw,
  Search,
  Send,
  Settings,
  Shield,
  Trash2,
  Vote,
  Wallet,
  ChevronRight,
  Check,
} from "lucide-react";

import { SEO } from "@/components/seo";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { SectionBackground } from "@/components/sections/section-background";
import {
  DEFAULT_MONITORING_URL,
  DEFAULT_RPC_URL,
} from "@/lib/netchain-client";
import { useNetChain } from "@/lib/use-netchain";
import type {
  AccountInfo,
  BlockDetails,
  NetChainEndpoints,
  ProposalInfo,
  SignedTransaction,
  StakingInfo,
  TransactionType,
  WalletWatchItem,
} from "@/lib/types";
import { cn } from "@/lib/utils";

const DEFAULT_ENDPOINTS: NetChainEndpoints = {
  rpcUrl: DEFAULT_RPC_URL,
  monitoringUrl: DEFAULT_MONITORING_URL,
  wsUrl: "ws://127.0.0.1:8546",
};

const WATCHLIST_STORAGE_KEY = "netchain-dashboard-watchlist";

function readStoredWatchlist(): WalletWatchItem[] {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    const raw = window.localStorage.getItem(WATCHLIST_STORAGE_KEY);
    if (!raw) {
      return [];
    }

    const parsed = JSON.parse(raw) as WalletWatchItem[];
    return Array.isArray(parsed) ? parsed.filter(Boolean) : [];
  } catch {
    return [];
  }
}

function truncateToken(value: string, head = 10, tail = 6) {
  if (!value || value.length <= head + tail + 3) {
    return value;
  }

  return `${value.slice(0, head)}...${value.slice(-tail)}`;
}

function formatCount(value: number | null | undefined) {
  return value == null ? "—" : value.toLocaleString();
}

function formatCompactNumber(value: number | null | undefined) {
  if (value == null || Number.isNaN(value)) {
    return "—";
  }

  return new Intl.NumberFormat("en-US", {
    maximumFractionDigits: 2,
  }).format(value);
}

function formatDuration(totalSeconds: number | null | undefined) {
  if (totalSeconds == null || Number.isNaN(totalSeconds)) {
    return "—";
  }

  const seconds = Math.max(0, Math.floor(totalSeconds));
  const days = Math.floor(seconds / 86_400);
  const hours = Math.floor((seconds % 86_400) / 3_600);
  const minutes = Math.floor((seconds % 3_600) / 60);

  if (days > 0) {
    return `${days}d ${hours}h`;
  }

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  if (minutes > 0) {
    return `${minutes}m`;
  }

  return `${seconds}s`;
}

function formatDateTime(value: string | number | null | undefined) {
  if (value == null) {
    return "—";
  }

  const date =
    typeof value === "number" ? new Date(value * 1000) : new Date(value);
  if (Number.isNaN(date.getTime())) {
    return String(value);
  }

  return date.toLocaleString();
}

function formatAddress(address: string) {
  return truncateToken(address, 12, 8);
}

function formatHash(hash: string) {
  return truncateToken(hash, 12, 8);
}

function shellQuote(value: string) {
  return JSON.stringify(value);
}

function describeTxType(txType: TransactionType | string | null | undefined) {
  if (!txType) {
    return "Unknown";
  }

  if (typeof txType === "string") {
    return txType;
  }

  const [kind, payload] = Object.entries(txType)[0] ?? ["Unknown", null];
  if (!payload || typeof payload !== "object") {
    return kind;
  }

  const data = payload as Record<string, unknown>;
  if (kind === "CreateProposal") {
    const title = typeof data.title === "string" ? data.title : "";
    return title ? `Create proposal: ${title}` : "Create proposal";
  }

  if (kind === "VoteProposal") {
    const id =
      typeof data.proposal_id === "number" ? `#${data.proposal_id}` : "";
    const support =
      typeof data.support === "boolean"
        ? data.support
          ? "yes"
          : "no"
        : "";
    return `Vote proposal ${id}${support ? ` (${support})` : ""}`;
  }

  return kind;
}

function statusTone(status: string | null | undefined) {
  const normalized = (status ?? "").toLowerCase();
  if (
    normalized.includes("ok") ||
    normalized.includes("active") ||
    normalized.includes("passed") ||
    normalized.includes("connected")
  ) {
    return "bg-tertiary/10 text-tertiary";
  }

  if (
    normalized.includes("error") ||
    normalized.includes("failed") ||
    normalized.includes("rejected") ||
    normalized.includes("offline") ||
    normalized.includes("expired")
  ) {
    return "bg-destructive/10 text-destructive";
  }

  return "bg-accent/10 text-accent";
}

function proposalProgress(proposal: ProposalInfo | null) {
  if (!proposal) {
    return { yes: 0, no: 0, total: 0 };
  }

  const yes = proposal.yes_votes ?? 0;
  const no = proposal.no_votes ?? 0;
  return { yes, no, total: yes + no };
}

function copyToClipboard(value: string) {
  if (!value) {
    return;
  }

  void navigator.clipboard.writeText(value);
}

// ============================================================================
// COMPONENTS
// ============================================================================

type MetricCardProps = {
  label: string;
  value: string;
  detail: string;
  icon: LucideIcon;
  variant?: "default" | "primary" | "tertiary" | "accent";
};

function MetricCard({
  label,
  value,
  detail,
  icon: Icon,
  variant = "default",
}: MetricCardProps) {
  const iconVariants = {
    default: "bg-foreground/10 text-foreground",
    primary: "bg-primary/15 text-primary",
    tertiary: "bg-tertiary/15 text-tertiary",
    accent: "bg-accent/15 text-accent",
  };

  return (
    <div className="group relative overflow-hidden rounded-xl border border-border bg-card p-6 transition-all duration-500 hover:border-primary/30 hover:bg-surface-hover">
      <div className="absolute inset-0 bg-gradient-to-br from-primary/5 to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100" />

      <div className="relative flex items-start justify-between gap-4">
        <div className="space-y-2">
          <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
            {label}
          </p>
          <div className="text-3xl font-bold tracking-tight text-foreground">
            {value}
          </div>
        </div>
        <div
          className={cn(
            "flex size-12 items-center justify-center rounded-xl transition-all duration-300 group-hover:scale-110",
            iconVariants[variant]
          )}
        >
          <Icon className="size-5" />
        </div>
      </div>
      <div className="relative mt-4 pt-4 border-t border-border">
        <p className="text-xs text-muted-foreground">
          {detail}
        </p>
      </div>
    </div>
  );
}

type DetailRowProps = {
  label: string;
  value: ReactNode;
  mono?: boolean;
};

function DetailRow({ label, value, mono = false }: DetailRowProps) {
  return (
    <div className="flex flex-col gap-1.5 rounded-xl border border-border bg-card px-4 py-3 transition-all hover:bg-surface-hover">
      <div className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
        {label}
      </div>
      <div
        className={cn(
          "font-medium text-foreground",
          mono ? "font-mono text-xs break-all" : "text-sm"
        )}
      >
        {value}
      </div>
    </div>
  );
}

function EmptyPane({
  icon: Icon,
  title,
  description,
}: {
  icon: LucideIcon;
  title: string;
  description: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed border-border bg-card/50 px-5 py-10 text-center">
      <div className="flex size-12 items-center justify-center rounded-xl bg-primary/10 text-primary">
        <Icon className="size-5" />
      </div>
      <div>
        <p className="text-sm font-medium text-foreground">{title}</p>
        <p className="mt-1 text-sm text-muted-foreground">{description}</p>
      </div>
    </div>
  );
}

function BlockDetailCard({ block }: { block: BlockDetails | null }) {
  return (
    <Card variant="glass">
      <CardHeader className="border-b border-border">
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2 text-lg text-foreground">
              <Layers3 className="size-4 text-primary" />
              Block Inspector
            </CardTitle>
            <CardDescription className="text-muted-foreground">
              Signed payloads, validator, and commitment data.
            </CardDescription>
          </div>
          {block && (
            <Badge variant="glass" className="gap-1.5">
              <Blocks className="size-3.5" />
              #{block.index}
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4 pt-4">
        {!block ? (
          <EmptyPane
            icon={Blocks}
            title="No block selected"
            description="Load a block from the search bar or click any recent block to inspect."
          />
        ) : (
          <>
            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
              <DetailRow
                label="Hash"
                value={
                  <div className="flex items-center gap-2">
                    <span className="font-mono">{formatHash(block.hash)}</span>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="size-6 text-muted-foreground hover:text-primary"
                      onClick={() => copyToClipboard(block.hash)}
                      aria-label="Copy block hash"
                    >
                      <Copy className="size-3.5" />
                    </Button>
                  </div>
                }
                mono
              />
              <DetailRow
                label="Validator"
                value={formatAddress(block.validator)}
                mono
              />
              <DetailRow
                label="Timestamp"
                value={formatDateTime(block.timestamp)}
              />
              <DetailRow
                label="Previous hash"
                value={formatHash(block.previous_hash)}
                mono
              />
              <DetailRow
                label="Merkle root"
                value={formatHash(block.merkle_root)}
                mono
              />
              <DetailRow
                label="Transactions"
                value={`${block.transactions.length} signed payloads`}
              />
            </div>

            <div className="rounded-xl border border-border bg-card">
              <div className="flex items-center justify-between border-b border-border px-4 py-3">
                <div className="flex items-center gap-2">
                  <FileText className="size-4 text-muted-foreground" />
                  <span className="text-sm font-medium text-foreground">
                    Transaction payloads
                  </span>
                </div>
                <span className="text-xs text-muted-foreground">
                  {block.transactions.length} total
                </span>
              </div>
              <div className="max-h-[26rem] divide-y divide-border overflow-auto">
                {block.transactions.length === 0 ? (
                  <div className="px-4 py-6 text-sm text-muted-foreground">
                    No transactions in this block.
                  </div>
                ) : (
                  block.transactions.map((transaction, index) => (
                    <BlockTransactionRow
                      key={`${transaction.signature}-${index}`}
                      transaction={transaction}
                      index={index}
                    />
                  ))
                )}
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}

function BlockTransactionRow({
  transaction,
  index,
}: {
  transaction: SignedTransaction;
  index: number;
}) {
  return (
    <div className="grid gap-3 px-4 py-4 lg:grid-cols-[1.1fr_1.1fr_auto_auto_auto]">
      <div>
        <div className="text-[10px] uppercase tracking-wider text-muted-foreground">
          Transaction {index + 1}
        </div>
        <div className="mt-1 font-medium text-foreground">
          {describeTxType(transaction.tx.tx_type)}
        </div>
      </div>
      <div>
        <div className="text-[10px] uppercase tracking-wider text-muted-foreground">
          Route
        </div>
        <div className="mt-1 font-mono text-xs text-foreground/80">
          {formatAddress(transaction.tx.sender)}{" "}
          <ChevronRight className="inline size-3 align-text-top text-muted-foreground" />{" "}
          {transaction.tx.receiver
            ? formatAddress(transaction.tx.receiver)
            : "Protocol"}
        </div>
      </div>
      <DetailRow label="Amount" value={formatCount(transaction.tx.amount)} />
      <DetailRow label="Fee" value={formatCount(transaction.tx.fee)} />
      <DetailRow label="Nonce" value={formatCount(transaction.tx.nonce)} />
      <div className="grid gap-3 lg:col-span-5 lg:grid-cols-2">
        <DetailRow
          label="Signature"
          value={
            <span className="font-mono text-xs">
              {truncateToken(transaction.signature, 12, 10)}
            </span>
          }
          mono
        />
        <DetailRow
          label="Pubkey"
          value={
            <span className="font-mono text-xs">
              {truncateToken(transaction.pubkey, 12, 10)}
            </span>
          }
          mono
        />
      </div>
      {transaction.tx.memo && (
        <div className="lg:col-span-5">
          <DetailRow label="Memo" value={transaction.tx.memo} />
        </div>
      )}
    </div>
  );
}

function ProposalDetailCard({ proposal }: { proposal: ProposalInfo | null }) {
  const { yes, no, total } = proposalProgress(proposal);
  const yesWidth = total > 0 ? (yes / total) * 100 : 0;
  const noWidth = total > 0 ? (no / total) * 100 : 0;

  return (
    <Card variant="glass">
      <CardHeader className="border-b border-border">
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2 text-lg text-foreground">
              <Vote className="size-4 text-accent" />
              Proposal Inspector
            </CardTitle>
            <CardDescription className="text-muted-foreground">
              Governance metadata, vote pressure, and expiry timing.
            </CardDescription>
          </div>
          {proposal && (
            <Badge
              variant="glass"
              className={cn("border-transparent", statusTone(proposal.status))}
            >
              {proposal.status}
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4 pt-4">
        {!proposal ? (
          <EmptyPane
            icon={Vote}
            title="No proposal selected"
            description="Search by proposal ID or click a proposal to inspect."
          />
        ) : (
          <>
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-sm font-semibold text-foreground">
                  {proposal.title}
                </p>
                <p className="mt-1 text-sm leading-relaxed text-muted-foreground">
                  {proposal.description || "No description provided."}
                </p>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="text-muted-foreground hover:text-primary"
                onClick={() => copyToClipboard(String(proposal.id))}
                aria-label="Copy proposal ID"
              >
                <Copy className="size-4" />
              </Button>
            </div>

            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
              <DetailRow label="Proposal ID" value={`#${proposal.id}`} mono />
              <DetailRow
                label="Proposer"
                value={proposal.proposer ? formatAddress(proposal.proposer) : "—"}
                mono
              />
              <DetailRow
                label="Voters"
                value={formatCount(proposal.voter_count)}
              />
              <DetailRow
                label="Created"
                value={formatDateTime(proposal.created_at)}
              />
              <DetailRow
                label="Expires"
                value={formatDateTime(proposal.expires_at)}
              />
              <DetailRow
                label="Votes"
                value={`${formatCount(proposal.yes_votes)} yes / ${formatCount(
                  proposal.no_votes
                )} no`}
              />
            </div>

            <div className="space-y-3 rounded-xl border border-border bg-card p-4">
              <div className="flex items-center justify-between text-xs uppercase tracking-wider text-muted-foreground">
                <span>Vote pressure</span>
                <span>{total} total votes</span>
              </div>
              <div className="flex h-2 overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full bg-tertiary transition-[width] duration-300"
                  style={{ width: `${yesWidth}%` }}
                />
                <div
                  className="h-full bg-destructive transition-[width] duration-300"
                  style={{ width: `${noWidth}%` }}
                />
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-tertiary">
                  Yes: {formatCount(proposal.yes_votes)}
                </span>
                <span className="text-destructive">
                  No: {formatCount(proposal.no_votes)}
                </span>
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}

function WalletInspectorCard({
  account,
  staking,
  selectedAddress,
}: {
  account: AccountInfo | null;
  staking: StakingInfo | null;
  selectedAddress: string | null;
}) {
  return (
    <Card variant="glass">
      <CardHeader className="border-b border-border">
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2 text-lg text-foreground">
              <Wallet className="size-4 text-tertiary" />
              Wallet Inspector
            </CardTitle>
            <CardDescription className="text-muted-foreground">
              Watch-only balances, nonce tracking, and staking.
            </CardDescription>
          </div>
          {selectedAddress && (
            <Badge variant="glass" className="gap-1.5">
              <Shield className="size-3.5" />
              watch-only
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4 pt-4">
        {!account || !staking ? (
          <EmptyPane
            icon={Wallet}
            title="No wallet selected"
            description="Load an address from the watchlist to inspect balance and stake."
          />
        ) : (
          <div className="grid gap-3 md:grid-cols-2">
            <DetailRow
              label="Address"
              value={
                <span className="font-mono text-xs">
                  {formatAddress(account.address)}
                </span>
              }
              mono
            />
            <DetailRow label="Balance" value={formatCount(account.balance)} />
            <DetailRow label="Nonce" value={formatCount(account.nonce)} />
            <DetailRow
              label="Staked balance"
              value={formatCount(account.staked_balance)}
            />
            <DetailRow
              label="Total staked"
              value={formatCount(staking.total_staked)}
            />
            <DetailRow
              label="Selected wallet"
              value={selectedAddress ? formatAddress(selectedAddress) : "—"}
              mono
            />
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function AppShellError({
  icon: Icon,
  title,
  message,
}: {
  icon: LucideIcon;
  title: string;
  message: string;
}) {
  return (
    <div className="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm">
      <div className="flex items-start gap-3">
        <div className="mt-0.5 flex size-8 items-center justify-center rounded-lg bg-destructive/20 text-destructive">
          <Icon className="size-4" />
        </div>
        <div>
          <p className="font-medium text-destructive">{title}</p>
          <p className="mt-1 text-sm text-destructive/80">
            {message}
          </p>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// MAIN DASHBOARD
// ============================================================================

export function Dashboard() {
  const {
    settings,
    setSettings,
    isConnected,
    wsError,
    rpcError,
    monitoringError,
    snapshotStatus,
    lastSyncedAt,
    health,
    metrics,
    chainInfo,
    chainParams: _chainParams,
    recentBlocks,
    recentTransactions,
    proposals,
    slashEvents: _slashEvents,
    selectedBlock,
    selectedProposal,
    walletView,
    refreshSnapshot,
    lookupBlock,
    lookupProposal,
    lookupAccount,
    clearSelections,
  } = useNetChain();

  const [endpointDraft, setEndpointDraft] = useState(settings);
  const [watchlist, setWatchlist] = useState<WalletWatchItem[]>(
    readStoredWatchlist
  );
  const [watchLabel, setWatchLabel] = useState("");
  const [watchAddress, setWatchAddress] = useState("");
  const [blockQuery, setBlockQuery] = useState("");
  const [accountQuery, setAccountQuery] = useState("");
  const [draftFrom, setDraftFrom] = useState("");
  const [draftTo, setDraftTo] = useState("");
  const [draftAmount, setDraftAmount] = useState("");
  const [draftFee, setDraftFee] = useState("1");
  const [draftMemo] = useState("");
  const [blockLoading, setBlockLoading] = useState(false);
  const [copiedCommand, setCopiedCommand] = useState(false);

  useEffect(() => {
    setEndpointDraft(settings);
  }, [settings]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    window.localStorage.setItem(
      WATCHLIST_STORAGE_KEY,
      JSON.stringify(watchlist)
    );
  }, [watchlist]);

  useEffect(() => {
    if (!accountQuery && watchlist[0]) {
      setAccountQuery(watchlist[0].address);
    }
  }, [watchlist, accountQuery]);

  useEffect(() => {
    if (walletView.selectedAddress && !draftFrom) {
      setDraftFrom(walletView.selectedAddress);
    }
  }, [draftFrom, walletView.selectedAddress]);

  const latestHeight = chainInfo?.height ?? health?.chain_height ?? null;
  const activeProposalCount = proposals.filter((proposal) =>
    proposal.status.toLowerCase().includes("active")
  ).length;

  const endpointChanges =
    endpointDraft.rpcUrl.trim() !== settings.rpcUrl ||
    endpointDraft.monitoringUrl.trim() !== settings.monitoringUrl ||
    endpointDraft.wsUrl.trim() !== settings.wsUrl;

  const walletCommand = [
    "netchain-wallet send",
    `--rpc ${settings.rpcUrl}`,
    `--from ${draftFrom || "<wallet>"}`,
    `--to ${draftTo || "<recipient>"}`,
    `--amount ${draftAmount || "<amount>"}`,
    `--fee ${draftFee || "1"}`,
    draftMemo.trim() ? `--memo ${shellQuote(draftMemo.trim())}` : "",
  ]
    .filter(Boolean)
    .join(" \\\n  ");

  async function handleRefresh() {
    await refreshSnapshot();
  }

  async function handleLoadBlock(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = blockQuery.trim();
    const height = trimmed ? Number(trimmed) : latestHeight;

    if (height == null || Number.isNaN(height)) {
      return;
    }

    setBlockLoading(true);
    try {
      await lookupBlock(height);
    } finally {
      setBlockLoading(false);
    }
  }

  function handleInspectWatchAddress(address: string) {
    setAccountQuery(address);
    setDraftFrom(address);
    void lookupAccount(address).catch(() => undefined);
  }

  function handleAddWatchItem(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedAddress = watchAddress.trim();
    if (!normalizedAddress) {
      return;
    }

    const nextItem: WalletWatchItem = {
      label: watchLabel.trim() || formatAddress(normalizedAddress),
      address: normalizedAddress,
    };

    setWatchlist((current) => {
      const existingIndex = current.findIndex(
        (item) => item.address === normalizedAddress
      );
      if (existingIndex >= 0) {
        const next = [...current];
        next[existingIndex] = nextItem;
        return next;
      }

      return [nextItem, ...current].slice(0, 12);
    });

    setWatchLabel("");
    setWatchAddress("");
  }

  function handleRemoveWatchAddress(address: string) {
    setWatchlist((current) =>
      current.filter((item) => item.address !== address)
    );
  }

  function handleApplyEndpoints() {
    const nextSettings: NetChainEndpoints = {
      rpcUrl: endpointDraft.rpcUrl.trim() || DEFAULT_ENDPOINTS.rpcUrl,
      monitoringUrl:
        endpointDraft.monitoringUrl.trim() || DEFAULT_ENDPOINTS.monitoringUrl,
      wsUrl: endpointDraft.wsUrl.trim() || DEFAULT_ENDPOINTS.wsUrl,
    };

    setEndpointDraft(nextSettings);
    setSettings(nextSettings);
  }

  function handleResetEndpoints() {
    setEndpointDraft(DEFAULT_ENDPOINTS);
    setSettings(DEFAULT_ENDPOINTS);
  }

  function handleCopyCommand() {
    copyToClipboard(walletCommand);
    setCopiedCommand(true);
    setTimeout(() => setCopiedCommand(false), 2000);
  }

  return (
    <div className="relative min-h-screen">
      <SEO
        title="Dashboard - NetChain Explorer"
        description="Explore the NetChain blockchain in real-time: view blocks, transactions, validator metrics, governance proposals, and network health."
        keywords="NetChain explorer, blockchain explorer, block explorer, transaction explorer, validator metrics"
      />

      {/* Hero Section */}
      <section className="relative pt-32 pb-12 overflow-hidden">
        <SectionBackground variant="gradient" />

        <div className="container-wide relative z-10">
          <div className="flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between">
            <div className="space-y-4">
              <div className="flex flex-wrap items-center gap-3">
                <Badge variant="signal" className="gap-1.5">
                  <span className="size-1.5 rounded-full bg-tertiary animate-pulse" />
                  Live
                </Badge>
                <Badge variant="glass" className="gap-1.5">
                  <Radar className="size-3" />
                  {isConnected ? "Connected" : "Offline"}
                </Badge>
                <Badge variant="glass" className="gap-1.5">
                  <Clock3 className="size-3" />
                  {lastSyncedAt ? formatDateTime(lastSyncedAt) : "Not synced"}
                </Badge>
              </div>

              <h1 className="text-4xl font-bold tracking-tight sm:text-5xl">
                Network{" "}
                <span className="text-gradient">
                  Dashboard
                </span>
              </h1>
              <p className="text-lg text-muted-foreground max-w-xl">
                Monitor network health, inspect transactions, track wallet
                balances, and participate in governance.
              </p>
            </div>

            <div className="flex flex-wrap items-center gap-4">
              <div className="flex items-center gap-6 px-6 py-4 rounded-xl border border-border bg-card">
                <div className="space-y-0.5">
                  <div className="text-xs text-muted-foreground">Status</div>
                  <div className="text-sm font-medium text-tertiary">Operational</div>
                </div>
                <div className="h-8 w-px bg-border" />
                <div className="space-y-0.5">
                  <div className="text-xs text-muted-foreground">Height</div>
                  <div className="text-sm font-mono font-medium text-primary">
                    #{formatCount(health?.chain_height)}
                  </div>
                </div>
              </div>

              <Button
                onClick={() => void handleRefresh()}
                disabled={snapshotStatus === "loading"}
                className="h-12"
              >
                <RefreshCw
                  className={cn(
                    "size-4",
                    snapshotStatus === "loading" && "animate-spin"
                  )}
                />
                Sync
              </Button>
              <Button
                variant="outline"
                onClick={clearSelections}
                className="h-12"
              >
                Reset
              </Button>
            </div>
          </div>
        </div>
      </section>

      {/* Metrics Grid */}
      <section className="container-wide pb-12">
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
          <MetricCard
            label="Block Height"
            value={`#${formatCount(health?.chain_height)}`}
            detail="Latest committed block index"
            icon={Blocks}
            variant="primary"
          />
          <MetricCard
            label="Mempool"
            value={formatCount(health?.mempool_size)}
            detail="Pending transactions"
            icon={Activity}
            variant="tertiary"
          />
          <MetricCard
            label="Peers"
            value={formatCount(health?.peer_count)}
            detail="Active connections"
            icon={Globe}
            variant="accent"
          />
          <MetricCard
            label="Validators"
            value={formatCount(health?.validator_count)}
            detail="Active validator set"
            icon={Shield}
            variant="primary"
          />
        </div>
      </section>

      {/* Main Content */}
      <section className="container-wide pb-24">
        <div className="grid gap-8 lg:grid-cols-[1fr_400px]">
          {/* Left Column */}
          <div className="space-y-8">
            {/* Telemetry Card */}
            <Card variant="glass">
              <CardHeader className="border-b border-border">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Radar className="size-5 text-primary" />
                    <CardTitle className="text-foreground">Node Telemetry</CardTitle>
                  </div>
                  <Badge variant="signal">Live</Badge>
                </div>
                <CardDescription className="text-muted-foreground">
                  Connectivity, bandwidth, and network state
                </CardDescription>
              </CardHeader>
              <CardContent className="p-6 space-y-6">
                {(rpcError || monitoringError || wsError) && (
                  <div className="space-y-3">
                    {rpcError && (
                      <AppShellError
                        icon={AlertTriangle}
                        title="RPC Error"
                        message={rpcError}
                      />
                    )}
                    {monitoringError && (
                      <AppShellError
                        icon={AlertTriangle}
                        title="Monitoring Error"
                        message={monitoringError}
                      />
                    )}
                    {wsError && (
                      <AppShellError
                        icon={AlertTriangle}
                        title="WebSocket Error"
                        message={wsError}
                      />
                    )}
                  </div>
                )}

                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                  <DetailRow
                    label="Uptime"
                    value={formatDuration(health?.uptime_secs)}
                  />
                  <DetailRow
                    label="Latency"
                    value={
                      metrics?.netchain_latency_ms != null
                        ? `${formatCompactNumber(metrics.netchain_latency_ms)} ms`
                        : "—"
                    }
                  />
                  <DetailRow
                    label="Epoch"
                    value={formatCount(health?.current_epoch)}
                  />
                  <DetailRow
                    label="Proposals"
                    value={formatCount(activeProposalCount)}
                  />
                </div>

                <div className="grid gap-4 lg:grid-cols-2">
                  <div className="rounded-xl border border-border bg-card p-5 space-y-3">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                        <ArrowDown className="size-3.5 text-primary" />
                        Download
                      </div>
                      <Badge variant="glass">Stable</Badge>
                    </div>
                    <div className="text-2xl font-bold text-foreground">
                      {metrics?.netchain_download_mbps != null
                        ? `${formatCompactNumber(metrics.netchain_download_mbps)} Mbps`
                        : "—"}
                    </div>
                    <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden">
                      <div className="h-full w-[65%] bg-gradient-to-r from-primary to-accent rounded-full" />
                    </div>
                  </div>

                  <div className="rounded-xl border border-border bg-card p-5 space-y-3">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                        <ArrowUp className="size-3.5 text-tertiary" />
                        Upload
                      </div>
                      <Badge variant="glass">Stable</Badge>
                    </div>
                    <div className="text-2xl font-bold text-foreground">
                      {metrics?.netchain_upload_mbps != null
                        ? `${formatCompactNumber(metrics.netchain_upload_mbps)} Mbps`
                        : "—"}
                    </div>
                    <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden">
                      <div className="h-full w-[45%] bg-gradient-to-r from-tertiary to-tertiary/70 rounded-full" />
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Explorer Card */}
            <Card variant="glass">
              <CardHeader className="border-b border-border">
                <div className="flex items-center gap-2">
                  <Search className="size-5 text-primary" />
                  <CardTitle className="text-foreground">Explorer</CardTitle>
                </div>
                <CardDescription className="text-muted-foreground">
                  Search blocks, proposals, and addresses
                </CardDescription>
              </CardHeader>
              <CardContent className="p-6 space-y-8">
                <form onSubmit={handleLoadBlock} className="flex gap-3">
                  <div className="relative flex-1">
                    <Search className="absolute left-4 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      placeholder="Search block height..."
                      className="pl-11 h-11"
                      value={blockQuery}
                      onChange={(e) => setBlockQuery(e.target.value)}
                    />
                  </div>
                  <Button type="submit" disabled={blockLoading} className="h-11">
                    {blockLoading ? (
                      <Loader2 className="animate-spin size-4" />
                    ) : (
                      "Search"
                    )}
                  </Button>
                </form>

                <div className="grid gap-8 lg:grid-cols-2">
                  {/* Recent Blocks */}
                  <div className="space-y-4">
                    <div className="flex items-center justify-between border-b border-border pb-3">
                      <div className="flex items-center gap-2">
                        <Blocks className="size-4 text-primary" />
                        <h3 className="text-sm font-semibold text-foreground">Latest Blocks</h3>
                      </div>
                      <Badge variant="glass">PoI</Badge>
                    </div>
                    <div className="space-y-2">
                      {recentBlocks.length > 0 ? (
                        recentBlocks.map((block) => (
                          <button
                            key={block.hash}
                            onClick={() => void lookupBlock(block.index)}
                            className={cn(
                              "group flex w-full flex-col gap-2 rounded-xl border border-border bg-card p-4 text-left transition-all hover:bg-surface-hover hover:border-primary/30",
                              selectedBlock?.hash === block.hash &&
                                "border-primary/50 bg-primary/5"
                            )}
                          >
                            <div className="flex items-center justify-between">
                              <span className="font-mono text-xs font-medium text-primary">
                                #{block.index}
                              </span>
                              <span className="text-xs text-muted-foreground">
                                {new Date(block.timestamp).toLocaleTimeString()}
                              </span>
                            </div>
                            <div className="font-mono text-[10px] text-muted-foreground group-hover:text-foreground/60 break-all">
                              {block.hash}
                            </div>
                          </button>
                        ))
                      ) : (
                        <EmptyPane
                          title="No blocks"
                          description="Unable to fetch recent blocks."
                          icon={Blocks}
                        />
                      )}
                    </div>
                  </div>

                  {/* Gossip Feed */}
                  <div className="space-y-4">
                    <div className="flex items-center justify-between border-b border-border pb-3">
                      <div className="flex items-center gap-2">
                        <Activity className="size-4 text-tertiary" />
                        <h3 className="text-sm font-semibold text-foreground">Gossip Feed</h3>
                      </div>
                      <Badge variant="signal">Live</Badge>
                    </div>
                    <div className="space-y-2">
                      {recentTransactions.length > 0 ? (
                        recentTransactions.map((tx) => (
                          <div
                            key={tx.tx_hash}
                            className="flex flex-col gap-2 rounded-xl border border-border bg-card p-4"
                          >
                            <div className="flex items-center justify-between">
                              <Badge variant="glass" className="text-[10px]">
                                {tx.tx_type}
                              </Badge>
                              <span className="text-xs text-muted-foreground">
                                Pending
                              </span>
                            </div>
                            <div className="font-mono text-[10px] text-muted-foreground break-all">
                              {tx.tx_hash}
                            </div>
                          </div>
                        ))
                      ) : (
                        <EmptyPane
                          title="No transactions"
                          description="No pending transactions in gossip pool."
                          icon={Activity}
                        />
                      )}
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Watchlist & Wallet */}
            <div className="grid gap-8 lg:grid-cols-2">
              <Card variant="glass">
                <CardHeader className="border-b border-border">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Plus className="size-5 text-primary" />
                      <CardTitle className="text-foreground">Watchlist</CardTitle>
                    </div>
                    <Badge variant="glass">{watchlist.length}/12</Badge>
                  </div>
                </CardHeader>
                <CardContent className="p-6 space-y-4">
                  <form onSubmit={handleAddWatchItem} className="space-y-3">
                    <Input
                      placeholder="Address..."
                      className="h-10 text-sm"
                      value={watchAddress}
                      onChange={(e) => setWatchAddress(e.target.value)}
                    />
                    <div className="flex gap-2">
                      <Input
                        placeholder="Label (optional)..."
                        className="h-10 text-sm flex-1"
                        value={watchLabel}
                        onChange={(e) => setWatchLabel(e.target.value)}
                      />
                      <Button type="submit" size="sm" className="h-10 px-4">
                        Add
                      </Button>
                    </div>
                  </form>

                  <div className="space-y-2">
                    {watchlist.length > 0 ? (
                      watchlist.map((entry) => (
                        <div
                          key={entry.address}
                          className={cn(
                            "group flex items-center justify-between rounded-xl border border-border bg-card p-4 transition-all hover:bg-surface-hover",
                            walletView.selectedAddress === entry.address &&
                              "border-primary/50 bg-primary/5"
                          )}
                        >
                          <button
                            onClick={() =>
                              void handleInspectWatchAddress(entry.address)
                            }
                            className="flex-1 min-w-0 pr-4 text-left"
                          >
                            <div className="text-sm font-medium text-foreground truncate">
                              {entry.label}
                            </div>
                            <div className="font-mono text-[10px] text-muted-foreground truncate">
                              {entry.address}
                            </div>
                          </button>
                          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => copyToClipboard(entry.address)}
                              className="size-7 text-muted-foreground hover:text-primary"
                            >
                              <Copy className="size-3.5" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() =>
                                handleRemoveWatchAddress(entry.address)
                              }
                              className="size-7 text-muted-foreground hover:text-destructive"
                            >
                              <Trash2 className="size-3.5" />
                            </Button>
                          </div>
                        </div>
                      ))
                    ) : (
                      <div className="flex flex-col items-center justify-center py-8 text-center space-y-2 border-2 border-dashed border-border rounded-xl">
                        <Plus className="size-5 text-muted-foreground" />
                        <p className="text-xs text-muted-foreground">
                          Add addresses for quick access
                        </p>
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>

              <WalletInspectorCard
                account={walletView.account}
                staking={walletView.staking}
                selectedAddress={walletView.selectedAddress}
              />
            </div>
          </div>

          {/* Right Column */}
          <div className="space-y-8">
            <div className="sticky top-24 space-y-8">
              {/* Detail Inspector */}
              {selectedBlock ? (
                <BlockDetailCard block={selectedBlock} />
              ) : selectedProposal ? (
                <ProposalDetailCard proposal={selectedProposal} />
              ) : (
                <Card variant="glass">
                  <CardContent className="flex flex-col items-center justify-center py-16 text-center gap-4">
                    <div className="flex size-16 items-center justify-center rounded-2xl border-2 border-dashed border-border bg-card/50 text-muted-foreground">
                      <Gauge className="size-8" />
                    </div>
                    <div className="space-y-2">
                      <h3 className="text-lg font-semibold text-foreground">
                        Inspector
                      </h3>
                      <p className="text-sm text-muted-foreground max-w-[200px]">
                        Select a block or proposal to view details.
                      </p>
                    </div>
                  </CardContent>
                </Card>
              )}

              {/* Governance */}
              <Card variant="glass">
                <CardHeader className="border-b border-border">
                  <div className="flex items-center gap-2">
                    <Vote className="size-5 text-accent" />
                    <CardTitle className="text-foreground">Governance</CardTitle>
                  </div>
                </CardHeader>
                <CardContent className="p-6 space-y-3">
                  {proposals.length > 0 ? (
                    proposals.map((proposal) => (
                      <button
                        key={proposal.id}
                        onClick={() => void lookupProposal(proposal.id)}
                        className={cn(
                          "group flex w-full items-center justify-between rounded-xl border border-border bg-card p-4 text-left transition-all hover:bg-surface-hover hover:border-accent/30",
                          selectedProposal?.id === proposal.id &&
                            "border-accent/50 bg-accent/5"
                        )}
                      >
                        <div className="space-y-1.5">
                          <div className="flex items-center gap-2">
                            <Badge
                              variant="glass"
                              className={cn(
                                "text-[10px]",
                                statusTone(proposal.status)
                              )}
                            >
                              {proposal.status}
                            </Badge>
                            <span className="font-mono text-[10px] text-muted-foreground">
                              #{proposal.id}
                            </span>
                          </div>
                          <div className="text-sm font-medium text-foreground group-hover:text-accent transition-colors">
                            {proposal.title}
                          </div>
                        </div>
                        <ChevronRight className="size-4 text-muted-foreground group-hover:text-accent group-hover:translate-x-0.5 transition-all" />
                      </button>
                    ))
                  ) : (
                    <EmptyPane
                      title="No proposals"
                      description="No active governance proposals."
                      icon={Vote}
                    />
                  )}
                </CardContent>
              </Card>

              {/* Transaction Constructor */}
              <Card variant="glass">
                <CardHeader className="border-b border-border">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Send className="size-5 text-tertiary" />
                      <CardTitle className="text-foreground">Constructor</CardTitle>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={handleCopyCommand}
                      className="size-8 text-muted-foreground hover:text-primary"
                    >
                      {copiedCommand ? (
                        <Check className="size-4 text-tertiary" />
                      ) : (
                        <Copy className="size-4" />
                      )}
                    </Button>
                  </div>
                </CardHeader>
                <CardContent className="p-6 space-y-6">
                  <div className="space-y-4">
                    <div className="space-y-1.5">
                      <label className="text-xs font-medium text-muted-foreground">
                        Source Wallet
                      </label>
                      <Input
                        placeholder="name or address..."
                        className="h-11"
                        value={draftFrom}
                        onChange={(e) => setDraftFrom(e.target.value)}
                      />
                    </div>
                    <div className="space-y-1.5">
                      <label className="text-xs font-medium text-muted-foreground">
                        Destination
                      </label>
                      <Input
                        placeholder="netchain1..."
                        className="h-11"
                        value={draftTo}
                        onChange={(e) => setDraftTo(e.target.value)}
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-1.5">
                        <label className="text-xs font-medium text-muted-foreground">
                          Amount
                        </label>
                        <Input
                          placeholder="0.0"
                          className="h-11"
                          value={draftAmount}
                          onChange={(e) => setDraftAmount(e.target.value)}
                        />
                      </div>
                      <div className="space-y-1.5">
                        <label className="text-xs font-medium text-muted-foreground">
                          Fee
                        </label>
                        <Input
                          placeholder="1"
                          className="h-11"
                          value={draftFee}
                          onChange={(e) => setDraftFee(e.target.value)}
                        />
                      </div>
                    </div>
                  </div>

                  <div className="rounded-xl border border-border bg-code-bg p-4 space-y-3">
                    <div className="flex items-center justify-between">
                      <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
                        CLI Preview
                      </span>
                      <span className="size-1.5 rounded-full bg-tertiary animate-pulse" />
                    </div>
                    <pre className="text-[11px] font-mono text-foreground/70 leading-relaxed overflow-x-auto whitespace-pre-wrap break-all max-h-32">
                      <code>{walletCommand}</code>
                    </pre>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </section>

      {/* Settings Section */}
      <section className="container-wide pb-24">
        <Card variant="glass">
          <CardHeader className="border-b border-border">
            <div className="flex flex-col gap-6 lg:flex-row lg:items-center lg:justify-between">
              <div className="flex items-center gap-3">
                <div className="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
                  <Settings className="size-5" />
                </div>
                <div>
                  <CardTitle className="text-foreground">Endpoint Configuration</CardTitle>
                  <CardDescription className="text-muted-foreground">
                    Customize transport layer parameters
                  </CardDescription>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <Button
                  onClick={handleApplyEndpoints}
                  disabled={!endpointChanges}
                  className="h-10"
                >
                  Apply Changes
                </Button>
                <Button
                  variant="outline"
                  onClick={handleResetEndpoints}
                  className="h-10"
                >
                  Reset Defaults
                </Button>
              </div>
            </div>
          </CardHeader>
          <CardContent className="p-6">
            <div className="grid gap-6 md:grid-cols-3">
              <div className="space-y-2">
                <label className="text-xs font-medium text-muted-foreground">
                  RPC Endpoint
                </label>
                <Input
                  value={endpointDraft.rpcUrl}
                  onChange={(e) =>
                    setEndpointDraft({ ...endpointDraft, rpcUrl: e.target.value })
                  }
                  className="h-11"
                  placeholder="http://..."
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-medium text-muted-foreground">
                  Monitoring API
                </label>
                <Input
                  value={endpointDraft.monitoringUrl}
                  onChange={(e) =>
                    setEndpointDraft({
                      ...endpointDraft,
                      monitoringUrl: e.target.value,
                    })
                  }
                  className="h-11"
                  placeholder="http://..."
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-medium text-muted-foreground">
                  WebSocket Gateway
                </label>
                <Input
                  value={endpointDraft.wsUrl}
                  onChange={(e) =>
                    setEndpointDraft({ ...endpointDraft, wsUrl: e.target.value })
                  }
                  className="h-11"
                  placeholder="ws://..."
                />
              </div>
            </div>
          </CardContent>
        </Card>
      </section>
    </div>
  );
}
