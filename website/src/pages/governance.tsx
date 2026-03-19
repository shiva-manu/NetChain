import { ArrowLeftRight, Shield, Vote, Wallet } from "lucide-react";

import { CtaBanner, InsightGrid, PageHero, ProcessList, SectionHeading, StatGrid } from "@/components/marketing/page-primitives";
import { SEO } from "@/components/seo";

const lifecycleSteps = [
  {
    step: "01",
    title: "Stake first",
    description:
      "Proposal creation requires an existing stake position, which keeps governance participation anchored to economic exposure inside the protocol.",
  },
  {
    step: "02",
    title: "Validate the action before mempool entry",
    description:
      "Supported proposal types are checked before they are accepted, so malformed or unsupported governance changes do not circulate as valid state transitions.",
  },
  {
    step: "03",
    title: "Vote with currently staked balance",
    description:
      "Voting power tracks the amount of stake held by the voter, which keeps governance influence aligned with active protocol exposure.",
  },
  {
    step: "04",
    title: "Meet quorum and approval thresholds",
    description:
      "A proposal becomes actionable only after the required share of total staked balance participates and yes-votes exceed the configured approval threshold.",
  },
  {
    step: "05",
    title: "Apply runtime changes without restarting the node",
    description:
      "Passed proposals can update selected chain parameters live, which makes governance operational instead of purely symbolic.",
  },
];

const actionCards = [
  {
    icon: ArrowLeftRight,
    eyebrow: "Proposal Action",
    title: "Change block reward",
    description:
      "Governance can update the reward rate paid for block production so incentives remain adjustable as the network model changes.",
  },
  {
    icon: Vote,
    eyebrow: "Proposal Action",
    title: "Change block interval",
    description:
      "Block cadence is adjustable through on-chain governance rather than fixed permanently in code or runtime configuration.",
  },
  {
    icon: Wallet,
    eyebrow: "Proposal Action",
    title: "Change stake weight",
    description:
      "The influence of stake inside the hybrid validator model can be tuned through proposals if the network wants a different economic balance.",
  },
  {
    icon: Shield,
    eyebrow: "Proposal Action",
    title: "Change max transactions per block",
    description:
      "Execution throughput controls are part of the governance surface, which lets the protocol respond to workload and performance findings.",
  },
];

const governanceStats = [
  {
    value: "100",
    label: "Minimum Proposal Stake",
    detail: "A participant needs an active stake position before a proposal can be created.",
  },
  {
    value: "20%",
    label: "Quorum",
    detail: "At least one-fifth of total staked balance must participate for the vote to count.",
  },
  {
    value: "50.01%",
    label: "Approval Threshold",
    detail: "Yes-votes among participating stake must cross the configured majority requirement.",
  },
  {
    value: "4",
    label: "Runtime Actions",
    detail: "Four parameter changes are currently supported by proposal execution in the node.",
  },
];

function GovernanceBoard() {
  return (
    <div className="surface-card overflow-hidden">
      <div className="border-b border-border/70 px-6 py-5">
        <p className="eyebrow">Decision Surface</p>
        <h2 className="mt-3 font-heading text-3xl text-foreground text-balance">
          Governance changes protocol behavior, not just documentation.
        </h2>
      </div>
      <div className="grid gap-3 px-6 py-6">
        {[
          "Stake required to create proposals",
          "Vote weight equals active staked balance",
          "Proposal status tracks active, passed, or rejected",
          "Passed actions update runtime parameters",
        ].map((item) => (
          <div key={item} className="rounded-[24px] border border-border/70 bg-secondary/55 px-5 py-4">
            <p className="text-sm font-semibold text-foreground">{item}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

export function GovernancePage() {
  return (
    <div>
      <SEO
        title="NetChain Governance | Stake-Weighted Runtime Control"
        description="Review NetChain governance, proposal validation, quorum rules, approval thresholds, and runtime parameter updates."
        keywords="NetChain governance, proposal voting, stake weighted voting, block reward, stake weight"
      />

      <PageHero
        eyebrow="Governance Model"
        title="Runtime parameters stay adjustable through stake-backed coordination."
        description="NetChain governance is not separated from execution. Proposal creation, voting, status resolution, and parameter updates all live inside the same state machine as staking and slashing."
        primaryAction={{ label: "Read the Docs", to: "/docs" }}
        secondaryAction={{ label: "Run Governance Locally", to: "/get-started" }}
        metrics={[
          { label: "Proposal Gate", value: "Existing stake position required" },
          { label: "Vote Weight", value: "Currently staked balance" },
          { label: "Execution", value: "Passed actions can update runtime parameters" },
          { label: "Safety", value: "Proposal actions validated before mempool entry" },
        ]}
        aside={<GovernanceBoard />}
      />

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Lifecycle"
            title="A short governance path with clear admission and execution rules."
            description="The design stays intentionally narrow: only supported actions are accepted, voting power is tied to stake, and passed proposals directly change selected runtime settings."
          />
          <ProcessList items={lifecycleSteps} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/40">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Current Proposal Set"
            title="The present governance surface is focused on core runtime tuning."
            description="The implementation exposes only a small set of parameter changes, which is appropriate for an experimental protocol where execution simplicity still matters."
          />
          <InsightGrid items={actionCards} columns={2} />
        </div>
      </section>

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Defaults"
            title="The key thresholds are explicit and easy to audit."
            description="Because the voting model is already implemented, the current defaults matter. They define how much stake is needed to propose, how much participation is required, and when a proposal crosses the line."
          />
          <StatGrid items={governanceStats} />
        </div>
      </section>

      <CtaBanner
        eyebrow="Next Layer"
        title="Move from governance rules to the actual command surface."
        description="The docs page brings the repository commands, RPC methods, WebSocket subscription format, and monitoring endpoints into one practical reference."
        primaryAction={{ label: "Open Docs", to: "/docs" }}
        secondaryAction={{ label: "Open Explorer", to: "/dashboard" }}
      />
    </div>
  );
}
