import { Blocks, Database, Globe, Server, Shield, Wallet, Wifi } from "lucide-react";

import { CtaBanner, InsightGrid, PageHero, SectionHeading, StatGrid } from "@/components/marketing/page-primitives";
import { SEO } from "@/components/seo";

const moduleCards = [
  {
    icon: Blocks,
    eyebrow: "Chain Layer",
    title: "`src/chain`",
    description:
      "Core blockchain primitives live here, including blocks, transactions, state validation, and the ledger rules that define legal transitions.",
    meta: "This is where the protocol state becomes canonical execution.",
  },
  {
    icon: Wifi,
    eyebrow: "Networking Layer",
    title: "`src/net`",
    description:
      "libp2p networking, JSON-RPC, WebSocket events, and monitoring handlers expose the node to peers, local operators, and dashboard clients.",
    meta: "Peer traffic and operator visibility share a coherent surface.",
  },
  {
    icon: Server,
    eyebrow: "Node Layer",
    title: "`src/node`",
    description:
      "Mempool coordination, block production, and sled-backed storage turn the consensus model into a running service with durable local state.",
    meta: "Runtime services stay separate from chain rules and network plumbing.",
  },
  {
    icon: Shield,
    eyebrow: "Consensus Layer",
    title: "`src/poi`",
    description:
      "Proof of Internet scoring, metric aggregation, attestations, and anti-gaming checks live in dedicated modules instead of being hidden inside generic validation code.",
    meta: "The experimental part of the protocol is isolated and inspectable.",
  },
  {
    icon: Wallet,
    eyebrow: "Operator Layer",
    title: "`src/wallet` and `src/bin/wallet.rs`",
    description:
      "Wallet helpers, encrypted key storage, and the CLI entry point support signing, local custody, and user-side interaction with protocol state.",
    meta: "The project ships a first-party wallet path alongside the node.",
  },
  {
    icon: Globe,
    eyebrow: "App Layer",
    title: "`website/`",
    description:
      "The marketing site and explorer route are a separate front-end project that sit next to the node, giving the repository a full-stack inspection surface.",
    meta: "Protocol narrative and live runtime reads stay connected.",
  },
];

const technologyStats = [
  {
    value: "30333",
    label: "Default P2P Port",
    detail: "Peer discovery and gossip traffic use the node's default external listener.",
  },
  {
    value: "8545",
    label: "Default RPC Port",
    detail: "JSON-RPC reads and writes are exposed locally for wallets, scripts, and dashboards.",
  },
  {
    value: "9090",
    label: "Default Monitoring Port",
    detail: "Prometheus-style metrics and health data are available through the monitoring service.",
  },
  {
    value: "8546",
    label: "Default WebSocket Port",
    detail: "New blocks, transactions, proposals, and slashing events can be streamed to clients.",
  },
];

function TechnologyBoard() {
  return (
    <div className="surface-card overflow-hidden">
      <div className="border-b border-border/70 px-6 py-5">
        <p className="eyebrow">Runtime Layout</p>
        <h2 className="mt-3 font-heading text-3xl text-foreground text-balance">
          A small module map with clear boundaries.
        </h2>
      </div>
      <div className="grid gap-3 px-6 py-6">
        {[
          { label: "Runtime config", detail: "TOML file plus environment overrides", icon: Database },
          { label: "Structured logs", detail: "Tracing-based visibility with configurable levels", icon: Shield },
          { label: "Persistent state", detail: "sled storage for local durability", icon: Server },
          { label: "Operator access", detail: "RPC, WebSocket, monitoring, explorer", icon: Globe },
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

export function TechnologyPage() {
  return (
    <div>
      <SEO
        title="NetChain Technology | Rust Node, libp2p, and Runtime Interfaces"
        description="Review NetChain's technical architecture across chain state, Proof of Internet modules, libp2p networking, sled storage, wallet tooling, and front-end explorer routes."
        keywords="NetChain technology, Rust blockchain, libp2p, sled, WebSocket, JSON-RPC, monitoring"
      />

      <PageHero
        eyebrow="Technical Architecture"
        title="A full-stack protocol prototype organized for inspection."
        description="NetChain is built as a Rust node with clearly separated domains for chain logic, networking, node services, Proof of Internet scoring, configuration, and wallet tooling. The website adds the operator-facing layer on top."
        primaryAction={{ label: "Read the Docs", to: "/docs" }}
        secondaryAction={{ label: "Run the Node", to: "/get-started" }}
        metrics={[
          { label: "Language", value: "Rust 2021 across node and wallet binaries" },
          { label: "Networking", value: "libp2p plus JSON-RPC, WebSocket, and health services" },
          { label: "Storage", value: "sled-backed local persistence for runtime data" },
          { label: "Front End", value: "React and Vite for the website plus explorer route" },
        ]}
        aside={<TechnologyBoard />}
      />

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Module Map"
            title="The codebase keeps protocol concerns in explicit domains."
            description="That structure matters because the experimental logic is easy to inspect: Proof of Internet is not hidden in general infrastructure, and operator services are not mixed into chain rules."
          />
          <InsightGrid items={moduleCards} columns={3} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/40">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Runtime Interfaces"
            title="Local services are part of the default developer workflow."
            description="The repository exposes distinct interfaces for peer traffic, application reads, health inspection, and event subscriptions. That separation helps testing, tooling, and local operations."
          />
          <StatGrid items={technologyStats} />
        </div>
      </section>

      <CtaBanner
        eyebrow="Next Layer"
        title="Move from architecture to governance behavior."
        description="The governance page explains how proposal validation, stake-weighted voting, quorum, and runtime parameter changes work inside the same execution model."
        primaryAction={{ label: "Open Governance", to: "/governance" }}
        secondaryAction={{ label: "Open Docs", to: "/docs" }}
      />
    </div>
  );
}
