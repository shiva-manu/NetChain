import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const techStack = [
  {
    category: "Core Runtime",
    icon: "⚙️",
    color: "from-cyan-500 to-blue-500",
    items: [
      { name: "Rust", detail: "Systems programming language", highlight: true },
      { name: "Tokio", detail: "Async multi-threaded runtime" },
      { name: "sled", detail: "Embedded persistent database" },
    ],
  },
  {
    category: "Networking",
    icon: "🌐",
    color: "from-emerald-500 to-teal-500",
    items: [
      { name: "libp2p", detail: "P2P gossip & discovery", highlight: true },
      { name: "mDNS", detail: "Local peer discovery" },
      { name: "Noise + Yamux", detail: "Encrypted multiplexed transport" },
    ],
  },
  {
    category: "Cryptography",
    icon: "🔐",
    color: "from-violet-500 to-purple-500",
    items: [
      { name: "Ed25519", detail: "Transaction signing", highlight: true },
      { name: "SHA-256", detail: "Block hashing" },
      { name: "Argon2 + AES-GCM", detail: "Wallet encryption" },
    ],
  },
  {
    category: "Interfaces",
    icon: "🔌",
    color: "from-orange-500 to-amber-500",
    items: [
      { name: "JSON-RPC", detail: "Port 8545", highlight: true },
      { name: "WebSocket", detail: "Port 8546 event streaming" },
      { name: "Prometheus", detail: "Port 9090 metrics" },
    ],
  },
];

const architecture = [
  { file: "consensus.rs", description: "Hybrid trust scoring & validator selection", category: "PoI" },
  { file: "measurement.rs", description: "Internet measurement logic", category: "PoI" },
  { file: "metrics_aggregator.rs", description: "Attestation, reputation, and epoch aggregation", category: "PoI" },
  { file: "state.rs", description: "Stake, governance, and slashing state", category: "Chain" },
  { file: "anti_gaming.rs", description: "Validation & anti-abuse checks", category: "PoI" },
  { file: "blockchain.rs", description: "Chain validation & sync", category: "Chain" },
  { file: "p2p.rs", description: "libp2p networking & gossip", category: "Net" },
  { file: "rpc.rs", description: "JSON-RPC server", category: "Net" },
  { file: "websocket.rs", description: "Real-time event streaming", category: "Net" },
];

const categoryColors: Record<string, string> = {
  PoI: "bg-primary/10 text-primary border-primary/30",
  Chain: "bg-accent/10 text-accent border-accent/30",
  Net: "bg-violet-500/10 text-violet-400 border-violet-500/30",
};

export function Technology() {
  return (
    <section id="technology" className="relative py-24 sm:py-32">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        <div className="absolute right-0 top-1/4 h-[500px] w-[500px] rounded-full bg-accent/5 blur-[120px]" />
        <div className="absolute bottom-0 left-0 h-[400px] w-[400px] rounded-full bg-primary/5 blur-[100px]" />
      </div>

      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-3xl text-center">
          <span className="mb-4 inline-block text-sm font-semibold uppercase tracking-wider text-primary">
            Technology
          </span>
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl lg:text-5xl" style={{ textWrap: "balance" }}>
            Built With{" "}
            <span className="text-gradient">Modern Technology</span>
          </h2>
          <p className="mt-6 text-lg leading-relaxed text-muted-foreground">
            A carefully chosen stack for performance, reliability, and hybrid
            consensus telemetry.
          </p>
        </div>

        {/* Tech Stack Grid - Bento style */}
        <div className="mx-auto mt-16 grid max-w-6xl grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {techStack.map((group, groupIndex) => (
            <div
              key={group.category}
              className={cn(
                "group relative overflow-hidden rounded-2xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm",
                "transition-all duration-500 hover:border-border hover:bg-card/50 hover:shadow-xl",
                "opacity-0 animate-fade-in-up"
              )}
              style={{ 
                animationDelay: `${groupIndex * 100}ms`,
                animationFillMode: "forwards"
              }}
            >
              {/* Category header */}
              <div className="mb-5 flex items-center gap-3">
                <span className="text-2xl" aria-hidden="true">{group.icon}</span>
                <h3 
                  className={cn(
                    "text-sm font-semibold uppercase tracking-wider",
                    "bg-gradient-to-r bg-clip-text text-transparent",
                    group.color
                  )}
                >
                  {group.category}
                </h3>
              </div>

              {/* Items */}
              <div className="space-y-4">
                {group.items.map((item) => (
                  <div
                    key={item.name}
                    className="group/item flex items-baseline gap-3 transition-colors"
                  >
                    <span 
                      className={cn(
                        "font-mono text-sm font-semibold",
                        item.highlight ? "text-foreground" : "text-foreground/80"
                      )}
                    >
                      {item.name}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {item.detail}
                    </span>
                  </div>
                ))}
              </div>

              {/* Gradient glow */}
              <div 
                className={cn(
                  "pointer-events-none absolute -bottom-20 -right-20 h-40 w-40 rounded-full blur-3xl opacity-0 transition-opacity duration-500 group-hover:opacity-30",
                  "bg-gradient-to-br",
                  group.color
                )}
                aria-hidden="true"
              />
            </div>
          ))}
        </div>

        {/* Architecture */}
        <div className="mx-auto mt-20 max-w-5xl">
          <div className="mb-8 text-center">
            <h3 className="text-2xl font-bold text-foreground">
              Project Architecture
            </h3>
            <p className="mt-2 text-muted-foreground">
              Modular Rust codebase organized by domain
            </p>
          </div>

          <div className="overflow-hidden rounded-2xl border border-border/50 bg-card/30 backdrop-blur-sm">
            {/* Terminal header */}
            <div className="flex items-center gap-2 border-b border-border/50 bg-muted/30 px-4 py-3">
              <div className="flex gap-1.5">
                <div className="size-3 rounded-full bg-red-500/80" />
                <div className="size-3 rounded-full bg-yellow-500/80" />
                <div className="size-3 rounded-full bg-green-500/80" />
              </div>
              <span className="ml-2 font-mono text-xs text-muted-foreground">
                src/
              </span>
            </div>

            {/* File list */}
            <div className="grid grid-cols-1 gap-px bg-border/30 sm:grid-cols-2 lg:grid-cols-3">
              {architecture.map((mod, index) => (
                <div
                  key={mod.file}
                  className={cn(
                    "group flex items-center gap-3 bg-card/50 px-4 py-3 transition-colors hover:bg-muted/30",
                    "opacity-0 animate-fade-in-up"
                  )}
                  style={{ 
                    animationDelay: `${400 + index * 50}ms`,
                    animationFillMode: "forwards"
                  }}
                >
                  <Badge
                    variant="outline"
                    className={cn(
                      "shrink-0 border font-mono text-[10px]",
                      categoryColors[mod.category]
                    )}
                  >
                    {mod.category}
                  </Badge>
                  <div className="min-w-0 flex-1">
                    <span className="block truncate font-mono text-sm font-medium text-foreground">
                      {mod.file}
                    </span>
                    <span className="block truncate text-xs text-muted-foreground">
                      {mod.description}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* GitHub link */}
          <div className="mt-8 text-center">
            <a
              href="https://github.com/shiva-manu/NetChain"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 text-sm font-medium text-primary transition-colors hover:text-primary/80"
            >
              <span>Explore the full source on GitHub</span>
              <svg className="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
                <path d="M7 17L17 7M17 7H7M17 7V17" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
