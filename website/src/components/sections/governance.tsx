import { Badge } from "@/components/ui/badge";
import {
  FileText,
  Scale,
  Settings,
  CircleCheck,
  CircleX,
  Timer,
  ArrowRight,
  Vote,
} from "lucide-react";
import { cn } from "@/lib/utils";

const txTypes = [
  { name: "Transfer", description: "Send tokens between accounts", icon: "💸" },
  { name: "Stake", description: "Lock tokens for consensus participation", icon: "🔒" },
  { name: "Unstake", description: "Withdraw staked tokens", icon: "🔓" },
  { name: "CreateProposal", description: "Submit a governance proposal", icon: "📝" },
  { name: "VoteProposal", description: "Vote on active proposals", icon: "🗳️" },
];

const governanceParams = [
  {
    icon: Settings,
    param: "ChangeBlockReward",
    description: "Adjust the block production reward",
    color: "from-cyan-500 to-blue-500",
  },
  {
    icon: Timer,
    param: "ChangeBlockInterval",
    description: "Modify time between blocks",
    color: "from-emerald-500 to-teal-500",
  },
  {
    icon: FileText,
    param: "ChangeMaxTxsPerBlock",
    description: "Set maximum transactions per block",
    color: "from-violet-500 to-purple-500",
  },
  {
    icon: Scale,
    param: "ChangeStakeWeight",
    description: "Adjust staking weight in hybrid consensus",
    color: "from-orange-500 to-amber-500",
  },
];

const defaults = [
  { label: "Min Proposal Stake", value: "100 tokens", icon: "💰" },
  { label: "Quorum", value: "20% of total staked", icon: "📊" },
  { label: "Approval Threshold", value: "50.01% yes votes", icon: "✅" },
  { label: "Attestation Quorum", value: "3 unique peers", icon: "👥" },
];

export function Governance() {
  return (
    <section
      id="governance"
      className="relative overflow-hidden py-24 sm:py-32"
    >
      {/* Background */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-muted/20 to-transparent" />
        <div className="absolute left-1/4 top-0 h-[600px] w-[600px] rounded-full bg-violet-500/5 blur-[120px]" />
        <div className="absolute bottom-0 right-1/4 h-[400px] w-[400px] rounded-full bg-primary/5 blur-[100px]" />
      </div>

      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-3xl text-center">
          <span className="mb-4 inline-block text-sm font-semibold uppercase tracking-wider text-primary">
            Governance
          </span>
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl lg:text-5xl" style={{ textWrap: "balance" }}>
            Native Governance{" "}
            <span className="text-gradient">& Staking</span>
          </h2>
          <p className="mt-6 text-lg leading-relaxed text-muted-foreground">
            Propose and vote on protocol changes directly on-chain. Passed
            proposals take effect immediately without node restarts.
          </p>
        </div>

        <div className="mx-auto mt-16 grid max-w-6xl grid-cols-1 gap-8 lg:grid-cols-2">
          {/* Transaction Types */}
          <div 
            className="opacity-0 animate-fade-in-up"
            style={{ animationDelay: "100ms", animationFillMode: "forwards" }}
          >
            <div className="mb-6 flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-xl bg-gradient-to-br from-primary to-accent text-white">
                <Vote className="size-5" aria-hidden="true" />
              </div>
              <h3 className="text-xl font-semibold text-foreground">
                Transaction Types
              </h3>
            </div>

            <div className="space-y-3">
              {txTypes.map((tx) => (
                <div
                  key={tx.name}
                  className={cn(
                    "group flex items-center gap-4 rounded-xl border border-border/50 bg-card/30 p-4 backdrop-blur-sm",
                    "transition-all duration-300 hover:border-border hover:bg-card/50"
                  )}
                >
                  <span className="text-2xl" aria-hidden="true">{tx.icon}</span>
                  <div className="flex-1">
                    <Badge
                      variant="outline"
                      className="mb-1 border-primary/30 bg-primary/5 font-mono text-xs text-primary"
                    >
                      {tx.name}
                    </Badge>
                    <p className="text-sm text-muted-foreground">
                      {tx.description}
                    </p>
                  </div>
                  <ArrowRight 
                    className="size-4 text-muted-foreground opacity-0 transition-all duration-300 group-hover:translate-x-1 group-hover:opacity-100" 
                    aria-hidden="true"
                  />
                </div>
              ))}
            </div>
          </div>

          {/* Right column */}
          <div className="space-y-8">
            {/* Governance Actions */}
            <div 
              className="opacity-0 animate-fade-in-up"
              style={{ animationDelay: "200ms", animationFillMode: "forwards" }}
            >
              <h3 className="mb-4 text-lg font-semibold text-foreground">
                Governable Parameters
              </h3>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                {governanceParams.map((item) => (
                  <div
                    key={item.param}
                    className="group relative overflow-hidden rounded-xl border border-border/50 bg-card/30 p-4 backdrop-blur-sm transition-all duration-300 hover:border-border hover:bg-card/50"
                  >
                    <div 
                      className={cn(
                        "mb-3 inline-flex size-8 items-center justify-center rounded-lg",
                        "bg-gradient-to-br text-white",
                        item.color
                      )}
                    >
                      <item.icon className="size-4" aria-hidden="true" />
                    </div>
                    <p className="font-mono text-sm font-medium text-foreground">
                      {item.param}
                    </p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {item.description}
                    </p>
                    {/* Glow */}
                    <div 
                      className={cn(
                        "pointer-events-none absolute -bottom-10 -right-10 size-20 rounded-full blur-2xl opacity-0 transition-opacity duration-300 group-hover:opacity-30",
                        "bg-gradient-to-br",
                        item.color
                      )}
                      aria-hidden="true"
                    />
                  </div>
                ))}
              </div>
            </div>

            {/* Proposal Lifecycle */}
            <div 
              className="opacity-0 animate-fade-in-up"
              style={{ animationDelay: "300ms", animationFillMode: "forwards" }}
            >
              <h3 className="mb-4 text-lg font-semibold text-foreground">
                Proposal Lifecycle
              </h3>
              <div className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
                {/* Status flow */}
                <div className="mb-6 flex items-center justify-center gap-3">
                  <div className="flex items-center gap-2 rounded-lg border border-primary/30 bg-primary/5 px-3 py-2">
                    <Timer className="size-4 text-primary" aria-hidden="true" />
                    <span className="text-sm font-medium text-foreground">Active</span>
                  </div>
                  <ArrowRight className="size-4 text-muted-foreground" aria-hidden="true" />
                  <div className="flex gap-2">
                    <div className="flex items-center gap-2 rounded-lg border border-green-500/30 bg-green-500/5 px-3 py-2">
                      <CircleCheck className="size-4 text-green-500" aria-hidden="true" />
                      <span className="text-sm font-medium text-foreground">Passed</span>
                    </div>
                    <span className="flex items-center text-muted-foreground">/</span>
                    <div className="flex items-center gap-2 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2">
                      <CircleX className="size-4 text-destructive" aria-hidden="true" />
                      <span className="text-sm font-medium text-foreground">Rejected</span>
                    </div>
                  </div>
                </div>

                {/* Defaults */}
                <div className="space-y-3">
                  {defaults.map((d) => (
                    <div
                      key={d.label}
                      className="flex items-center justify-between rounded-lg bg-muted/30 px-4 py-3 transition-colors hover:bg-muted/50"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-lg" aria-hidden="true">{d.icon}</span>
                        <span className="text-sm text-muted-foreground">{d.label}</span>
                      </div>
                      <span className="font-mono text-sm font-semibold text-foreground">
                        {d.value}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* CTA */}
        <div 
          className="mx-auto mt-16 max-w-2xl rounded-2xl border border-primary/20 bg-gradient-to-br from-primary/5 to-accent/5 p-8 text-center opacity-0 animate-fade-in-up"
          style={{ animationDelay: "400ms", animationFillMode: "forwards" }}
        >
          <h3 className="text-xl font-semibold text-foreground">
            Ready to Participate?
          </h3>
          <p className="mt-2 text-muted-foreground">
            Stake tokens, submit proposals, and shape the future of NetChain.
          </p>
          <div className="mt-6 flex flex-col items-center justify-center gap-4 sm:flex-row">
            <a
              href="/get-started"
              className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-primary to-accent px-6 py-3 font-semibold text-primary-foreground transition-shadow hover:shadow-lg hover:shadow-primary/20"
            >
              Get Started
              <ArrowRight className="size-4" aria-hidden="true" />
            </a>
            <a
              href="/docs"
              className="inline-flex items-center gap-2 rounded-xl border border-border px-6 py-3 font-semibold text-foreground transition-colors hover:bg-muted"
            >
              Read the Docs
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
