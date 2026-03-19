import { useState } from "react";
import { Link } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Book,
  Server,
  Wallet,
  Code,
  Globe,
  Terminal,
  Zap,
  Shield,
  ChevronRight,
  ExternalLink,
  Copy,
  Check,
  ArrowRight,
} from "lucide-react";
import { cn } from "@/lib/utils";

// Sidebar navigation items
const navSections = [
  { id: "overview", label: "Overview", icon: Book },
  { id: "quickstart", label: "Quick Start", icon: Zap },
  { id: "node-setup", label: "Running a Node", icon: Server },
  { id: "validator", label: "Becoming a Validator", icon: Shield },
  { id: "wallet", label: "Wallet CLI", icon: Wallet },
  { id: "rpc-api", label: "RPC API", icon: Code },
  { id: "websocket", label: "WebSocket Events", icon: Globe },
  { id: "contributing", label: "Contributing", icon: Terminal },
];

// RPC methods
const rpcMethods = [
  {
    method: "chain_getInfo",
    description: "Get blockchain info (height, latest hash, peer count)",
    example: '{"jsonrpc":"2.0","method":"chain_getInfo","params":[],"id":1}',
  },
  {
    method: "chain_getBlock",
    description: "Get block by height",
    example:
      '{"jsonrpc":"2.0","method":"chain_getBlock","params":[0],"id":1}',
  },
  {
    method: "chain_getTransaction",
    description: "Get transaction by hash",
    example:
      '{"jsonrpc":"2.0","method":"chain_getTransaction","params":["<tx_hash>"],"id":1}',
  },
  {
    method: "chain_getBalance",
    description: "Get balance for an address",
    example:
      '{"jsonrpc":"2.0","method":"chain_getBalance","params":["<address>"],"id":1}',
  },
  {
    method: "tx_submit",
    description: "Submit a signed transaction",
    example:
      '{"jsonrpc":"2.0","method":"tx_submit","params":["<signed_tx_hex>"],"id":1}',
  },
  {
    method: "mempool_getPending",
    description: "Get pending transactions in mempool",
    example:
      '{"jsonrpc":"2.0","method":"mempool_getPending","params":[],"id":1}',
  },
  {
    method: "poi_getValidators",
    description: "Get current validator set with scores",
    example:
      '{"jsonrpc":"2.0","method":"poi_getValidators","params":[],"id":1}',
  },
  {
    method: "poi_getMetrics",
    description: "Get PoI metrics for an address",
    example:
      '{"jsonrpc":"2.0","method":"poi_getMetrics","params":["<address>"],"id":1}',
  },
];

// WebSocket events
const wsEvents = [
  {
    event: "new_block",
    description: "Emitted when a new block is added to the chain",
    payload: "{ height, hash, timestamp, tx_count }",
    color: "from-cyan-500 to-blue-500",
  },
  {
    event: "new_transaction",
    description: "Emitted when a transaction enters the mempool",
    payload: "{ hash, from, to, amount, tx_type }",
    color: "from-emerald-500 to-teal-500",
  },
  {
    event: "validator_update",
    description: "Emitted when validator set changes",
    payload: "{ validators: [...], epoch }",
    color: "from-violet-500 to-purple-500",
  },
  {
    event: "metrics_update",
    description: "Emitted periodically with network metrics",
    payload: "{ peer_count, tps, mempool_size }",
    color: "from-orange-500 to-amber-500",
  },
];

function CodeBlock({
  code,
  language = "bash",
}: {
  code: string;
  language?: string;
}) {
  const [copied, setCopied] = useState(false);

  const copyToClipboard = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="group relative overflow-hidden rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
      <div className="flex items-center justify-between border-b border-border/50 bg-muted/30 px-4 py-2">
        <div className="flex items-center gap-2">
          <div className="flex gap-1" aria-hidden="true">
            <div className="size-2.5 rounded-full bg-red-500/60" />
            <div className="size-2.5 rounded-full bg-yellow-500/60" />
            <div className="size-2.5 rounded-full bg-green-500/60" />
          </div>
          <span className="font-mono text-xs text-muted-foreground">
            {language}
          </span>
        </div>
        <button
          onClick={copyToClipboard}
          className="flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
          aria-label="Copy code"
        >
          {copied ? (
            <>
              <Check className="size-3 text-accent" />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <Copy className="size-3" />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>
      <pre className="overflow-x-auto p-4 text-sm">
        <code className="font-mono text-foreground/90">{code}</code>
      </pre>
    </div>
  );
}

export function Docs() {
  const [activeSection, setActiveSection] = useState("overview");

  const scrollToSection = (id: string) => {
    setActiveSection(id);
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  };

  return (
    <section className="relative py-16 sm:py-20">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        <div className="absolute left-0 top-1/4 h-[400px] w-[400px] rounded-full bg-primary/5 blur-[100px]" />
        <div className="absolute bottom-1/4 right-0 h-[300px] w-[300px] rounded-full bg-accent/5 blur-[100px]" />
      </div>

      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-12">
          <span className="mb-4 inline-block text-sm font-semibold uppercase tracking-wider text-primary">
            Documentation
          </span>
          <h1 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl lg:text-5xl">
            Developer{" "}
            <span className="text-gradient">Documentation</span>
          </h1>
          <p className="mt-4 max-w-3xl text-lg text-muted-foreground">
            Everything you need to run a NetChain node, become a validator,
            interact with the RPC API, and contribute to the project.
          </p>
        </div>

        <div className="flex flex-col gap-8 lg:flex-row">
          {/* Sidebar Navigation */}
          <aside className="lg:w-64 lg:shrink-0">
            <nav className="sticky top-24 space-y-1 rounded-xl border border-border/50 bg-card/30 p-3 backdrop-blur-sm">
              {navSections.map((section) => {
                const Icon = section.icon;
                return (
                  <button
                    key={section.id}
                    onClick={() => scrollToSection(section.id)}
                    className={cn(
                      "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left text-sm font-medium transition-all duration-200",
                      "focus-visible:outline-2 focus-visible:outline-ring",
                      activeSection === section.id
                        ? "bg-primary/10 text-primary"
                        : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                    )}
                  >
                    <Icon className="size-4" aria-hidden="true" />
                    {section.label}
                    <ChevronRight
                      className={cn(
                        "ml-auto size-4 transition-transform",
                        activeSection === section.id && "rotate-90 text-primary"
                      )}
                      aria-hidden="true"
                    />
                  </button>
                );
              })}
            </nav>
          </aside>

          {/* Main Content */}
          <main className="min-w-0 flex-1 space-y-20">
            {/* Overview */}
            <section id="overview" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">Overview</h2>
              <p className="mt-4 text-muted-foreground leading-relaxed">
                NetChain is a Layer-1 blockchain with{" "}
                <strong className="text-foreground">
                  Proof of Internet (PoI)
                </strong>{" "}
                consensus. Validator selection is weighted by measured internet
                performance metrics (download, upload, latency, uptime,
                stability) blended with stake.
              </p>
              <div className="mt-6 grid gap-4 sm:grid-cols-2">
                <Card className="border-border/50 bg-card/30 backdrop-blur-sm">
                  <CardContent className="p-5">
                    <h3 className="mb-3 font-semibold text-foreground">Live Endpoints</h3>
                    <ul className="space-y-2 text-sm text-muted-foreground">
                      <li className="flex items-center gap-2">
                        <span className="size-2 rounded-full bg-accent animate-pulse" aria-hidden="true" />
                        <strong className="text-foreground">RPC:</strong>{" "}
                        <code className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs">
                          https://api.netchain.me/rpc
                        </code>
                      </li>
                      <li className="flex items-center gap-2">
                        <span className="size-2 rounded-full bg-accent animate-pulse" aria-hidden="true" />
                        <strong className="text-foreground">WebSocket:</strong>{" "}
                        <code className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs">
                          wss://api.netchain.me/ws
                        </code>
                      </li>
                      <li className="flex items-center gap-2">
                        <span className="size-2 rounded-full bg-accent animate-pulse" aria-hidden="true" />
                        <strong className="text-foreground">Metrics:</strong>{" "}
                        <code className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs">
                          https://api.netchain.me/metrics
                        </code>
                      </li>
                    </ul>
                  </CardContent>
                </Card>
                <Card className="border-border/50 bg-card/30 backdrop-blur-sm">
                  <CardContent className="p-5">
                    <h3 className="mb-3 font-semibold text-foreground">Resources</h3>
                    <ul className="space-y-2 text-sm">
                      <li>
                        <a
                          href="https://github.com/shiva-manu/NetChain"
                          target="_blank"
                          rel="noopener noreferrer"
                          className="group inline-flex items-center gap-1.5 text-primary hover:underline"
                        >
                          GitHub Repository
                          <ExternalLink className="size-3 transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
                        </a>
                      </li>
                      <li>
                        <Link
                          to="/dashboard"
                          className="group inline-flex items-center gap-1.5 text-primary hover:underline"
                        >
                          Live Dashboard
                          <ArrowRight className="size-3 transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
                        </Link>
                      </li>
                    </ul>
                  </CardContent>
                </Card>
              </div>
            </section>

            {/* Quick Start */}
            <section id="quickstart" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">Quick Start</h2>
              <p className="mt-4 text-muted-foreground">
                Get a NetChain node running in under 5 minutes.
              </p>

              <div className="mt-6 space-y-6">
                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Prerequisites
                  </h3>
                  <ul className="list-inside list-disc space-y-1 text-muted-foreground marker:text-primary">
                    <li>Rust 1.75+ (install via rustup)</li>
                    <li>Git</li>
                    <li>Linux, macOS, or Windows with WSL</li>
                  </ul>
                </div>

                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Build & Run
                  </h3>
                  <CodeBlock
                    code={`# Clone the repository
git clone https://github.com/shiva-manu/NetChain.git
cd NetChain

# Build in release mode
cargo build --release

# Run the node
./target/release/netchain`}
                  />
                </div>

                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Using Docker
                  </h3>
                  <CodeBlock
                    code={`# Build and run with Docker Compose
docker compose up --build

# Or build manually
docker build -t netchain .
docker run -p 8545:8545 -p 8546:8546 -p 9090:9090 netchain`}
                  />
                </div>
              </div>
            </section>

            {/* Node Setup */}
            <section id="node-setup" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">
                Running a Node
              </h2>
              <p className="mt-4 text-muted-foreground">
                Configure and run a NetChain node for development or production.
              </p>

              <div className="mt-6 space-y-6">
                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Configuration
                  </h3>
                  <p className="mb-3 text-sm text-muted-foreground">
                    Configuration is loaded from{" "}
                    <code className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs text-foreground">
                      config/default.toml
                    </code>{" "}
                    and can be overridden with environment variables.
                  </p>
                  <CodeBlock
                    language="toml"
                    code={`# config/default.toml
[node]
data_dir = "./data"
log_level = "info"

[rpc]
enabled = true
bind_address = "0.0.0.0:8545"

[websocket]
enabled = true
bind_address = "0.0.0.0:8546"

[p2p]
listen_address = "/ip4/0.0.0.0/tcp/9000"
bootstrap_peers = []

[monitoring]
enabled = true
bind_address = "0.0.0.0:9090"`}
                  />
                </div>

                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Ports
                  </h3>
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="outline" className="border-primary/30 bg-primary/5 font-mono">
                      8545 — JSON-RPC
                    </Badge>
                    <Badge variant="outline" className="border-accent/30 bg-accent/5 font-mono">
                      8546 — WebSocket
                    </Badge>
                    <Badge variant="outline" className="border-violet-500/30 bg-violet-500/5 font-mono">
                      9000 — P2P
                    </Badge>
                    <Badge variant="outline" className="border-orange-500/30 bg-orange-500/5 font-mono">
                      9090 — Metrics
                    </Badge>
                  </div>
                </div>
              </div>
            </section>

            {/* Validator */}
            <section id="validator" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">
                Becoming a Validator
              </h2>
              <p className="mt-4 text-muted-foreground">
                NetChain uses Proof of Internet (PoI) consensus. Validators are
                selected based on a hybrid score combining stake and internet
                performance metrics.
              </p>

              <div className="mt-6 space-y-6">
                <Card className="border-border/50 bg-card/30 backdrop-blur-sm">
                  <CardContent className="p-5">
                    <h3 className="mb-4 text-lg font-semibold text-foreground">PoI Metrics</h3>
                    <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                      {[
                        { name: "Download", desc: "Bandwidth (Mbps)", color: "from-cyan-500 to-blue-500" },
                        { name: "Upload", desc: "Bandwidth (Mbps)", color: "from-emerald-500 to-teal-500" },
                        { name: "Latency", desc: "RTT to peers (ms)", color: "from-violet-500 to-purple-500" },
                        { name: "Uptime", desc: "Availability %", color: "from-orange-500 to-amber-500" },
                        { name: "Stability", desc: "Connection consistency", color: "from-pink-500 to-rose-500" },
                      ].map((metric) => (
                        <div key={metric.name} className="flex items-center gap-3 rounded-lg bg-muted/30 p-3">
                          <div className={cn("size-8 rounded-lg bg-gradient-to-br", metric.color)} />
                          <div>
                            <span className="block text-sm font-medium text-foreground">{metric.name}</span>
                            <span className="text-xs text-muted-foreground">{metric.desc}</span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>

                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Register as Validator
                  </h3>
                  <CodeBlock
                    code={`# Create a wallet first
./target/release/netchain-wallet create

# Stake tokens to register as validator
./target/release/netchain-wallet stake --amount 1000

# Check validator status
curl -X POST https://api.netchain.me/rpc \\
  -H "Content-Type: application/json" \\
  -d '{"jsonrpc":"2.0","method":"poi_getValidators","params":[],"id":1}'`}
                  />
                </div>
              </div>
            </section>

            {/* Wallet CLI */}
            <section id="wallet" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">Wallet CLI</h2>
              <p className="mt-4 text-muted-foreground">
                The{" "}
                <code className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs text-foreground">
                  netchain-wallet
                </code>{" "}
                CLI manages keys, balances, and transactions.
              </p>

              <div className="mt-6">
                <CodeBlock
                  code={`# Build the wallet CLI
cargo build --release --bin netchain-wallet

# Create a new wallet (encrypted with password)
./target/release/netchain-wallet create

# Show wallet address and balance
./target/release/netchain-wallet info

# Send tokens
./target/release/netchain-wallet send --to <address> --amount 100

# Stake tokens (become a validator)
./target/release/netchain-wallet stake --amount 1000

# Unstake tokens
./target/release/netchain-wallet unstake --amount 500`}
                />
                <p className="mt-4 text-sm text-muted-foreground">
                  Wallet keys are stored encrypted with AES-256-GCM. The password
                  is derived using Argon2id.
                </p>
              </div>
            </section>

            {/* RPC API */}
            <section id="rpc-api" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">RPC API</h2>
              <p className="mt-4 text-muted-foreground">
                NetChain exposes a JSON-RPC 2.0 API at{" "}
                <code className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs text-foreground">
                  https://api.netchain.me/rpc
                </code>
              </p>

              <div className="mt-6 space-y-4">
                {rpcMethods.map((method) => (
                  <Card key={method.method} className="border-border/50 bg-card/30 backdrop-blur-sm">
                    <CardContent className="p-5">
                      <code className="font-mono text-sm font-semibold text-primary">
                        {method.method}
                      </code>
                      <p className="mt-1 text-sm text-muted-foreground">
                        {method.description}
                      </p>
                      <div className="mt-3">
                        <CodeBlock language="json" code={method.example} />
                      </div>
                    </CardContent>
                  </Card>
                ))}

                <div className="mt-8">
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Example: Get Chain Info
                  </h3>
                  <CodeBlock
                    code={`curl -X POST https://api.netchain.me/rpc \\
  -H "Content-Type: application/json" \\
  -d '{"jsonrpc":"2.0","method":"chain_getInfo","params":[],"id":1}'

# Response:
{
  "jsonrpc": "2.0",
  "result": {
    "height": 12345,
    "latest_hash": "abc123...",
    "peer_count": 5,
    "mempool_size": 42
  },
  "id": 1
}`}
                  />
                </div>
              </div>
            </section>

            {/* WebSocket Events */}
            <section id="websocket" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">
                WebSocket Events
              </h2>
              <p className="mt-4 text-muted-foreground">
                Subscribe to real-time events via WebSocket at{" "}
                <code className="rounded bg-muted/50 px-1.5 py-0.5 font-mono text-xs text-foreground">
                  wss://api.netchain.me/ws
                </code>
              </p>

              <div className="mt-6 grid gap-4 sm:grid-cols-2">
                {wsEvents.map((event) => (
                  <Card key={event.event} className="border-border/50 bg-card/30 backdrop-blur-sm">
                    <CardContent className="p-5">
                      <div className="mb-3 flex items-center gap-2">
                        <div className={cn("size-3 rounded-full bg-gradient-to-br", event.color)} />
                        <code className="font-mono text-sm font-semibold text-foreground">
                          {event.event}
                        </code>
                      </div>
                      <p className="text-sm text-muted-foreground">
                        {event.description}
                      </p>
                      <p className="mt-2 font-mono text-xs text-muted-foreground">
                        {event.payload}
                      </p>
                    </CardContent>
                  </Card>
                ))}
              </div>

              <div className="mt-8">
                <h3 className="mb-3 text-lg font-semibold text-foreground">
                  JavaScript Example
                </h3>
                <CodeBlock
                  language="javascript"
                  code={`const ws = new WebSocket("wss://api.netchain.me/ws");

ws.onopen = () => {
  console.log("Connected to NetChain");
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch (data.type) {
    case "new_block":
      console.log("New block:", data.height);
      break;
    case "new_transaction":
      console.log("New tx:", data.hash);
      break;
  }
};

ws.onerror = (error) => {
  console.error("WebSocket error:", error);
};`}
                />
              </div>
            </section>

            {/* Contributing */}
            <section id="contributing" className="scroll-mt-24">
              <h2 className="text-2xl font-bold text-foreground">Contributing</h2>
              <p className="mt-4 text-muted-foreground">
                NetChain is open source. Contributions are welcome!
              </p>

              <div className="mt-6 space-y-6">
                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Development Setup
                  </h3>
                  <CodeBlock
                    code={`# Clone and build
git clone https://github.com/shiva-manu/NetChain.git
cd NetChain
cargo build

# Run tests
cargo test --all-targets

# Format code
cargo fmt --all

# Check lints
cargo check --all-targets`}
                  />
                </div>

                <div>
                  <h3 className="mb-3 text-lg font-semibold text-foreground">
                    Project Structure
                  </h3>
                  <CodeBlock
                    language="text"
                    code={`src/
├── main.rs          # Node binary entry point
├── lib.rs           # Library crate exports
├── chain/           # Block, blockchain, state, transaction
├── net/             # P2P, RPC, WebSocket, monitoring
├── node/            # Mempool, block producer, storage
├── poi/             # Consensus, measurement, anti-gaming
└── wallet/          # Wallet crypto & CLI`}
                  />
                </div>

                <Card className="border-primary/20 bg-gradient-to-br from-primary/5 to-accent/5 backdrop-blur-sm">
                  <CardContent className="p-6">
                    <h3 className="mb-2 font-semibold text-foreground">
                      Ready to contribute?
                    </h3>
                    <p className="text-sm text-muted-foreground">
                      Check out the{" "}
                      <a
                        href="https://github.com/shiva-manu/NetChain/issues"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-primary hover:underline"
                      >
                        open issues
                      </a>{" "}
                      or read the{" "}
                      <a
                        href="https://github.com/shiva-manu/NetChain/blob/main/AGENTS.md"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-primary hover:underline"
                      >
                        AGENTS.md
                      </a>{" "}
                      for code style guidelines.
                    </p>
                  </CardContent>
                </Card>
              </div>
            </section>
          </main>
        </div>
      </div>
    </section>
  );
}
