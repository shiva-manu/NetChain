import { Link } from "react-router-dom";
import {
  Activity,
  ArrowRight,
  Blocks,
  BookOpen,
  Code,
  Copy,
  CheckCircle2,
  FileCode,
  Globe,
  Server,
  Terminal,
  Wallet,
  Wifi,
  Zap,
} from "lucide-react";
import { useState } from "react";

import { SEO } from "@/components/seo";
import { FadeIn } from "@/components/ui/fade-in";
import { SectionHeader } from "@/components/sections/section-header";
import { SectionBackground } from "@/components/sections/section-background";
import { CtaSection } from "@/components/sections/cta-section";

const commandSections = [
  {
    title: "Build & Check",
    icon: Code,
    commands: [
      { cmd: "cargo build", desc: "Compile debug binaries" },
      { cmd: "cargo build --release", desc: "Compile optimized release binaries" },
      { cmd: "cargo check --all-targets", desc: "Type-check without building" },
      { cmd: "cargo fmt --all -- --check", desc: "Verify code formatting" },
    ],
  },
  {
    title: "Run",
    icon: Zap,
    commands: [
      { cmd: "cargo run --bin netchain", desc: "Start the blockchain node" },
      { cmd: "cargo run --bin netchain-wallet", desc: "Launch the wallet CLI" },
      { cmd: "NETCHAIN_CONFIG=./config/default.toml cargo run --bin netchain", desc: "Run with explicit config" },
    ],
  },
  {
    title: "Test",
    icon: Terminal,
    commands: [
      { cmd: "cargo test --all-targets", desc: "Run full test suite" },
      { cmd: "cargo test test_empty_merkle_root", desc: "Run specific test by name" },
      { cmd: "cargo test -- --exact test_name", desc: "Exact match test name" },
    ],
  },
];

const interfaces = [
  {
    icon: Globe,
    name: "JSON-RPC",
    port: "8545",
    description: "Query balances, submit transactions, fetch blocks and chain info",
    methods: ["get_balance", "send_transaction", "get_block", "get_chain_info"],
    color: "primary",
  },
  {
    icon: Wifi,
    name: "WebSocket",
    port: "8546",
    description: "Subscribe to real-time events for blocks, transactions, and proposals",
    methods: ["new_blocks", "new_transactions", "proposals", "slashing"],
    color: "accent",
  },
  {
    icon: Activity,
    name: "Monitoring",
    port: "9090",
    description: "Health checks and Prometheus-compatible metrics endpoint",
    methods: ["/health", "/metrics"],
    color: "tertiary",
  },
];

const quickLinks = [
  {
    icon: FileCode,
    title: "Architecture",
    description: "Understand the modular design and PoI consensus",
    href: "/technology",
  },
  {
    icon: Blocks,
    title: "Features",
    description: "Explore core capabilities and protocol features",
    href: "/features",
  },
  {
    icon: Server,
    title: "Explorer",
    description: "Inspect live chain state in the dashboard",
    href: "/dashboard",
  },
];

function CommandBlock({ cmd, desc }: { cmd: string; desc: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="group flex w-full min-w-0 flex-col gap-3 rounded-xl border border-border bg-code-bg px-4 py-3 transition-all duration-300 hover:border-primary/30 hover:bg-surface-hover sm:flex-row sm:items-center sm:justify-between">
      <div className="flex min-w-0 flex-1 items-start gap-3 overflow-hidden">
        <span className="mt-0.5 shrink-0 text-primary/70">$</span>
        <code className="block min-w-0 break-all font-mono text-[0.8125rem] leading-6 text-foreground/80 sm:truncate sm:text-sm sm:leading-5 sm:whitespace-nowrap">
          {cmd}
        </code>
      </div>
      <div className="flex items-center gap-3 self-end sm:self-auto">
        <span className="hidden max-w-[12rem] truncate text-xs text-muted-foreground sm:block">
          {desc}
        </span>
        <button
          onClick={handleCopy}
          className="shrink-0 rounded-lg p-2 text-muted-foreground opacity-100 transition-all hover:bg-foreground/10 hover:text-primary sm:opacity-0 sm:group-hover:opacity-100"
          aria-label="Copy command"
        >
          {copied ? (
            <CheckCircle2 className="h-3.5 w-3.5 text-tertiary" />
          ) : (
            <Copy className="h-3.5 w-3.5" />
          )}
        </button>
      </div>
    </div>
  );
}

function InterfaceCard({
  icon: Icon,
  name,
  port,
  description,
  methods,
  color,
  index,
}: {
  icon: typeof Globe;
  name: string;
  port: string;
  description: string;
  methods: string[];
  color: string;
  index: number;
}) {
  const colorClasses: Record<string, string> = {
    primary: "from-primary/20 to-primary/5 text-primary border-primary/30",
    accent: "from-accent/20 to-accent/5 text-accent border-accent/30",
    tertiary: "from-tertiary/20 to-tertiary/5 text-tertiary border-tertiary/30",
  };

  const iconBg: Record<string, string> = {
    primary: "bg-gradient-to-br from-primary/20 to-primary/5 text-primary",
    accent: "bg-gradient-to-br from-accent/20 to-accent/5 text-accent",
    tertiary: "bg-gradient-to-br from-tertiary/20 to-tertiary/5 text-tertiary",
  };

  return (
    <FadeIn delay={index * 0.1}>
      <div className="group relative overflow-hidden rounded-xl border border-border bg-card p-6 transition-all duration-500 hover:border-primary/30 hover:bg-surface-hover">
        <div className={`absolute inset-0 bg-gradient-to-br ${colorClasses[color]} opacity-0 transition-opacity duration-500 group-hover:opacity-100`} />

        <div className="relative">
          <div className="mb-5 flex items-start justify-between">
            <div className={`flex h-12 w-12 items-center justify-center rounded-xl ${iconBg[color]} transition-transform duration-300 group-hover:scale-110`}>
              <Icon className="h-6 w-6" />
            </div>
            <code className="rounded-lg bg-code-bg px-3 py-1.5 font-mono text-xs text-muted-foreground">
              :{port}
            </code>
          </div>
          <h3 className="mb-2 text-lg font-semibold text-foreground">{name}</h3>
          <p className="mb-5 text-sm text-muted-foreground">{description}</p>
          <div className="flex flex-wrap gap-2">
            {methods.map((method) => (
              <span
                key={method}
                className="rounded-full bg-secondary/50 px-3 py-1 font-mono text-xs text-muted-foreground transition-colors group-hover:bg-secondary group-hover:text-foreground/80"
              >
                {method}
              </span>
            ))}
          </div>
        </div>
      </div>
    </FadeIn>
  );
}

export function DocsPage() {
  return (
    <div className="relative min-h-screen">
      <SEO
        title="Documentation | NetChain"
        description="Developer documentation for NetChain. Build, run, and interact with the blockchain node using JSON-RPC, WebSocket, and monitoring interfaces."
        keywords="NetChain docs, blockchain documentation, JSON-RPC, WebSocket, cargo commands"
      />

      {/* Hero Section */}
      <section className="relative overflow-hidden pt-32 pb-20">
        <SectionBackground variant="gradient" />
        <div className="absolute inset-0 bg-grid-fine opacity-30" />

        <div className="container-wide relative z-10">
          <FadeIn className="mx-auto max-w-3xl text-center">
            <SectionHeader
              badge={{ label: "Developer Documentation", icon: BookOpen }}
              title="Build and run NetChain"
              highlight="NetChain"
              description="Everything you need to build the node, interact with the network, and monitor protocol activity. Simple Cargo commands, explicit interfaces, no magic."
            />
          </FadeIn>

          {/* Quick Reference Card */}
          <FadeIn delay={0.2} className="mx-auto mt-12 max-w-2xl">
            <div className="rounded-xl border border-border bg-card p-6">
              <div className="mb-5 flex items-center gap-2 text-sm text-muted-foreground">
                <Terminal className="h-4 w-4 text-primary" />
                <span>Quick Reference</span>
              </div>
              <div className="grid gap-3 sm:grid-cols-2">
                {[
                  { label: "Node binary", value: "netchain" },
                  { label: "Wallet binary", value: "netchain-wallet" },
                  { label: "JSON-RPC", value: "127.0.0.1:8545" },
                  { label: "WebSocket", value: "127.0.0.1:8546" },
                ].map((item) => (
                  <div
                    key={item.label}
                    className="flex flex-col items-start gap-1 rounded-xl bg-code-bg px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
                  >
                    <span className="text-sm text-muted-foreground">{item.label}</span>
                    <code className="max-w-full break-all font-mono text-sm text-primary">
                      {item.value}
                    </code>
                  </div>
                ))}
              </div>
            </div>
          </FadeIn>
        </div>
      </section>

      {/* Commands Section */}
      <section className="relative py-20">
        <SectionBackground variant="subtle" />

        <div className="container-wide relative z-10">
          <FadeIn className="mb-12">
            <SectionHeader
              badge={{ label: "Commands", icon: Code }}
              title="Standard Cargo workflow"
              highlight="Cargo"
              description="No custom build tools or scripts. The repository follows conventional Rust patterns for building, testing, and running the node."
              align="left"
              className="mb-0"
            />
          </FadeIn>

          <div className="grid gap-8 lg:grid-cols-3">
            {commandSections.map((section, sectionIndex) => (
              <FadeIn key={section.title} delay={sectionIndex * 0.1}>
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 border border-primary/20 text-primary">
                      <section.icon className="h-5 w-5" />
                    </div>
                    <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                      {section.title}
                    </h3>
                  </div>
                  <div className="space-y-2">
                    {section.commands.map((cmd) => (
                      <CommandBlock key={cmd.cmd} cmd={cmd.cmd} desc={cmd.desc} />
                    ))}
                  </div>
                </div>
              </FadeIn>
            ))}
          </div>
        </div>
      </section>

      {/* Interfaces Section */}
      <section className="relative py-20">
        <SectionBackground variant="gradient" />

        <div className="container-wide relative z-10">
          <FadeIn className="mb-12">
            <SectionHeader
              badge={{ label: "Interfaces", icon: Server }}
              title="Protocol access points"
              highlight="Access"
              description="The node exposes three primary interfaces. All bind to localhost by default, configurable via TOML or environment variables."
              align="left"
              className="mb-0"
            />
          </FadeIn>

          <div className="grid gap-6 md:grid-cols-3">
            {interfaces.map((iface, index) => (
              <InterfaceCard key={iface.name} {...iface} index={index} />
            ))}
          </div>
        </div>
      </section>

      {/* Configuration Section */}
      <section className="relative py-20">
        <SectionBackground variant="subtle" />

        <div className="container-wide relative z-10">
          <div className="grid gap-12 lg:grid-cols-2">
            <FadeIn>
              <SectionHeader
                badge={{ label: "Configuration", icon: Wallet }}
                title="TOML config with environment overrides"
                highlight="Config"
                description=""
                align="left"
                className="mb-0"
              />
              <p className="mt-4 text-muted-foreground">
                The node loads <code className="text-primary bg-primary/10 px-1.5 py-0.5 rounded font-mono text-xs">config/default.toml</code> by
                default. Every setting can be overridden via environment variables, making
                deployments scriptable while keeping local development explicit.
              </p>

              <div className="mt-8 space-y-4">
                <h3 className="flex items-center gap-2 text-sm font-semibold uppercase tracking-wider text-foreground">
                  <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                  Key Environment Variables
                </h3>
                <div className="space-y-2">
                  {[
                    { name: "DATA_DIR", desc: "Node data directory" },
                    { name: "RPC_PORT", desc: "JSON-RPC bind port" },
                    { name: "NETCHAIN_WS_PORT", desc: "WebSocket bind port" },
                    { name: "NETCHAIN_BLOCK_INTERVAL_SECS", desc: "Block production interval" },
                    { name: "NETCHAIN_LOG_LEVEL", desc: "Logging verbosity" },
                  ].map((env) => (
                    <div
                      key={env.name}
                      className="flex flex-col items-start gap-2 rounded-xl border border-border bg-card px-4 py-3 transition-all duration-300 hover:border-primary/30 hover:bg-surface-hover sm:flex-row sm:items-center sm:justify-between"
                    >
                      <code className="max-w-full break-all font-mono text-sm text-primary">
                        {env.name}
                      </code>
                      <span className="text-sm text-muted-foreground sm:text-right">
                        {env.desc}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </FadeIn>

            <FadeIn delay={0.2}>
              <div className="rounded-xl border border-border bg-card p-6">
                <div className="mb-4 flex items-center gap-2 text-sm text-muted-foreground">
                  <FileCode className="h-4 w-4 text-primary" />
                  <span>config/default.toml</span>
                </div>

                <div className="overflow-hidden rounded-xl border border-border bg-code-bg">
                  <div className="flex items-center gap-2 border-b border-border bg-surface-elevated px-4 py-2">
                    <div className="h-3 w-3 rounded-full bg-red-500/60" />
                    <div className="h-3 w-3 rounded-full bg-yellow-500/60" />
                    <div className="h-3 w-3 rounded-full bg-green-500/60" />
                  </div>

                  <pre className="overflow-x-auto p-4 text-sm">
                    <code className="text-foreground/70">
{`# Network configuration
[network]
p2p_port = 9000
rpc_port = 8545
ws_port = 8546
monitoring_port = 9090

# Consensus settings
[consensus]
block_interval_secs = 10
max_validators = 100

# Storage
[storage]
data_dir = "./data"

# Logging
log_level = "info"`}
                    </code>
                  </pre>
                </div>
              </div>
            </FadeIn>
          </div>
        </div>
      </section>

      {/* Quick Links Section */}
      <section className="relative py-20">
        <div className="container-wide">
          <FadeIn className="mb-12 text-center">
            <h2 className="text-2xl font-bold text-foreground">Continue exploring</h2>
            <p className="mt-2 text-muted-foreground">
              Dive deeper into architecture, features, or start exploring live data
            </p>
          </FadeIn>

          <div className="mx-auto grid max-w-3xl gap-4 md:grid-cols-3">
            {quickLinks.map((link, index) => (
              <FadeIn key={link.title} delay={index * 0.1}>
                <Link
                  to={link.href}
                  className="group flex flex-col rounded-xl border border-border bg-card p-6 transition-all duration-300 hover:border-primary/40 hover:bg-surface-hover hover:shadow-lg hover:shadow-primary/5"
                >
                  <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 border border-primary/20 text-primary transition-all duration-300 group-hover:scale-110 group-hover:bg-primary/20">
                    <link.icon className="h-6 w-6" />
                  </div>
                  <h3 className="mb-1 font-semibold text-foreground group-hover:text-primary transition-colors">{link.title}</h3>
                  <p className="mb-4 flex-1 text-sm text-muted-foreground">{link.description}</p>
                  <div className="flex items-center gap-1 text-sm text-primary">
                    <span>Learn more</span>
                    <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover:translate-x-1" />
                  </div>
                </Link>
              </FadeIn>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <CtaSection
        badge={{ label: "Ready" }}
        title="Ready to run the node?"
        description="Follow the quickstart guide to build the binaries, launch the node, and start exploring the protocol in minutes."
        primaryAction={{ label: "Get Started", href: "/get-started" }}
        secondaryAction={{ label: "Open Explorer", href: "/dashboard" }}
      />
    </div>
  );
}
