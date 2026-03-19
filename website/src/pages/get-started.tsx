import { Blocks, Terminal, Wallet, Wifi } from "lucide-react";

import { ChecklistGrid, CommandGrid, CtaBanner, InsightGrid, PageHero, SectionHeading } from "@/components/marketing/page-primitives";
import { SEO } from "@/components/seo";

const quickStartSteps = [
  {
    title: "Build the repository",
    description:
      "Compile the Rust binaries first so the node and wallet surfaces are available locally before you begin testing the network model.",
  },
  {
    title: "Run the node",
    description:
      "Start `netchain` with the default configuration or an explicit config path, then verify the RPC, WebSocket, and monitoring ports are reachable.",
  },
  {
    title: "Inspect the runtime",
    description:
      "Use the explorer route, JSON-RPC, or health and metrics endpoints to confirm that the node is exposing the expected consensus and governance data.",
  },
];

const startCommands = [
  {
    label: "Build the Binaries",
    command: `cargo build`,
    description:
      "This produces the main node binary and the wallet CLI so the local environment matches the repository's default workflow.",
  },
  {
    label: "Run the Node",
    command: `cargo run --bin netchain`,
    description:
      "Start the blockchain node with the default configuration and open the monitoring and explorer surfaces in parallel.",
  },
  {
    label: "Use an Explicit Config",
    command: `NETCHAIN_CONFIG=./config/default.toml cargo run --bin netchain`,
    description:
      "This is the most direct way to make runtime assumptions explicit when you are comparing environment or parameter changes.",
  },
  {
    label: "Run the Wallet CLI",
    command: `cargo run --bin netchain-wallet`,
    description:
      "Use the wallet binary when you want a first-party command-line path for signing and interacting with protocol state.",
  },
];

const environmentCards = [
  {
    icon: Terminal,
    eyebrow: "Configuration",
    title: "Default config file plus env overrides",
    description:
      "The node loads `config/default.toml` automatically and supports environment overrides for data directories, bind addresses, ports, block timing, block reward, and log level.",
    meta: "Useful overrides include `RPC_PORT`, `NETCHAIN_WS_PORT`, and `NETCHAIN_BLOCK_INTERVAL_SECS`.",
  },
  {
    icon: Wifi,
    eyebrow: "Interfaces",
    title: "Check the local access points immediately",
    description:
      "After the node starts, inspect `127.0.0.1:8545` for JSON-RPC, `127.0.0.1:8546` for WebSocket events, and `127.0.0.1:9090` for health and metrics.",
    meta: "That confirms the runtime is observable before you begin transaction or governance testing.",
  },
  {
    icon: Wallet,
    eyebrow: "Operator Tooling",
    title: "Use both the wallet and explorer surfaces",
    description:
      "The repository ships a CLI wallet and a browser-based explorer route, so you can compare command-line and UI-level views of the same runtime state.",
    meta: "This is useful when validating proposal status, balances, or telemetry deltas.",
  },
  {
    icon: Blocks,
    eyebrow: "Testing",
    title: "Keep the full suite close at hand",
    description:
      "Use `cargo test --all-targets` after significant changes so the redesign work stays grounded in the same repository discipline as the protocol code.",
    meta: "CI uses format, check, and test gates on every push and pull request.",
  },
];

function StartBoard() {
  return (
    <div className="surface-card overflow-hidden">
      <div className="border-b border-border/70 px-6 py-5">
        <p className="eyebrow">Local Bring-Up</p>
        <h2 className="mt-3 font-heading text-3xl text-foreground text-balance">
          The shortest path from clone to live node.
        </h2>
      </div>
      <div className="grid gap-3 px-6 py-6">
        {[
          "1. `cargo build`",
          "2. `cargo run --bin netchain`",
          "3. Open `/dashboard`",
          "4. Inspect `/health` and `/metrics`",
        ].map((item) => (
          <div key={item} className="rounded-[24px] border border-border/70 bg-secondary/55 px-5 py-4">
            <p className="font-mono text-sm text-foreground">{item}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

export function GetStartedPage() {
  return (
    <div>
      <SEO
        title="Get Started With NetChain | Run the Node and Inspect the Network"
        description="Build NetChain, run the Rust node, launch the wallet CLI, and inspect JSON-RPC, WebSocket, health, and metrics interfaces locally."
        keywords="get started NetChain, cargo run netchain, wallet CLI, local blockchain node, health metrics"
      />

      <PageHero
        eyebrow="Get Started"
        title="Bring the node up locally and inspect the full operator surface."
        description="NetChain does not require a complex bootstrap story. Build the binaries, run the node, check the local interfaces, and use the explorer route or wallet CLI to validate the runtime end to end."
        primaryAction={{ label: "Open the Docs", to: "/docs" }}
        secondaryAction={{ label: "Open the Explorer", to: "/dashboard" }}
        metrics={[
          { label: "Build Path", value: "Standard Cargo workflow" },
          { label: "Node Entry", value: "`cargo run --bin netchain`" },
          { label: "Wallet Entry", value: "`cargo run --bin netchain-wallet`" },
          { label: "Observability", value: "RPC, WebSocket, health, and metrics services" },
        ]}
        aside={<StartBoard />}
      />

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Quick Path"
            title="Three steps are enough for a credible local evaluation."
            description="The point is to get to a running system fast, then spend time on the protocol questions that matter: validator quality, governance execution, and telemetry fidelity."
          />
          <ChecklistGrid items={quickStartSteps} />
        </div>
      </section>

      <section className="section-band border-y border-border/60 bg-secondary/40">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Bring-Up Commands"
            title="Use the repository's default operational sequence."
            description="These commands cover the shortest route to building, running, and inspecting the node while staying consistent with the rest of the codebase."
          />
          <CommandGrid items={startCommands} />
        </div>
      </section>

      <section className="section-band">
        <div className="site-grid space-y-10">
          <SectionHeading
            eyebrow="Runtime Environment"
            title="Know what to inspect once the node is live."
            description="Configuration, interfaces, wallet tooling, and test discipline are the four things that matter immediately after local startup. They determine whether the protocol is actually inspectable, not just running."
          />
          <InsightGrid items={environmentCards} columns={2} />
        </div>
      </section>

      <CtaBanner
        eyebrow="Continue"
        title="Move from local bring-up to protocol inspection."
        description="Once the node is live, use the explorer route, WebSocket events, and governance screens to examine how the hybrid consensus model behaves under real local conditions."
        primaryAction={{ label: "Open Explorer", to: "/dashboard" }}
        secondaryAction={{ label: "Return Home", to: "/" }}
      />
    </div>
  );
}
