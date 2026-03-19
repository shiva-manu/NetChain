import { Activity, Blocks, Globe, Terminal, Wallet, Wifi } from "lucide-react";

import { CommandGrid, CtaBanner, InsightGrid, PageHero, SectionHeading } from "@/components/marketing/page-primitives";
import { SEO } from "@/components/seo";

const commandCards = [
  {
    label: "Build and Check",
    command: `cargo build
cargo check --all-targets
cargo fmt --all -- --check`,
    description:
      "Use the standard Rust workflow to build the binaries, type-check the repository, and verify formatting before making or reviewing changes.",
  },
  {
    label: "Run the Node",
    command: `cargo run --bin netchain
NETCHAIN_CONFIG=./config/default.toml cargo run --bin netchain`,
    description:
      "Start the node with the default configuration or provide an explicit config path through the environment to test runtime variants.",
  },
  {
    label: "Run the Wallet",
    command: `cargo run --bin netchain-wallet`,
    description:
      "The first-party CLI wallet is shipped alongside the node and is the simplest way to interact with signing and local key management flows.",
  },
  {
    label: "Run the Test Suite",
    command: `cargo test --all-targets
cargo test test_empty_merkle_root
cargo test -- --exact test_name`,
    description:
      "The repository supports full-suite execution as well as narrower test targeting for individual behaviors or exact test names.",
  },
];

const interfaceCards = [
  {
    icon: Globe,
    eyebrow: "JSON-RPC",
    title: "Local application reads and writes",
    description:
      "The default RPC server listens on `127.0.0.1:8545` and exposes account, balance, block, mempool, proposal, and transaction endpoints.",
    meta: "Key methods include `get_balance`, `send_transaction`, `get_block`, `get_proposals`, and `get_chain_info`.",
  },
  {
    icon: Wifi,
    eyebrow: "WebSocket",
    title: "Event streaming for live clients",
    description:
      "The WebSocket service runs on `127.0.0.1:8546` and supports topic subscriptions for new blocks, transactions, proposals, and slashing activity.",
    meta: "Subscriptions use JSON messages such as `{\"action\":\"subscribe\",\"topics\":[\"new_blocks\"]}`.",
  },
  {
    icon: Activity,
    eyebrow: "Monitoring",
    title: "Health and metrics endpoints",
    description:
      "The monitoring service listens on `127.0.0.1:9090` and exposes `GET /health` for JSON status and `GET /metrics` for Prometheus-style output.",
    meta: "The monitoring surface includes consensus mode, validator counts, and aggregate trust signals.",
  },
  {
    icon: Blocks,
    eyebrow: "Transactions",
    title: "Native protocol actions",
    description:
      "Transfer, stake, unstake, proposal creation, and proposal voting are first-class transaction types inside the chain state model.",
    meta: "Governance is not bolted on outside the execution layer.",
  },
  {
    icon: Wallet,
    eyebrow: "Configuration",
    title: "TOML plus environment overrides",
    description:
      "The node loads `config/default.toml` by default and supports overrides such as `DATA_DIR`, `RPC_PORT`, `NETCHAIN_BLOCK_INTERVAL_SECS`, and `NETCHAIN_LOG_LEVEL`.",
    meta: "That keeps local runs explicit while still making deployments scriptable.",
  },
  {
    icon: Terminal,
    eyebrow: "Deployment",
    title: "Docker and local Rust workflows",
    description:
      "Developers can build and run the project directly with Cargo or use Docker Compose for a containerized local environment.",
    meta: "The compose surface exposes P2P, RPC, and monitoring ports.",
  },
];

function DocsBoard() {
  return (
    <div className="surface-card overflow-hidden">
      <div className="border-b border-border/70 px-6 py-5">
        <p className="eyebrow">Reference Surface</p>
        <h2 className="mt-3 font-heading text-3xl text-foreground text-balance">
          Everything needed for a serious local evaluation.
        </h2>
      </div>
      <div className="grid gap-3 px-6 py-6">
        {[
          "Node binary: `netchain`",
          "Wallet binary: `netchain-wallet`",
          "JSON-RPC: `127.0.0.1:8545`",
          "WebSocket: `127.0.0.1:8546`",
          "Monitoring: `127.0.0.1:9090`",
        ].map((item) => (
          <div key={item} className="rounded-[24px] border border-border/70 bg-secondary/55 px-5 py-4">
            <p className="font-mono text-sm text-foreground">{item}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

export function DocsPage() {
  return (
    <div>
      <SEO
        title="NetChain Docs | Build, Run, and Inspect the Protocol"
        description="Use the NetChain documentation surface to build the node, run the wallet, inspect RPC and WebSocket interfaces, and work with monitoring endpoints."
        keywords="NetChain docs, cargo run netchain, wallet CLI, JSON-RPC, WebSocket, monitoring"
      />

      <PageHero
        eyebrow="Developer Documentation"
        title="Commands, interfaces, and runtime surfaces in one reference path."
        description="The repository is small enough to inspect directly but broad enough to need structure. This page collects the build, run, interface, and deployment entry points that matter when evaluating NetChain locally."
        primaryAction={{ label: "Get Started", to: "/get-started" }}
        secondaryAction={{ label: "Read the Technology", to: "/technology" }}
        metrics={[
          { label: "Build", value: "Cargo-based Rust workflow" },
          { label: "Inspect", value: "RPC, WebSocket, health, and metrics endpoints" },
          { label: "Operate", value: "Config file plus environment overrides" },
          { label: "Contribute", value: "CI checks include fmt, check, and test targets" },
        ]}
        aside={<DocsBoard />}
      />

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Core Commands"
            title="The build and local run workflow is intentionally direct."
            description="There is no complicated bootstrap path here. Cargo commands cover build, test, and runtime entry, while the wallet CLI and explorer route provide two immediate ways to interact with the node."
          />
          <CommandGrid items={commandCards} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/40">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Interface Reference"
            title="Protocol access points are explicit and easy to wire into tooling."
            description="Local RPC methods, streaming events, monitoring reads, and governance-aware transaction types give developers multiple ways to inspect or automate the runtime."
          />
          <InsightGrid items={interfaceCards} columns={3} />
        </div>
      </section>

      <CtaBanner
        eyebrow="Start Running"
        title="Use the local bring-up path and inspect the protocol while it is live."
        description="The get-started page turns the docs into an operational checklist: build the binaries, start the node, open the explorer, and verify the interfaces end to end."
        primaryAction={{ label: "Open Get Started", to: "/get-started" }}
        secondaryAction={{ label: "Open Explorer", to: "/dashboard" }}
      />
    </div>
  );
}
