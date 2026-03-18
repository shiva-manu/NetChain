import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

const techStack = [
  {
    category: "Core Runtime",
    items: [
      { name: "Rust", detail: "Systems programming language" },
      { name: "Tokio", detail: "Async multi-threaded runtime" },
      { name: "sled", detail: "Embedded persistent database" },
    ],
  },
  {
    category: "Networking",
    items: [
      { name: "libp2p", detail: "P2P gossip & discovery" },
      { name: "mDNS", detail: "Local peer discovery" },
      { name: "Noise + Yamux", detail: "Encrypted multiplexed transport" },
    ],
  },
  {
    category: "Cryptography",
    items: [
      { name: "Ed25519", detail: "Transaction signing" },
      { name: "SHA-256", detail: "Block hashing" },
      { name: "Argon2 + AES-GCM", detail: "Wallet encryption" },
    ],
  },
  {
    category: "Interfaces",
    items: [
      { name: "JSON-RPC", detail: "Port 8545" },
      { name: "WebSocket", detail: "Port 8546 event streaming" },
      { name: "Prometheus", detail: "Port 9090 metrics" },
    ],
  },
];

const architecture = [
  {
    file: "consensus.rs",
    description: "Hybrid trust scoring & validator selection",
  },
  { file: "measurement.rs", description: "Internet measurement logic" },
  {
    file: "metrics_aggregator.rs",
    description: "Attestation, reputation, and epoch aggregation",
  },
  { file: "state.rs", description: "Stake, governance, and slashing state" },
  { file: "anti_gaming.rs", description: "Validation & anti-abuse checks" },
  { file: "blockchain.rs", description: "Chain validation & sync" },
  { file: "p2p.rs", description: "libp2p networking & gossip" },
  { file: "rpc.rs", description: "JSON-RPC server" },
  { file: "websocket.rs", description: "Real-time event streaming" },
];

export function Technology() {
  return (
    <section id="technology" className="py-20 sm:py-28">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
            Built With Modern Technology
          </h2>
          <p className="mt-4 text-lg text-muted-foreground">
            A carefully chosen stack for performance, reliability, and hybrid
            consensus telemetry.
          </p>
        </div>

        {/* Tech Stack Grid */}
        <div className="mx-auto mt-16 grid max-w-5xl grid-cols-1 gap-6 sm:grid-cols-2">
          {techStack.map((group) => (
            <Card
              key={group.category}
              className="border-border/50 bg-card/50"
            >
              <CardContent className="p-6">
                <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-primary">
                  {group.category}
                </h3>
                <div className="space-y-3">
                  {group.items.map((item) => (
                    <div key={item.name} className="flex items-baseline gap-3">
                      <span className="font-mono text-sm font-medium text-foreground">
                        {item.name}
                      </span>
                      <span className="text-sm text-muted-foreground">
                        {item.detail}
                      </span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Architecture */}
        <div className="mx-auto mt-16 max-w-3xl">
          <h3 className="mb-6 text-center text-xl font-semibold text-foreground">
            Project Architecture
          </h3>
          <div className="rounded-xl border border-border/50 bg-card/30 p-6">
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              {architecture.map((mod) => (
                <div
                  key={mod.file}
                  className="flex items-center gap-3 rounded-lg px-3 py-2 transition-colors hover:bg-muted/50"
                >
                  <Badge
                    variant="secondary"
                    className="shrink-0 font-mono text-xs"
                  >
                    {mod.file}
                  </Badge>
                  <span className="text-sm text-muted-foreground">
                    {mod.description}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
