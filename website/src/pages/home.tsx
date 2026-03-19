import {
  Gauge,
  Globe,
  LayoutDashboard,
  Radar,
  Server,
  Shield,
  Vote,
} from "lucide-react";

import { CtaBanner, InsightGrid, PageHero, ProcessList, SectionHeading, StatGrid } from "@/components/marketing/page-primitives";
import { SEO } from "@/components/seo";
import { REPOSITORY_URL } from "@/content/site";

const homeSignals = [
  { label: "Measured Inputs", value: "Download, upload, latency, uptime, stability" },
  { label: "Economic Weight", value: "Stake remains part of selection, not the entire model" },
  { label: "Trust Hardening", value: "Identity confidence, attestations, and reputation" },
  { label: "Penalty Memory", value: "Slashing history persists as a trust discount" },
];

const valueCards = [
  {
    icon: Globe,
    eyebrow: "Consensus Thesis",
    title: "Internet performance becomes a protocol signal.",
    description:
      "NetChain measures the quality of a validator's network conditions instead of assuming every node can reach the same operating baseline.",
    meta: "Proof of Internet tracks download speed, upload speed, latency, uptime, and stability.",
  },
  {
    icon: Shield,
    eyebrow: "Security Model",
    title: "Stake remains important without becoming the only gate.",
    description:
      "Selection blends economic weight with observed performance, reputation, identity confidence, and slashing history so the validator set is harder to buy or spoof.",
    meta: "Hybrid scoring reduces the single-signal fragility of pure stake or pure telemetry.",
  },
  {
    icon: Radar,
    eyebrow: "Anti-Gaming",
    title: "Attestations and challenge-response checks raise confidence.",
    description:
      "Peer metric announcements are paired with validation logic, challenge workflows, and quorum-style attestations that make fake scores materially harder to sustain.",
    meta: "The protocol rewards stable operators that can be repeatedly verified by peers.",
  },
];

const operatingSurface = [
  {
    icon: Server,
    eyebrow: "Node Runtime",
    title: "Rust node with persistent local state",
    description:
      "The repository ships blocks, state transitions, mempool handling, block production, sled-backed storage, and runtime config loading in the main node binary.",
  },
  {
    icon: LayoutDashboard,
    eyebrow: "Explorer Surface",
    title: "Live telemetry inside the website",
    description:
      "The route-based explorer exposes chain data, validator health, proposals, wallet watchlists, and monitoring snapshots without leaving the front-end.",
  },
  {
    icon: Vote,
    eyebrow: "Governance Surface",
    title: "Runtime controls stay on-chain",
    description:
      "Proposal voting can update block reward, block interval, maximum transactions per block, and stake weight after quorum and approval are met.",
  },
];

const protocolFlow = [
  {
    step: "01",
    title: "Nodes publish and verify performance measurements",
    description:
      "Validators gather internet-quality metrics and circulate them through the network, where peers can compare readings and challenge suspicious claims.",
  },
  {
    step: "02",
    title: "The protocol computes a composite validator score",
    description:
      "Proof of Internet telemetry is combined with stake, reputation, identity confidence, attestation support, and slashing history before any validator is favored.",
  },
  {
    step: "03",
    title: "Block production follows the stronger composite profile",
    description:
      "Nodes with better measured delivery characteristics and healthier trust signals receive more weight in validator selection than unstable or poorly attested peers.",
  },
  {
    step: "04",
    title: "Governance and telemetry keep the system observable",
    description:
      "Operators can inspect health, metrics, WebSocket events, and proposal status while protocol parameters remain adjustable through stake-weighted voting.",
  },
];

const protocolStats = [
  {
    value: "5",
    label: "PoI Signals",
    detail: "Download, upload, latency, uptime, and stability feed the network-performance score.",
  },
  {
    value: "4",
    label: "Governance Actions",
    detail: "Block reward, block interval, stake weight, and max transactions per block can be changed through proposals.",
  },
  {
    value: "3",
    label: "Operator Interfaces",
    detail: "JSON-RPC, WebSocket events, and Prometheus-style monitoring are available by default.",
  },
  {
    value: "2",
    label: "Binaries",
    detail: "The repository includes the main `netchain` node and the `netchain-wallet` CLI.",
  },
];

function HomeSignalBoard() {
  return (
    <div className="surface-card overflow-hidden">
      <div className="border-b border-border/70 px-6 py-5">
        <p className="eyebrow">Validator Profile</p>
        <h2 className="mt-3 font-heading text-3xl text-foreground text-balance">
          NetChain scores the network, not only the wallet.
        </h2>
      </div>
      <div className="grid gap-3 px-6 py-6">
        {homeSignals.map((signal, index) => (
          <div
            key={signal.label}
            className="rounded-[24px] border border-border/70 bg-secondary/52 px-5 py-4"
          >
            <div className="flex items-center justify-between gap-3">
              <p className="text-sm font-semibold text-foreground">{signal.label}</p>
              <span
                className="flex size-9 items-center justify-center rounded-full border border-border/60 bg-card font-heading text-sm text-primary"
                aria-hidden="true"
              >
                0{index + 1}
              </span>
            </div>
            <p className="mt-3 text-sm leading-7 text-muted-foreground">{signal.value}</p>
          </div>
        ))}
        <div className="rounded-[24px] border border-primary/18 bg-primary/8 px-5 py-4">
          <div className="flex items-center gap-3 text-primary">
            <Gauge className="size-5" aria-hidden="true" />
            <p className="text-sm font-semibold uppercase tracking-[0.18em]">Composite Selection</p>
          </div>
          <p className="mt-3 text-sm leading-7 text-foreground/80">
            The protocol keeps stake in the model while making room for trust,
            verification, and measurable delivery quality.
          </p>
        </div>
      </div>
    </div>
  );
}

export function HomePage() {
  return (
    <div>
      <SEO
        title="NetChain | Experimental Proof of Internet Layer-1"
        description="NetChain is an experimental Rust Layer-1 where validator selection blends Proof of Internet telemetry with stake, reputation, identity confidence, attestations, and slashing history."
        keywords="NetChain, Proof of Internet, Layer-1, Rust blockchain, validator telemetry, staking, governance"
      />

      <PageHero
        eyebrow="Infrastructure For Measurable Networks"
        title="A Layer-1 designed around how well a node can actually deliver."
        description="NetChain is an experimental blockchain in Rust that makes internet performance part of consensus. Validator selection blends Proof of Internet metrics with stake, identity confidence, reputation, multi-party attestations, and slashing memory."
        primaryAction={{ label: "Read the Architecture", to: "/technology" }}
        secondaryAction={{ label: "Run a Local Node", to: "/get-started" }}
        tertiaryAction={{ label: "Open the Repository", href: REPOSITORY_URL }}
        metrics={[
          { label: "Implementation", value: "Rust 2021 node plus wallet CLI" },
          { label: "Networking", value: "libp2p, JSON-RPC, WebSocket, and monitoring" },
          { label: "State", value: "Native staking, proposals, and slashing transitions" },
          { label: "Operator View", value: "Explorer route for telemetry and governance reads" },
        ]}
        aside={<HomeSignalBoard />}
      />

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Why NetChain"
            title="Institutional clarity for an experimental protocol."
            description="The project is still a prototype, but the design problem is concrete: most validator models ignore whether a node can consistently deliver work over the network. NetChain treats delivery quality as something that should be measured, challenged, and weighted."
          />
          <InsightGrid items={valueCards} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/42">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Protocol At A Glance"
            title="The repository already spans runtime logic, governance, and observability."
            description="NetChain is more than a landing page concept. The codebase includes block validation, state transitions, libp2p networking, storage, monitoring, WebSocket events, and route-based explorer views."
          />
          <StatGrid items={protocolStats} />
        </div>
      </section>

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Consensus Flow"
            title="How validator quality moves from measurement to execution."
            description="Proof of Internet is one signal inside a broader trust model. Selection becomes a pipeline of measurement, verification, scoring, block production, and governance oversight instead of a single static weight."
          />
          <ProcessList items={protocolFlow} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/36">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Operational Surface"
            title="Built for operators, contributors, and protocol reviewers."
            description="The same repository supports node bring-up, wallet actions, health inspection, proposal review, and explorer-style telemetry. That makes the project easier to inspect end to end."
          />
          <InsightGrid items={operatingSurface} columns={3} />
        </div>
      </section>

      <CtaBanner
        eyebrow="Next Step"
        title="Inspect the code, run the node, and decide where the model needs pressure."
        description="NetChain should be evaluated as an open protocol experiment. The fastest way to understand it is to read the architecture, start the local services, and compare the governance and telemetry surfaces against the consensus thesis."
        primaryAction={{ label: "Get Started", to: "/get-started" }}
        secondaryAction={{ label: "Open Explorer", to: "/dashboard" }}
      />
    </div>
  );
}
