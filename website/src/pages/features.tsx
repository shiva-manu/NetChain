import {
  Activity,
  Gauge,
  LayoutDashboard,
  Server,
  Shield,
  Vote,
  Wallet,
  Wifi,
} from "lucide-react";

import { CtaBanner, InsightGrid, PageHero, SectionHeading, StatGrid } from "@/components/marketing/page-primitives";
import { SEO } from "@/components/seo";

const featureCards = [
  {
    icon: Gauge,
    eyebrow: "Validator Scoring",
    title: "Composite validator selection",
    description:
      "NetChain does not treat stake as a complete proxy for quality. Selection combines internet performance, stake, identity confidence, reputation, attestation support, and slashing history.",
    meta: "Proof of Internet remains central, but it is no longer isolated.",
  },
  {
    icon: Shield,
    eyebrow: "Trust Hardening",
    title: "Anti-gaming checks and persistent penalties",
    description:
      "Challenge-response validation and slashing-aware scoring make false metric announcements more expensive than simple self-reporting schemes.",
    meta: "Past misconduct continues to lower trust instead of disappearing after a single epoch.",
  },
  {
    icon: Wifi,
    eyebrow: "Runtime Interfaces",
    title: "Network visibility from the first run",
    description:
      "Health checks, Prometheus-style metrics, WebSocket events, and JSON-RPC methods expose the behavior of the node while it is producing blocks or processing governance actions.",
    meta: "Operators can inspect the system without custom tooling.",
  },
  {
    icon: Vote,
    eyebrow: "On-Chain Governance",
    title: "Protocol parameters can move without a restart",
    description:
      "Proposal actions are validated before mempool admission and, if passed, can update core runtime parameters while the network stays online.",
    meta: "The repository already supports four proposal action types.",
  },
  {
    icon: Wallet,
    eyebrow: "Native Actions",
    title: "Transfers, staking, and proposal voting in one state model",
    description:
      "The transaction set covers basic value transfer, stake management, proposal creation, and voting so the governance surface is not bolted on as a side system.",
    meta: "Voting power follows currently staked balance.",
  },
  {
    icon: LayoutDashboard,
    eyebrow: "Operator UX",
    title: "Explorer-style reads are part of the website",
    description:
      "The front-end includes a dashboard route for blocks, proposals, wallet watchlists, and runtime snapshots, making protocol behavior easier to inspect during local development.",
    meta: "Marketing and telemetry live inside the same project surface.",
  },
];

const featureStats = [
  {
    value: "5",
    label: "Measured Metrics",
    detail: "Download, upload, latency, uptime, and stability influence Proof of Internet scoring.",
  },
  {
    value: "4",
    label: "Governance Changes",
    detail: "Proposal execution can update block reward, block interval, max transactions, and stake weight.",
  },
  {
    value: "6+",
    label: "Trust Inputs",
    detail: "Stake, telemetry, attestations, identity, reputation, penalties, and liveness all contribute to operator quality.",
  },
  {
    value: "1",
    label: "Unified Codebase",
    detail: "Node runtime, wallet CLI, explorer, and protocol website are shipped together for faster inspection.",
  },
];

function FeatureBoard() {
  return (
    <div className="surface-card overflow-hidden">
      <div className="border-b border-border/70 px-6 py-5">
        <p className="eyebrow">Capability Stack</p>
        <h2 className="mt-3 font-heading text-3xl text-foreground text-balance">
          Feature work stays close to the protocol core.
        </h2>
      </div>
      <div className="grid gap-3 px-6 py-6">
        {[
          {
            label: "Consensus",
            detail: "Hybrid Proof of Internet scoring",
            icon: Gauge,
          },
          {
            label: "Networking",
            detail: "libp2p gossip plus RPC and WebSocket",
            icon: Wifi,
          },
          {
            label: "State",
            detail: "Staking, slashing, and governance transitions",
            icon: Activity,
          },
          {
            label: "Storage",
            detail: "Persistent sled-backed local data",
            icon: Server,
          },
        ].map((item) => (
          <div key={item.label} className="rounded-[24px] border border-border/70 bg-secondary/55 px-5 py-4">
            <div className="flex items-center gap-3 text-primary">
              <item.icon className="size-5" aria-hidden="true" />
              <p className="text-sm font-semibold uppercase tracking-[0.18em]">{item.label}</p>
            </div>
            <p className="mt-3 text-sm leading-7 text-muted-foreground">{item.detail}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

export function FeaturesPage() {
  return (
    <div>
      <SEO
        title="NetChain Features | Hybrid Consensus, Governance, and Telemetry"
        description="Explore NetChain features including hybrid validator scoring, anti-gaming checks, governance controls, runtime telemetry, and native staking flows."
        keywords="NetChain features, Proof of Internet, validator scoring, governance, slashing, telemetry"
      />

      <PageHero
        eyebrow="Protocol Features"
        title="Features that connect the consensus thesis to day-to-day operation."
        description="NetChain is structured as a practical operator stack. Consensus signals, governance controls, storage, networking, and observability are wired together so the protocol can be evaluated as a working system."
        primaryAction={{ label: "See How It Works", to: "/how-it-works" }}
        secondaryAction={{ label: "Review the Docs", to: "/docs" }}
        metrics={[
          { label: "Consensus", value: "Proof of Internet plus stake and trust signals" },
          { label: "Governance", value: "Stake-weighted proposals with runtime updates" },
          { label: "Telemetry", value: "Health, metrics, WebSocket, and explorer views" },
          { label: "Operations", value: "Rust node, wallet CLI, Docker, and config overrides" },
        ]}
        aside={<FeatureBoard />}
      />

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Capability Overview"
            title="The repo already exposes the surfaces a real operator needs."
            description="The goal of the front-end is no longer to sell abstraction. It should explain how the protocol behaves, where the trust model hardens, and what an engineer can inspect immediately."
          />
          <InsightGrid items={featureCards} columns={3} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/40">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Numbers That Matter"
            title="A small protocol surface, but one with enough hooks to test seriously."
            description="NetChain is still experimental, yet the current implementation already covers the most important edges: validator quality, governance transitions, telemetry, and multiple operator entry points."
          />
          <StatGrid items={featureStats} />
        </div>
      </section>

      <CtaBanner
        eyebrow="Continue"
        title="Follow the validator-selection path from raw measurement to block production."
        description="The next page breaks down how measurements are announced, challenged, blended with stake and reputation, and fed into validator choice and governance oversight."
        primaryAction={{ label: "Open How It Works", to: "/how-it-works" }}
        secondaryAction={{ label: "Open Explorer", to: "/dashboard" }}
      />
    </div>
  );
}
