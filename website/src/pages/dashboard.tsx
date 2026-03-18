import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import {
  Activity,
  AlertTriangle,
  ArrowDown,
  ArrowUp,
  Blocks,
  Check,
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
  Server,
  Shield,
  Sparkles,
  Trash2,
  Vote,
  Wallet,
  Wifi,
  ChevronRight,
} from "lucide-react";

import { Footer } from "@/components/sections/footer";
import { Navbar } from "@/components/sections/navbar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  DEFAULT_MONITORING_URL,
  DEFAULT_RPC_URL,
} from "@/lib/netchain-client";
import { useNetChain } from "@/lib/use-netchain";
import type {
  AccountInfo,
  BlockDetails,
  BlockSummary,
  ChainParams,
  HealthSnapshot,
  MetricsSnapshot,
  NetChainEndpoints,
  ProposalInfo,
  SignedTransaction,
  StakingInfo,
  TransactionFeedItem,
  TransactionType,
  ValidatorSlashedEvent,
  WalletWatchItem,
} from "@/lib/types";
import { cn } from "@/lib/utils";

const DEFAULT_ENDPOINTS: NetChainEndpoints = {
  rpcUrl: DEFAULT_RPC_URL,
  monitoringUrl: DEFAULT_MONITORING_URL,
  wsUrl: "ws://127.0.0.1:8546",
};

const WATCHLIST_STORAGE_KEY = "netchain-dashboard-watchlist";
const FIELD_CLASS =
  "flex h-10 w-full rounded-xl border border-border/70 bg-background/80 px-3 text-sm text-foreground shadow-sm outline-none transition placeholder:text-muted-foreground focus:border-ring focus:ring-2 focus:ring-ring/30";
const TEXTAREA_CLASS = cn(FIELD_CLASS, "min-h-24 py-2");

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

function formatPercent(value: number | null | undefined) {
  if (value == null || Number.isNaN(value)) {
    return "—";
  }

  return `${formatCompactNumber(value * 100)}%`;
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
    return "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
  }

  if (
    normalized.includes("error") ||
    normalized.includes("failed") ||
    normalized.includes("rejected") ||
    normalized.includes("offline") ||
    normalized.includes("expired")
  ) {
    return "bg-rose-500/10 text-rose-600 dark:text-rose-300";
  }

  return "bg-sky-500/10 text-sky-600 dark:text-sky-300";
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

type MetricCardProps = {
  label: string;
  value: string;
  detail: string;
  icon: LucideIcon;
  toneClassName: string;
};

function MetricCard({
  label,
  value,
  detail,
  icon: Icon,
  toneClassName,
}: MetricCardProps) {
  return (
    <div className="rounded-2xl border border-border/70 bg-background/75 p-4 shadow-sm backdrop-blur-sm">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-[11px] uppercase tracking-[0.24em] text-muted-foreground">
            {label}
          </p>
          <div className="mt-2 text-2xl font-semibold tracking-tight text-foreground">
            {value}
          </div>
        </div>
        <div
          className={cn(
            "flex size-10 items-center justify-center rounded-xl border border-border/70",
            toneClassName
          )}
        >
          <Icon className="size-4" />
        </div>
      </div>
      <p className="mt-3 text-sm text-muted-foreground">{detail}</p>
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
    <div className="rounded-xl border border-border/60 bg-background/60 px-3 py-2">
      <div className="text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
        {label}
      </div>
      <div
        className={cn(
          "mt-1 text-sm text-foreground",
          mono && "font-mono text-xs break-all"
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
    <div className="flex flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-border/80 bg-muted/20 px-5 py-10 text-center">
      <div className="flex size-11 items-center justify-center rounded-2xl bg-primary/10 text-primary">
        <Icon className="size-5" />
      </div>
      <div>
        <p className="text-sm font-medium text-foreground">{title}</p>
        <p className="mt-1 text-sm text-muted-foreground">{description}</p>
      </div>
    </div>
  );
}

function BlockDetailCard({
  block,
}: {
  block: BlockDetails | null;
}) {
  return (
    <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
      <CardHeader className="border-b border-border/60">
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Layers3 className="size-4 text-primary" />
              Block Inspector
            </CardTitle>
            <CardDescription>
              Signed payloads, validator, and commitment data for the selected
              height.
            </CardDescription>
          </div>
          {block && (
            <Badge variant="secondary" className="gap-1.5">
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
            description="Load a block from the search bar or click any recent block to inspect its signed transactions."
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
                      size="icon-xs"
                      className="size-6"
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

            <div className="rounded-2xl border border-border/60 bg-background/60">
              <div className="flex items-center justify-between border-b border-border/60 px-4 py-3">
                <div className="flex items-center gap-2">
                  <FileText className="size-4 text-muted-foreground" />
                  <span className="text-sm font-medium text-foreground">
                    Transaction payloads
                  </span>
                </div>
                <span className="text-xs text-muted-foreground">
                  Signatures and public keys are shown instead of a derived hash.
                </span>
              </div>
              <div className="max-h-[26rem] divide-y divide-border/60 overflow-auto">
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
        <div className="text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
          Transaction {index + 1}
        </div>
        <div className="mt-1 font-medium text-foreground">
          {describeTxType(transaction.tx.tx_type)}
        </div>
      </div>
      <div>
        <div className="text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
          Route
        </div>
        <div className="mt-1 font-mono text-xs text-foreground">
          {formatAddress(transaction.tx.sender)}{" "}
          <ChevronRight className="inline size-3 align-text-top text-muted-foreground" />{" "}
          {transaction.tx.receiver
            ? formatAddress(transaction.tx.receiver)
            : "Protocol"}
        </div>
      </div>
      <DetailRow
        label="Amount"
        value={formatCount(transaction.tx.amount)}
      />
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

function ProposalDetailCard({
  proposal,
}: {
  proposal: ProposalInfo | null;
}) {
  const { yes, no, total } = proposalProgress(proposal);
  const yesWidth = total > 0 ? (yes / total) * 100 : 0;
  const noWidth = total > 0 ? (no / total) * 100 : 0;

  return (
    <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
      <CardHeader className="border-b border-border/60">
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Vote className="size-4 text-primary" />
              Proposal Inspector
            </CardTitle>
            <CardDescription>
              Governance metadata, vote pressure, and expiry timing.
            </CardDescription>
          </div>
          {proposal && (
            <Badge
              variant="outline"
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
            description="Search by proposal id or click a proposal card to inspect its vote weight and expiry."
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
                size="icon-sm"
                onClick={() => copyToClipboard(String(proposal.id))}
                aria-label="Copy proposal id"
              >
                <Copy className="size-4" />
              </Button>
            </div>

            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
              <DetailRow
                label="Proposal id"
                value={`#${proposal.id}`}
                mono
              />
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

            <div className="space-y-2 rounded-2xl border border-border/60 bg-background/60 p-4">
              <div className="flex items-center justify-between text-xs uppercase tracking-[0.22em] text-muted-foreground">
                <span>Vote pressure</span>
                <span>{total} total votes</span>
              </div>
              <div className="flex h-2 overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full bg-emerald-500 transition-[width] duration-300"
                  style={{ width: `${yesWidth}%` }}
                />
                <div
                  className="h-full bg-rose-500 transition-[width] duration-300"
                  style={{ width: `${noWidth}%` }}
                />
              </div>
              <div className="flex items-center justify-between text-sm text-muted-foreground">
                <span className="text-emerald-500">
                  Yes: {formatCount(proposal.yes_votes)}
                </span>
                <span className="text-rose-500">
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
    <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
      <CardHeader className="border-b border-border/60">
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Wallet className="size-4 text-primary" />
              Wallet Inspector
            </CardTitle>
            <CardDescription>
              Watch-only balances, nonce tracking, and staking visibility.
            </CardDescription>
          </div>
          {selectedAddress && (
            <Badge variant="secondary" className="gap-1.5">
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
            description="Load an address from the watchlist or the explorer search bar to inspect balance and stake."
          />
        ) : (
          <>
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
          </>
        )}
      </CardContent>
    </Card>
  );
}

function TelemetryCard({
  health,
  metrics,
  chainParams,
  slashes,
}: {
  health: HealthSnapshot | null;
  metrics: MetricsSnapshot | null;
  chainParams: ChainParams | null;
  slashes: ValidatorSlashedEvent[];
}) {
  return (
    <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
      <CardHeader className="border-b border-border/60">
        <CardTitle className="flex items-center gap-2">
          <Activity className="size-4 text-primary" />
          Network Telemetry
        </CardTitle>
        <CardDescription>
          Health, hybrid trust metrics, chain parameters, and slashing events.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4 pt-4">
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <DetailRow
            label="Node status"
            value={
              <Badge
                variant="outline"
                className={cn("border-transparent", statusTone(health?.status))}
              >
                {health?.status ?? "Unknown"}
              </Badge>
            }
          />
          <DetailRow label="Uptime" value={formatDuration(health?.uptime_secs)} />
          <DetailRow
            label="Chain height"
            value={formatCount(health?.chain_height)}
            mono
          />
          <DetailRow
            label="Mempool"
            value={formatCount(health?.mempool_size)}
          />
          <DetailRow label="Peers" value={formatCount(health?.peer_count)} />
          <DetailRow
            label="Validators"
            value={formatCount(health?.validator_count)}
          />
          <DetailRow
            label="Consensus mode"
            value={
              <Badge variant="secondary" className="capitalize">
                {health?.consensus_mode ?? "unknown"}
              </Badge>
            }
          />
          <DetailRow
            label="Aggregator nodes"
            value={formatCount(health?.aggregator_nodes)}
          />
          <DetailRow
            label="Epoch"
            value={formatCount(health?.current_epoch)}
          />
          <DetailRow
            label="Latency"
            value={
              metrics?.netchain_latency_ms == null
                ? "—"
                : `${formatCompactNumber(metrics.netchain_latency_ms)} ms`
            }
          />
          <DetailRow
            label="Download"
            value={
              metrics?.netchain_download_mbps == null
                ? "—"
                : `${formatCompactNumber(metrics.netchain_download_mbps)} Mbps`
            }
          />
          <DetailRow
            label="Upload"
            value={
              metrics?.netchain_upload_mbps == null
                ? "—"
                : `${formatCompactNumber(metrics.netchain_upload_mbps)} Mbps`
            }
          />
          <DetailRow
            label="Uptime score"
            value={
              metrics?.netchain_uptime_percent == null
                ? "—"
                : `${formatCompactNumber(metrics.netchain_uptime_percent)}%`
            }
          />
        </div>

          <div className="rounded-2xl border border-border/60 bg-background/60 p-4">
            <div className="flex items-center gap-2">
              <Shield className="size-4 text-muted-foreground" />
              <span className="text-sm font-medium text-foreground">
                Hybrid trust snapshot
              </span>
            </div>
            <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-3">
            <DetailRow
              label="Verified validators"
              value={formatCount(health?.verified_validator_count)}
            />
            <DetailRow
              label="Unverified validators"
              value={formatCount(health?.unverified_validator_count)}
            />
            <DetailRow
              label="Slashed validators"
              value={formatCount(health?.slashed_validator_count)}
            />
            <DetailRow
              label="Average reputation"
              value={formatPercent(health?.average_reputation)}
            />
            <DetailRow
              label="Average identity"
              value={formatPercent(health?.average_identity_score)}
            />
            <DetailRow
              label="Quorum coverage"
              value={`${formatCount(health?.verified_validator_count)} verified / ${formatCount(
                health?.unverified_validator_count
              )} pending`}
            />
          </div>
        </div>

        <div className="rounded-2xl border border-border/60 bg-background/60 p-4">
          <div className="flex items-center gap-2">
            <Gauge className="size-4 text-muted-foreground" />
            <span className="text-sm font-medium text-foreground">
              Chain parameters
            </span>
          </div>
          {chainParams ? (
            <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-3">
              <DetailRow
                label="Block reward"
                value={formatCount(chainParams.block_reward)}
              />
              <DetailRow
                label="Block interval"
                value={`${formatCount(chainParams.block_interval_secs)}s`}
              />
              <DetailRow
                label="Max txs per block"
                value={formatCount(chainParams.max_txs_per_block)}
              />
              <DetailRow
                label="Stake weight"
                value={`${formatCompactNumber(chainParams.stake_weight * 100)}%`}
              />
              <DetailRow
                label="Proposal quorum"
                value={`${formatCompactNumber(
                  chainParams.proposal_quorum_bps / 100
                )}%`}
              />
              <DetailRow
                label="Approval threshold"
                value={`${formatCompactNumber(
                  chainParams.proposal_approval_bps / 100
                )}%`}
              />
              <DetailRow
                label="Min proposal stake"
                value={formatCount(chainParams.min_proposal_stake)}
              />
            </div>
          ) : (
            <div className="mt-3 text-sm text-muted-foreground">
              Chain parameters are unavailable until the RPC snapshot completes.
            </div>
          )}
        </div>

        <div className="rounded-2xl border border-border/60 bg-background/60 p-4">
          <div className="flex items-center gap-2">
            <Shield className="size-4 text-muted-foreground" />
            <span className="text-sm font-medium text-foreground">
              Recent slashing events
            </span>
          </div>
          {slashes.length === 0 ? (
            <div className="mt-3 text-sm text-muted-foreground">
              No validator slash events have been emitted yet.
            </div>
          ) : (
            <div className="mt-3 space-y-3">
              {slashes.map((slash, index) => (
                <div
                  key={`${slash.validator}-${index}`}
                  className="rounded-xl border border-border/60 bg-background/60 px-3 py-3"
                >
                  <div className="flex items-center justify-between gap-3">
                    <div>
                      <p className="text-sm font-medium text-foreground">
                        {formatAddress(slash.validator)}
                      </p>
                      <p className="text-sm text-muted-foreground">
                        {slash.reason}
                      </p>
                    </div>
                    <Badge variant="destructive" className="gap-1.5">
                      -{formatCount(slash.amount_burned)}
                    </Badge>
                  </div>
                  <p className="mt-2 text-xs text-muted-foreground">
                    Remaining stake: {formatCount(slash.remaining_stake)}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>
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
    <div className="rounded-2xl border border-amber-500/25 bg-amber-500/10 px-4 py-3 text-sm text-amber-900 dark:text-amber-100">
      <div className="flex items-start gap-3">
        <div className="mt-0.5 flex size-8 items-center justify-center rounded-xl bg-amber-500/15 text-amber-600 dark:text-amber-200">
          <Icon className="size-4" />
        </div>
        <div>
          <p className="font-medium">{title}</p>
          <p className="mt-1 text-sm text-amber-900/80 dark:text-amber-100/80">
            {message}
          </p>
        </div>
      </div>
    </div>
  );
}

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
  } = useNetChain();

  const [endpointDraft, setEndpointDraft] = useState(settings);
  const [watchlist, setWatchlist] = useState<WalletWatchItem[]>(
    readStoredWatchlist
  );
  const [watchLabel, setWatchLabel] = useState("");
  const [watchAddress, setWatchAddress] = useState("");
  const [blockQuery, setBlockQuery] = useState("");
  const [proposalQuery, setProposalQuery] = useState("");
  const [accountQuery, setAccountQuery] = useState("");
  const [draftFrom, setDraftFrom] = useState("");
  const [draftTo, setDraftTo] = useState("");
  const [draftAmount, setDraftAmount] = useState("");
  const [draftFee, setDraftFee] = useState("1");
  const [draftMemo, setDraftMemo] = useState("");
  const [blockLoading, setBlockLoading] = useState(false);
  const [proposalLoading, setProposalLoading] = useState(false);
  const [accountLoading, setAccountLoading] = useState(false);

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

  const latestBlock = recentBlocks[0] ?? null;
  const latestTx = recentTransactions[0] ?? null;
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

  async function handleLoadProposal(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const proposalId = Number(proposalQuery.trim());
    if (!Number.isFinite(proposalId)) {
      return;
    }

    setProposalLoading(true);
    try {
      await lookupProposal(proposalId);
    } finally {
      setProposalLoading(false);
    }
  }

  async function handleLoadAccount(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedAddress = accountQuery.trim();
    if (!normalizedAddress) {
      return;
    }

    setAccountLoading(true);
    try {
      await lookupAccount(normalizedAddress);
      if (!draftFrom) {
        setDraftFrom(normalizedAddress);
      }
    } finally {
      setAccountLoading(false);
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
    setWatchlist((current) => current.filter((item) => item.address !== address));
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

  return (
    <div className="relative min-h-dvh overflow-hidden">
      <div
        className="pointer-events-none absolute inset-0 -z-10"
        aria-hidden="true"
      >
        <div className="absolute left-[-6rem] top-16 size-[26rem] rounded-full bg-primary/15 blur-3xl" />
        <div className="absolute right-[-8rem] top-56 size-[28rem] rounded-full bg-accent/15 blur-3xl" />
        <div className="absolute inset-x-0 top-0 h-px bg-border/60" />
      </div>

      <Navbar />

      <main className="mx-auto flex w-full max-w-7xl flex-col gap-8 px-4 pb-16 pt-24 sm:px-6 lg:px-8">
        <section className="grid gap-6 lg:grid-cols-[minmax(0,1.45fr)_minmax(320px,0.9fr)]">
          <Card className="border-primary/20 bg-gradient-to-br from-primary/12 via-card to-accent/10 shadow-xl shadow-primary/5">
            <CardHeader className="space-y-4 border-b border-border/60">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="secondary" className="gap-1.5">
                  <Radar className="size-3.5" />
                  control room
                </Badge>
                <Badge
                  variant="outline"
                  className={cn(
                    "border-transparent",
                    isConnected
                      ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                      : "bg-rose-500/10 text-rose-700 dark:text-rose-300"
                  )}
                >
                  {isConnected ? "WS connected" : "WS offline"}
                </Badge>
                <Badge
                  variant="outline"
                  className={cn(
                    "border-transparent",
                    snapshotStatus === "ready"
                      ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                      : snapshotStatus === "degraded"
                        ? "bg-amber-500/10 text-amber-700 dark:text-amber-300"
                        : snapshotStatus === "loading"
                          ? "bg-sky-500/10 text-sky-700 dark:text-sky-300"
                          : "bg-rose-500/10 text-rose-700 dark:text-rose-300"
                  )}
                >
                  {snapshotStatus}
                </Badge>
                <Badge variant="outline" className="gap-1.5">
                  <Clock3 className="size-3.5" />
                  {lastSyncedAt ? formatDateTime(lastSyncedAt) : "not synced"}
                </Badge>
                <Badge variant="outline" className="gap-1.5 capitalize">
                  <Shield className="size-3.5" />
                  {health?.consensus_mode ?? "unknown"} consensus
                </Badge>
              </div>

              <div className="space-y-3">
                <div className="flex items-center gap-3">
                  <div className="flex size-12 items-center justify-center rounded-2xl border border-border/70 bg-background/75 text-primary shadow-sm">
                    <Sparkles className="size-5" />
                  </div>
                  <div>
                    <CardTitle className="text-3xl tracking-tight sm:text-4xl">
                      NetChain Explorer and Hybrid Wallet
                    </CardTitle>
                    <CardDescription className="mt-1 max-w-2xl text-base text-muted-foreground">
                      Live chain visibility, hybrid trust telemetry,
                      governance inspection, and a watch-only wallet workflow
                      in one browser workspace.
                    </CardDescription>
                  </div>
                </div>

                <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
                  <MetricCard
                    label="Chain height"
                    value={formatCount(latestHeight)}
                    detail="Latest committed height from RPC and monitoring."
                    icon={Blocks}
                    toneClassName="bg-primary/10 text-primary"
                  />
                  <MetricCard
                    label="Peers"
                    value={formatCount(health?.peer_count)}
                    detail="Connected peers reported by the monitoring server."
                    icon={Wifi}
                    toneClassName="bg-sky-500/10 text-sky-500"
                  />
                  <MetricCard
                    label="Mempool"
                    value={formatCount(health?.mempool_size)}
                    detail="Pending transactions waiting to land in a block."
                    icon={Gauge}
                    toneClassName="bg-amber-500/10 text-amber-500"
                  />
                  <MetricCard
                    label="Validators"
                    value={formatCount(health?.validator_count)}
                    detail="Validators currently visible to the node."
                    icon={Server}
                    toneClassName="bg-emerald-500/10 text-emerald-500"
                  />
                  <MetricCard
                    label="Consensus mode"
                    value={health?.consensus_mode ?? "—"}
                    detail="PoI, stake, identity, reputation, slashing, and attestations."
                    icon={Shield}
                    toneClassName="bg-slate-500/10 text-slate-500"
                  />
                </div>

                <div className="grid gap-3 md:grid-cols-2">
                  <div className="rounded-2xl border border-border/70 bg-background/75 p-4 shadow-sm backdrop-blur-sm">
                    <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                      <Blocks className="size-4 text-primary" />
                      Latest block
                    </div>
                    <p className="mt-2 font-mono text-xs text-muted-foreground">
                      {latestBlock
                        ? `#${latestBlock.index} · ${formatHash(latestBlock.hash)}`
                        : "Waiting for the next block"}
                    </p>
                  </div>
                  <div className="rounded-2xl border border-border/70 bg-background/75 p-4 shadow-sm backdrop-blur-sm">
                    <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                      <Activity className="size-4 text-primary" />
                      Latest transaction
                    </div>
                    <p className="mt-2 font-mono text-xs text-muted-foreground">
                      {latestTx
                        ? `${formatHash(latestTx.tx_hash)} · ${latestTx.tx_type}`
                        : "No live transaction yet"}
                    </p>
                  </div>
                </div>

                <div className="grid gap-3 md:grid-cols-2">
                  <div className="rounded-2xl border border-border/70 bg-background/75 p-4 shadow-sm backdrop-blur-sm">
                    <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                      <ArrowDown className="size-4 text-sky-500" />
                      Download
                    </div>
                    <p className="mt-2 text-2xl font-semibold tracking-tight text-foreground">
                      {metrics?.netchain_download_mbps == null
                        ? "—"
                        : `${formatCompactNumber(
                            metrics.netchain_download_mbps
                          )} Mbps`}
                    </p>
                  </div>
                  <div className="rounded-2xl border border-border/70 bg-background/75 p-4 shadow-sm backdrop-blur-sm">
                    <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                      <ArrowUp className="size-4 text-emerald-500" />
                      Upload
                    </div>
                    <p className="mt-2 text-2xl font-semibold tracking-tight text-foreground">
                      {metrics?.netchain_upload_mbps == null
                        ? "—"
                        : `${formatCompactNumber(
                            metrics.netchain_upload_mbps
                          )} Mbps`}
                    </p>
                  </div>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4 pt-4">
              {rpcError && (
                <AppShellError
                  icon={AlertTriangle}
                  title="RPC error"
                  message={rpcError}
                />
              )}
              {monitoringError && (
                <AppShellError
                  icon={AlertTriangle}
                  title="Monitoring error"
                  message={monitoringError}
                />
              )}
              {wsError && (
                <AppShellError
                  icon={AlertTriangle}
                  title="WebSocket error"
                  message={wsError}
                />
              )}

              <div className="grid gap-3 md:grid-cols-4">
                <DetailRow
                  label="Uptime"
                  value={formatDuration(health?.uptime_secs)}
                />
                <DetailRow
                  label="Latency"
                  value={
                    metrics?.netchain_latency_ms == null
                      ? "—"
                      : `${formatCompactNumber(metrics.netchain_latency_ms)} ms`
                  }
                />
                <DetailRow
                  label="Current epoch"
                  value={formatCount(health?.current_epoch)}
                />
                <DetailRow
                  label="Active proposals"
                  value={formatCount(activeProposalCount)}
                />
              </div>

              <div className="flex flex-wrap items-center gap-3">
                <Button
                  onClick={() => void handleRefresh()}
                  disabled={snapshotStatus === "loading"}
                  className="gap-2"
                >
                  <RefreshCw
                    className={cn(
                      "size-4",
                      snapshotStatus === "loading" && "animate-spin"
                    )}
                  />
                  Refresh snapshot
                </Button>
                <Button
                  variant="outline"
                  onClick={clearSelections}
                  className="gap-2"
                >
                  Clear selections
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
            <CardHeader className="border-b border-border/60">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    <Globe className="size-4 text-primary" />
                    Endpoint controls
                  </CardTitle>
                  <CardDescription>
                    Update the RPC, monitoring, and WebSocket endpoints used by
                    this browser session.
                  </CardDescription>
                </div>
                <Badge variant="outline" className="gap-1.5">
                  <Shield className="size-3.5" />
                  local config
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="space-y-4 pt-4">
              <form className="space-y-4" onSubmit={(event) => {
                event.preventDefault();
                handleApplyEndpoints();
              }}>
                <div>
                  <label className="mb-1.5 block text-sm font-medium text-foreground">
                    RPC base URL
                  </label>
                  <input
                    className={FIELD_CLASS}
                    value={endpointDraft.rpcUrl}
                    onChange={(event) =>
                      setEndpointDraft((current) => ({
                        ...current,
                        rpcUrl: event.target.value,
                      }))
                    }
                    placeholder="http://127.0.0.1:8545"
                  />
                  <p className="mt-1 text-xs text-muted-foreground">
                    The client appends <code>/rpc</code> automatically.
                  </p>
                </div>
                <div>
                  <label className="mb-1.5 block text-sm font-medium text-foreground">
                    Monitoring base URL
                  </label>
                  <input
                    className={FIELD_CLASS}
                    value={endpointDraft.monitoringUrl}
                    onChange={(event) =>
                      setEndpointDraft((current) => ({
                        ...current,
                        monitoringUrl: event.target.value,
                      }))
                    }
                    placeholder="http://127.0.0.1:9090"
                  />
                </div>
                <div>
                  <label className="mb-1.5 block text-sm font-medium text-foreground">
                    WebSocket URL
                  </label>
                  <input
                    className={FIELD_CLASS}
                    value={endpointDraft.wsUrl}
                    onChange={(event) =>
                      setEndpointDraft((current) => ({
                        ...current,
                        wsUrl: event.target.value,
                      }))
                    }
                    placeholder="ws://127.0.0.1:8546"
                  />
                </div>

                <div className="flex flex-wrap gap-2 pt-2">
                  <Button type="submit" className="gap-2" disabled={!endpointChanges}>
                    <Check className="size-4" />
                    Apply endpoints
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleResetEndpoints}
                    className="gap-2"
                  >
                    Reset defaults
                  </Button>
                </div>
              </form>

              <div className="grid gap-3 rounded-2xl border border-border/60 bg-muted/20 p-4">
                <DetailRow label="RPC" value={settings.rpcUrl} mono />
                <DetailRow label="Monitoring" value={settings.monitoringUrl} mono />
                <DetailRow label="WebSocket" value={settings.wsUrl} mono />
              </div>
            </CardContent>
          </Card>
        </section>

        <section className="grid gap-6 xl:grid-cols-[minmax(0,1.5fr)_minmax(320px,0.85fr)]">
          <div className="space-y-6">
            <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
              <CardHeader className="border-b border-border/60">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <CardTitle className="flex items-center gap-2">
                      <Search className="size-4 text-primary" />
                      Explorer
                    </CardTitle>
                    <CardDescription>
                      Search blocks, proposals, and addresses directly against
                      the live node.
                    </CardDescription>
                  </div>
                  <Badge variant="secondary" className="gap-1.5">
                    <Check className="size-3.5" />
                    RPC live
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-6 pt-4">
                <div className="grid gap-4 lg:grid-cols-3">
                  <form
                    className="space-y-3 rounded-2xl border border-border/60 bg-background/60 p-4"
                    onSubmit={handleLoadBlock}
                  >
                    <div className="flex items-center gap-2">
                      <Blocks className="size-4 text-primary" />
                      <h3 className="text-sm font-medium text-foreground">
                        Block
                      </h3>
                    </div>
                    <input
                      className={FIELD_CLASS}
                      value={blockQuery}
                      onChange={(event) => setBlockQuery(event.target.value)}
                      placeholder={latestHeight == null ? "latest" : String(latestHeight)}
                      inputMode="numeric"
                    />
                    <Button
                      type="submit"
                      className="w-full gap-2"
                      disabled={
                        blockLoading ||
                        (latestHeight == null && !blockQuery.trim())
                      }
                    >
                      {blockLoading ? (
                        <Loader2 className="size-4 animate-spin" />
                      ) : (
                        <Search className="size-4" />
                      )}
                      Load block
                    </Button>
                  </form>

                  <form
                    className="space-y-3 rounded-2xl border border-border/60 bg-background/60 p-4"
                    onSubmit={handleLoadProposal}
                  >
                    <div className="flex items-center gap-2">
                      <Vote className="size-4 text-primary" />
                      <h3 className="text-sm font-medium text-foreground">
                        Proposal
                      </h3>
                    </div>
                    <input
                      className={FIELD_CLASS}
                      value={proposalQuery}
                      onChange={(event) => setProposalQuery(event.target.value)}
                      placeholder="proposal id"
                      inputMode="numeric"
                    />
                    <Button
                      type="submit"
                      className="w-full gap-2"
                      disabled={proposalLoading || !proposalQuery.trim()}
                    >
                      {proposalLoading ? (
                        <Loader2 className="size-4 animate-spin" />
                      ) : (
                        <Search className="size-4" />
                      )}
                      Load proposal
                    </Button>
                  </form>

                  <form
                    className="space-y-3 rounded-2xl border border-border/60 bg-background/60 p-4"
                    onSubmit={handleLoadAccount}
                  >
                    <div className="flex items-center gap-2">
                      <Wallet className="size-4 text-primary" />
                      <h3 className="text-sm font-medium text-foreground">
                        Address
                      </h3>
                    </div>
                    <input
                      className={FIELD_CLASS}
                      value={accountQuery}
                      onChange={(event) => setAccountQuery(event.target.value)}
                      placeholder="wallet name or address"
                    />
                    <Button
                      type="submit"
                      className="w-full gap-2"
                      disabled={accountLoading || !accountQuery.trim()}
                    >
                      {accountLoading ? (
                        <Loader2 className="size-4 animate-spin" />
                      ) : (
                        <Search className="size-4" />
                      )}
                      Load account
                    </Button>
                  </form>
                </div>

                <div className="grid gap-6 xl:grid-cols-2">
                  <Card className="border-border/60 bg-background/55">
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between gap-3">
                        <div>
                          <CardTitle className="flex items-center gap-2 text-base">
                            <Blocks className="size-4 text-primary" />
                            Recent blocks
                          </CardTitle>
                          <CardDescription>
                            Newest heights first. Click a row to inspect the full
                            block payload.
                          </CardDescription>
                        </div>
                        <Badge variant="outline" className="gap-1.5">
                          {formatCount(recentBlocks.length)} live
                        </Badge>
                      </div>
                    </CardHeader>
                    <CardContent className="pt-0">
                      <div className="overflow-hidden rounded-2xl border border-border/60">
                        <div className="grid grid-cols-[auto_1.2fr_auto] gap-3 border-b border-border/60 bg-muted/40 px-4 py-3 text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                          <span>Height</span>
                          <span>Hash</span>
                          <span className="text-right">Txs</span>
                        </div>
                        <div className="max-h-[24rem] divide-y divide-border/60 overflow-auto">
                          {recentBlocks.length === 0 ? (
                            <div className="px-4 py-8">
                              <EmptyPane
                                icon={Blocks}
                                title="Waiting for blocks"
                                description="The node will stream block summaries here as soon as it syncs or produces new blocks."
                              />
                            </div>
                          ) : (
                            recentBlocks.map((block: BlockSummary) => (
                              <button
                                key={`${block.hash}-${block.index}`}
                                type="button"
                                onClick={() => void lookupBlock(block.index)}
                                className={cn(
                                  "grid w-full grid-cols-[auto_1.2fr_auto] gap-3 px-4 py-3 text-left text-sm transition-colors hover:bg-muted/50",
                                  selectedBlock?.index === block.index &&
                                    "bg-primary/5"
                                )}
                              >
                                <span className="font-mono text-primary">
                                  #{block.index}
                                </span>
                                <span
                                  className="truncate font-mono text-muted-foreground"
                                  title={block.hash}
                                >
                                  {formatHash(block.hash)}
                                </span>
                                <span className="text-right text-muted-foreground">
                                  {formatCount(block.tx_count)}
                                </span>
                              </button>
                            ))
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  <Card className="border-border/60 bg-background/55">
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between gap-3">
                        <div>
                          <CardTitle className="flex items-center gap-2 text-base">
                            <Activity className="size-4 text-primary" />
                            Live transaction feed
                          </CardTitle>
                          <CardDescription>
                            Real WS tx hashes with sender, receiver, and type.
                          </CardDescription>
                        </div>
                        <Badge variant="outline" className="gap-1.5">
                          {formatCount(recentTransactions.length)} live
                        </Badge>
                      </div>
                    </CardHeader>
                    <CardContent className="pt-0">
                      <div className="overflow-hidden rounded-2xl border border-border/60">
                        <div className="grid grid-cols-[1.2fr_1fr_auto] gap-3 border-b border-border/60 bg-muted/40 px-4 py-3 text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                          <span>Hash</span>
                          <span>Route</span>
                          <span className="text-right">Amount</span>
                        </div>
                        <div className="max-h-[24rem] divide-y divide-border/60 overflow-auto">
                          {recentTransactions.length === 0 ? (
                            <div className="px-4 py-8">
                              <EmptyPane
                                icon={Activity}
                                title="No live transaction events yet"
                                description="Once the node relays a signed transaction, its canonical hash will appear here."
                              />
                            </div>
                          ) : (
                            recentTransactions.map((tx: TransactionFeedItem) => (
                              <div
                                key={tx.tx_hash}
                                className="grid grid-cols-[1.2fr_1fr_auto] gap-3 px-4 py-3 text-sm"
                              >
                                <div className="min-w-0">
                                  <div className="font-mono text-muted-foreground">
                                    {formatHash(tx.tx_hash)}
                                  </div>
                                  <div className="mt-1 text-[11px] uppercase tracking-[0.22em] text-muted-foreground">
                                    {tx.tx_type}
                                  </div>
                                </div>
                                <div className="min-w-0 font-mono text-xs text-muted-foreground">
                                  {formatAddress(tx.sender)}{" "}
                                  <ChevronRight className="inline size-3 align-text-top" />{" "}
                                  {formatAddress(tx.receiver)}
                                </div>
                                <div className="text-right font-mono text-foreground">
                                  {formatCount(tx.amount)}
                                </div>
                              </div>
                            ))
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </div>

                <div className="grid gap-6 xl:grid-cols-2">
                  <Card className="border-border/60 bg-background/55">
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between gap-3">
                        <div>
                          <CardTitle className="flex items-center gap-2 text-base">
                            <Vote className="size-4 text-primary" />
                            Active proposals
                          </CardTitle>
                          <CardDescription>
                            Real-time governance status from the RPC snapshot.
                          </CardDescription>
                        </div>
                        <Badge variant="outline" className="gap-1.5">
                          {formatCount(activeProposalCount)} active
                        </Badge>
                      </div>
                    </CardHeader>
                    <CardContent className="pt-0">
                      <div className="space-y-3">
                        {proposals.length === 0 ? (
                          <EmptyPane
                            icon={Vote}
                            title="No proposals available"
                            description="The governance queue is empty or the snapshot has not completed yet."
                          />
                        ) : (
                          proposals.slice(0, 8).map((proposal) => (
                            <button
                              key={proposal.id}
                              type="button"
                              onClick={() => {
                                setProposalQuery(String(proposal.id));
                                void lookupProposal(proposal.id);
                              }}
                              className={cn(
                                "w-full rounded-2xl border border-border/60 bg-background/60 px-4 py-4 text-left transition-colors hover:bg-muted/50",
                                selectedProposal?.id === proposal.id &&
                                  "bg-primary/5"
                              )}
                            >
                              <div className="flex items-start justify-between gap-3">
                                <div>
                                  <div className="flex items-center gap-2">
                                    <span className="text-sm font-medium text-foreground">
                                      {proposal.title}
                                    </span>
                                    <Badge
                                      variant="outline"
                                      className={cn(
                                        "border-transparent",
                                        statusTone(proposal.status)
                                      )}
                                    >
                                      {proposal.status}
                                    </Badge>
                                  </div>
                                  <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
                                    {proposal.description || "No description provided."}
                                  </p>
                                </div>
                                <ChevronRight className="size-4 text-muted-foreground" />
                              </div>
                              <div className="mt-3 grid grid-cols-3 gap-2 text-xs text-muted-foreground">
                                <span>Yes {formatCount(proposal.yes_votes)}</span>
                                <span>No {formatCount(proposal.no_votes)}</span>
                                <span>ID #{proposal.id}</span>
                              </div>
                            </button>
                          ))
                        )}
                      </div>
                    </CardContent>
                  </Card>

                  <ProposalDetailCard proposal={selectedProposal} />
                </div>

                <BlockDetailCard block={selectedBlock} />
              </CardContent>
            </Card>
          </div>

          <div className="space-y-6">
            <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
              <CardHeader className="border-b border-border/60">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <CardTitle className="flex items-center gap-2">
                      <Wallet className="size-4 text-primary" />
                      Watchlist
                    </CardTitle>
                    <CardDescription>
                      Saved addresses for balance, nonce, and staking inspection.
                    </CardDescription>
                  </div>
                  <Badge variant="secondary" className="gap-1.5">
                    {formatCount(watchlist.length)} saved
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-4 pt-4">
                <form className="space-y-3" onSubmit={handleAddWatchItem}>
                  <div className="grid gap-3">
                    <div>
                      <label className="mb-1.5 block text-sm font-medium text-foreground">
                        Label
                      </label>
                      <input
                        className={FIELD_CLASS}
                        value={watchLabel}
                        onChange={(event) => setWatchLabel(event.target.value)}
                        placeholder="Treasury, validator, team wallet"
                      />
                    </div>
                    <div>
                      <label className="mb-1.5 block text-sm font-medium text-foreground">
                        Address
                      </label>
                      <input
                        className={FIELD_CLASS}
                        value={watchAddress}
                        onChange={(event) => setWatchAddress(event.target.value)}
                        placeholder="wallet address"
                      />
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button type="submit" className="gap-2">
                      <Plus className="size-4" />
                      Save address
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      onClick={() => {
                        setWatchLabel("");
                        setWatchAddress("");
                      }}
                    >
                      Clear
                    </Button>
                  </div>
                </form>

                <div className="space-y-2">
                  {watchlist.length === 0 ? (
                    <EmptyPane
                      icon={Wallet}
                      title="No saved addresses"
                      description="Add a wallet label and address to make account inspection one click away."
                    />
                  ) : (
                    watchlist.map((item) => (
                      <div
                        key={item.address}
                        className="rounded-2xl border border-border/60 bg-background/60 p-3"
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div>
                            <div className="text-sm font-medium text-foreground">
                              {item.label}
                            </div>
                            <div className="mt-1 font-mono text-xs text-muted-foreground">
                              {formatAddress(item.address)}
                            </div>
                          </div>
                          <div className="flex items-center gap-1.5">
                            <Button
                              variant="ghost"
                              size="icon-sm"
                              onClick={() => copyToClipboard(item.address)}
                              aria-label="Copy watched address"
                            >
                              <Copy className="size-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon-sm"
                              onClick={() => handleInspectWatchAddress(item.address)}
                              aria-label="Inspect watched address"
                            >
                              <Search className="size-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon-sm"
                              onClick={() => handleRemoveWatchAddress(item.address)}
                              aria-label="Remove watched address"
                            >
                              <Trash2 className="size-4" />
                            </Button>
                          </div>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </CardContent>
            </Card>

            <WalletInspectorCard
              account={walletView.account}
              staking={walletView.staking}
              selectedAddress={walletView.selectedAddress}
            />

            <Card className="border-border/70 bg-card/90 shadow-sm backdrop-blur-xl">
              <CardHeader className="border-b border-border/60">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <CardTitle className="flex items-center gap-2">
                      <Send className="size-4 text-primary" />
                      Wallet draft
                    </CardTitle>
                    <CardDescription>
                      The browser drafts the transaction, but signing still
                      happens in the CLI until a browser signer is added.
                    </CardDescription>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => copyToClipboard(walletCommand)}
                    className="gap-2"
                  >
                    <Copy className="size-4" />
                    Copy command
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="space-y-4 pt-4">
                <div className="grid gap-3">
                  <div className="grid gap-3 md:grid-cols-2">
                    <div>
                      <label className="mb-1.5 block text-sm font-medium text-foreground">
                        From
                      </label>
                      <input
                        className={FIELD_CLASS}
                        value={draftFrom}
                        onChange={(event) => setDraftFrom(event.target.value)}
                        placeholder="wallet name or address"
                      />
                    </div>
                    <div>
                      <label className="mb-1.5 block text-sm font-medium text-foreground">
                        To
                      </label>
                      <input
                        className={FIELD_CLASS}
                        value={draftTo}
                        onChange={(event) => setDraftTo(event.target.value)}
                        placeholder="recipient address"
                      />
                    </div>
                  </div>

                  <div className="grid gap-3 md:grid-cols-2">
                    <div>
                      <label className="mb-1.5 block text-sm font-medium text-foreground">
                        Amount
                      </label>
                      <input
                        className={FIELD_CLASS}
                        value={draftAmount}
                        onChange={(event) => setDraftAmount(event.target.value)}
                        placeholder="100"
                        inputMode="numeric"
                      />
                    </div>
                    <div>
                      <label className="mb-1.5 block text-sm font-medium text-foreground">
                        Fee
                      </label>
                      <input
                        className={FIELD_CLASS}
                        value={draftFee}
                        onChange={(event) => setDraftFee(event.target.value)}
                        placeholder="1"
                        inputMode="numeric"
                      />
                    </div>
                  </div>

                  <div>
                    <label className="mb-1.5 block text-sm font-medium text-foreground">
                      Memo
                    </label>
                    <textarea
                      className={TEXTAREA_CLASS}
                      value={draftMemo}
                      onChange={(event) => setDraftMemo(event.target.value)}
                      placeholder="Optional note for the recipient"
                    />
                  </div>
                </div>

                <div className="grid gap-3 rounded-2xl border border-border/60 bg-muted/20 p-4">
                  <DetailRow
                    label="Expected nonce"
                    value={
                      walletView.account ? formatCount(walletView.account.nonce + 1) : "Load an account first"
                    }
                  />
                  <DetailRow
                    label="CLI command"
                    value={
                      <pre className="whitespace-pre-wrap break-words font-mono text-xs leading-6 text-foreground">
                        {walletCommand}
                      </pre>
                    }
                    mono
                  />
                </div>
              </CardContent>
            </Card>

            <TelemetryCard
              health={health}
              metrics={metrics}
              chainParams={chainParams}
              slashes={slashEvents}
            />
          </div>
        </section>
      </main>

      <Footer />
    </div>
  );
}
