import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  FileText,
  Scale,
  Settings,
  CircleCheck,
  CircleX,
  Timer,
} from "lucide-react";

const txTypes = [
  { name: "Transfer", description: "Send tokens between accounts" },
  { name: "Stake", description: "Lock tokens to participate in consensus" },
  { name: "Unstake", description: "Withdraw staked tokens" },
  { name: "CreateProposal", description: "Submit a governance proposal" },
  { name: "VoteProposal", description: "Vote on active proposals" },
];

const governanceParams = [
  {
    icon: Settings,
    param: "ChangeBlockReward",
    description: "Adjust the block production reward",
  },
  {
    icon: Timer,
    param: "ChangeBlockInterval",
    description: "Modify time between blocks",
  },
  {
    icon: FileText,
    param: "ChangeMaxTxsPerBlock",
    description: "Set maximum transactions per block",
  },
  {
    icon: Scale,
    param: "ChangeStakeWeight",
    description: "Adjust staking weight in hybrid consensus",
  },
];

const defaults = [
  { label: "Min Proposal Stake", value: "100 tokens" },
  { label: "Quorum", value: "20% of total staked" },
  { label: "Approval Threshold", value: "50.01% yes votes" },
  { label: "Attestation Quorum", value: "3 unique peers" },
];

export function Governance() {
  return (
    <section
      id="governance"
      className="border-y border-border/40 bg-muted/30 py-20 sm:py-28"
    >
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
            Native Governance & Staking
          </h2>
          <p className="mt-4 text-lg text-muted-foreground">
            Propose and vote on protocol changes directly on-chain. Passed
            proposals take effect immediately without node restarts.
          </p>
        </div>

        <div className="mx-auto mt-16 grid max-w-6xl grid-cols-1 gap-8 lg:grid-cols-2">
          {/* Transaction Types */}
          <Card className="border-border/50 bg-card/50">
            <CardContent className="p-6">
              <h3 className="mb-4 text-lg font-semibold text-foreground">
                Transaction Types
              </h3>
              <div className="space-y-3">
                {txTypes.map((tx) => (
                  <div
                    key={tx.name}
                    className="flex items-center gap-3 rounded-lg border border-border/30 px-4 py-3 transition-colors hover:bg-muted/30"
                  >
                    <Badge
                      variant="outline"
                      className="shrink-0 font-mono text-xs"
                    >
                      {tx.name}
                    </Badge>
                    <span className="text-sm text-muted-foreground">
                      {tx.description}
                    </span>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Governance Actions */}
          <div className="space-y-6">
            <Card className="border-border/50 bg-card/50">
              <CardContent className="p-6">
                <h3 className="mb-4 text-lg font-semibold text-foreground">
                  Governable Parameters
                </h3>
                <div className="space-y-3">
                  {governanceParams.map((item) => (
                    <div
                      key={item.param}
                      className="flex items-center gap-3"
                    >
                      <item.icon
                        className="size-4 shrink-0 text-accent"
                        aria-hidden="true"
                      />
                      <span className="font-mono text-sm text-foreground">
                        {item.param}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        -- {item.description}
                      </span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Proposal Lifecycle */}
            <Card className="border-border/50 bg-card/50">
              <CardContent className="p-6">
                <h3 className="mb-4 text-lg font-semibold text-foreground">
                  Proposal Lifecycle
                </h3>
                <div className="flex items-center gap-4">
                  <div className="flex items-center gap-1.5">
                    <Timer
                      className="size-4 text-primary"
                      aria-hidden="true"
                    />
                    <span className="text-sm font-medium text-foreground">
                      Active
                    </span>
                  </div>
                  <svg
                    className="size-4 text-muted-foreground"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    aria-hidden="true"
                  >
                    <path d="M5 12h14M12 5l7 7-7 7" />
                  </svg>
                  <div className="flex items-center gap-1.5">
                    <CircleCheck
                      className="size-4 text-green-500"
                      aria-hidden="true"
                    />
                    <span className="text-sm font-medium text-foreground">
                      Passed
                    </span>
                  </div>
                  <span className="text-muted-foreground">/</span>
                  <div className="flex items-center gap-1.5">
                    <CircleX
                      className="size-4 text-destructive"
                      aria-hidden="true"
                    />
                    <span className="text-sm font-medium text-foreground">
                      Rejected
                    </span>
                  </div>
                </div>

                <div className="mt-6 space-y-2">
                  {defaults.map((d) => (
                    <div
                      key={d.label}
                      className="flex items-center justify-between"
                    >
                      <span className="text-sm text-muted-foreground">
                        {d.label}
                      </span>
                      <span className="font-mono text-sm font-medium text-foreground">
                        {d.value}
                      </span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </section>
  );
}
