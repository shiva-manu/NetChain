import { Gauge, Radar, Shield, Vote, Wifi } from "lucide-react";

import { CtaBanner, InsightGrid, PageHero, ProcessList, SectionHeading } from "@/components/marketing/page-primitives";
import { SEO } from "@/components/seo";

const scoringInputs = [
  {
    icon: Wifi,
    eyebrow: "Signal Family",
    title: "Measured network performance",
    description:
      "Download speed, upload speed, latency, uptime, and stability establish the Proof of Internet contribution to each node's profile.",
  },
  {
    icon: Gauge,
    eyebrow: "Signal Family",
    title: "Economic weight",
    description:
      "Stake still matters, but it is blended with measured performance rather than acting as the sole selector for validator influence.",
  },
  {
    icon: Shield,
    eyebrow: "Signal Family",
    title: "Trust and penalties",
    description:
      "Identity confidence, attestation quorum, reputation, and slashing history keep historical behavior inside the selection model.",
  },
];

const flowSteps = [
  {
    step: "01",
    title: "Measure node delivery quality",
    description:
      "Peers gather internet-quality readings that reflect how well a validator can participate in block production and network propagation.",
  },
  {
    step: "02",
    title: "Announce metrics to the network",
    description:
      "Validators share measurements so other participants can compare observations instead of accepting a private self-score at face value.",
  },
  {
    step: "03",
    title: "Challenge and attest suspicious results",
    description:
      "Challenge-response mechanisms and multi-party attestations raise confidence in legitimate reports while making fabricated performance harder to maintain.",
  },
  {
    step: "04",
    title: "Blend telemetry with stake and reputation",
    description:
      "The protocol computes a composite trust profile that incorporates performance, economic weight, identity, reputation, and prior penalties.",
  },
  {
    step: "05",
    title: "Select validators with the stronger composite profile",
    description:
      "Nodes that consistently deliver better network behavior and sustain healthier trust signals receive more favorable validator weighting.",
  },
  {
    step: "06",
    title: "Expose results through governance and telemetry",
    description:
      "Explorer reads, WebSocket events, health checks, and proposal flows make the ongoing state of the protocol visible to operators and reviewers.",
  },
];

const controls = [
  {
    icon: Radar,
    eyebrow: "Control Surface",
    title: "Challenge-response validation",
    description:
      "Measurement claims can be challenged rather than passively accepted, which adds friction to nodes attempting to game the scoring process.",
  },
  {
    icon: Shield,
    eyebrow: "Control Surface",
    title: "Persistent slashing memory",
    description:
      "Invalid blocks, fraudulent metrics, or missed duties lower future trust by leaving a durable penalty in the validator profile.",
  },
  {
    icon: Vote,
    eyebrow: "Control Surface",
    title: "Governance-backed parameter changes",
    description:
      "Passed proposals can change runtime parameters without a restart, which makes the scoring model adjustable through on-chain coordination.",
  },
];

function ConsensusBoard() {
  return (
    <div className="surface-card overflow-hidden">
      <div className="border-b border-border/70 px-6 py-5">
        <p className="eyebrow">Selection Model</p>
        <h2 className="mt-3 font-heading text-3xl text-foreground text-balance">
          Multiple signals feed validator choice.
        </h2>
      </div>
      <div className="grid gap-3 px-6 py-6">
        {[
          "Proof of Internet measurements",
          "Stake weight",
          "Identity confidence",
          "Attestation quorum",
          "Reputation history",
          "Slashing penalties",
        ].map((item) => (
          <div key={item} className="rounded-[24px] border border-border/70 bg-secondary/55 px-5 py-4">
            <div className="flex items-center gap-3">
              <span className="size-2.5 rounded-full bg-primary" aria-hidden="true" />
              <p className="text-sm font-semibold text-foreground">{item}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export function HowItWorksPage() {
  return (
    <div>
      <SEO
        title="How NetChain Works | Proof of Internet Consensus Flow"
        description="Learn how NetChain measures internet performance, validates claims, computes composite validator scores, and feeds results into governance and block production."
        keywords="how NetChain works, Proof of Internet consensus, validator scoring, attestation quorum, slashing"
      />

      <PageHero
        eyebrow="Consensus Flow"
        title="Measurement, attestation, and governance are part of the same loop."
        description="NetChain treats validator selection as a process instead of a single static weight. Network conditions are measured, challenged, blended with trust signals, and then exposed again through runtime telemetry and proposal controls."
        primaryAction={{ label: "Read the Technology", to: "/technology" }}
        secondaryAction={{ label: "Open the Explorer", to: "/dashboard" }}
        metrics={[
          { label: "Measurement", value: "Proof of Internet tracks five delivery metrics" },
          { label: "Verification", value: "Peer challenges and attestations harden the score" },
          { label: "Selection", value: "Stake and trust signals shape validator weighting" },
          { label: "Oversight", value: "Governance and telemetry keep the process inspectable" },
        ]}
        aside={<ConsensusBoard />}
      />

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Signal Composition"
            title="Proof of Internet remains visible, but it is not isolated from trust."
            description="The protocol aims to avoid two weak extremes: selecting validators on stake alone or trusting raw network telemetry without historical context. Composite scoring sits between those models."
          />
          <InsightGrid items={scoringInputs} columns={3} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/40">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Lifecycle"
            title="The path from raw measurement to actual validator influence."
            description="Every stage is designed to convert a noisy network observation into a more trustworthy operational signal. That includes peer comparison, trust weighting, and stateful penalties."
          />
          <ProcessList items={flowSteps} />
        </div>
      </section>

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Safeguards"
            title="Controls that keep the consensus model from collapsing into vanity metrics."
            description="Proof of Internet only matters if the protocol can detect manipulation, remember bad behavior, and adapt governance settings when the operating assumptions change."
          />
          <InsightGrid items={controls} columns={3} />
        </div>
      </section>

      <CtaBanner
        eyebrow="Next Layer"
        title="Review how the codebase is organized behind the model."
        description="The technology page maps the runtime into chain, networking, node, wallet, and Proof of Internet modules so the conceptual flow can be tied back to implementation boundaries."
        primaryAction={{ label: "Open Technology", to: "/technology" }}
        secondaryAction={{ label: "See Features", to: "/features" }}
      />
    </div>
  );
}
