import {
  Blocks,
  Network,
  Server,
  Shield,
  Wallet,
  Globe,
  Sparkles,
  Cpu,
  Zap,
  Code,
  Terminal,
} from "lucide-react";
import { SEO } from "@/components/seo";
import { Card, CardContent } from "@/components/ui/card";
import { FadeIn } from "@/components/ui/fade-in";
import { SectionHeader } from "@/components/sections/section-header";
import { SectionBackground } from "@/components/sections/section-background";
import { CtaSection } from "@/components/sections/cta-section";

const architecture = [
  { icon: Blocks, name: "Chain Layer", path: "src/chain", description: "Core blockchain primitives: blocks, transactions, state validation, and ledger rules.", details: "Defines the canonical execution model and state transitions.", color: "primary" },
  { icon: Network, name: "Networking Layer", path: "src/net", description: "libp2p networking, JSON-RPC, WebSocket events, and monitoring handlers.", details: "Peer traffic and operator visibility share a coherent surface.", color: "accent" },
  { icon: Server, name: "Node Layer", path: "src/node", description: "Mempool coordination, block production, and sled-backed persistent storage.", details: "Runtime services stay separate from chain rules and network plumbing.", color: "tertiary" },
  { icon: Shield, name: "Consensus Layer", path: "src/poi", description: "Proof of Internet scoring, metric aggregation, attestations, and anti-gaming.", details: "The experimental protocol is isolated and inspectable.", color: "primary" },
  { icon: Wallet, name: "Wallet Layer", path: "src/wallet", description: "Wallet helpers, encrypted key storage, and CLI interaction with protocol state.", details: "First-party wallet path alongside the node.", color: "accent" },
  { icon: Globe, name: "Application Layer", path: "website/", description: "Marketing site and explorer dashboard provide full-stack inspection.", details: "Protocol narrative and live runtime reads stay connected.", color: "tertiary" },
];

const endpoints = [
  { port: "30333", name: "P2P", description: "Peer discovery and gossip traffic", color: "primary" },
  { port: "8545", name: "RPC", description: "JSON-RPC for wallets and scripts", color: "accent" },
  { port: "9090", name: "Monitoring", description: "Prometheus metrics and health", color: "tertiary" },
  { port: "8546", name: "WebSocket", description: "Real-time event streaming", color: "primary" },
];

const techStack = [
  { name: "Rust 2021", description: "Core implementation", icon: Code },
  { name: "libp2p", description: "Peer-to-peer networking", icon: Network },
  { name: "sled", description: "Embedded database", icon: Server },
  { name: "tokio", description: "Async runtime", icon: Zap },
  { name: "Ed25519", description: "Digital signatures", icon: Shield },
  { name: "AES-256-GCM", description: "Encryption", icon: Wallet },
];

const poiSteps = [
  { title: "Performance Measurement", content: "Validators continuously report network metrics: download speed, upload speed, latency, uptime, and stability. These measurements form the basis of selection weight." },
  { title: "Peer Attestation", content: "Metrics aren't self-reported blindly. Other validators observe and attest to each node's performance, creating a trust network that's hard to game." },
  { title: "Composite Scoring", content: "The protocol blends measured performance with stake amount, reputation history, and identity verification to compute a final trust score." },
  { title: "Anti-Gaming Protections", content: "Statistical analysis, challenge-response verification, and gradual reputation building prevent metric manipulation." },
];

export function TechnologyPage() {
  return (
    <>
      <SEO
        title="Technology - NetChain"
        description="Explore NetChain's technical architecture: Rust-based implementation, modular design, and Proof of Internet consensus."
      />

      {/* Hero Section */}
      <section className="relative pt-32 pb-24 overflow-hidden">
        <SectionBackground variant="gradient" />
        <div className="absolute inset-0 bg-grid-fine opacity-30" />

        <div className="container-wide relative z-10">
          <FadeIn direction="up">
            <div className="max-w-4xl">
              <SectionHeader
                badge={{ label: "Technology", icon: Cpu }}
                title="Engineered for Performance"
                highlight="Performance"
                description="A deep dive into NetChain's architecture, from the consensus layer to the runtime interfaces that power the network."
                align="left"
                className="mb-0"
              />
            </div>
          </FadeIn>
        </div>
      </section>

      {/* Architecture Overview */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="subtle" />

        <div className="container-wide relative z-10">
          <SectionHeader
            badge={{ label: "Architecture", icon: Blocks }}
            title="Modular Architecture"
            highlight="Architecture"
            description="Clean separation of concerns enables maintainability, testing, and future evolution."
          />

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-5">
            {architecture.map((layer, index) => (
              <FadeIn key={layer.name} delay={index * 80} direction="up">
                <Card variant="default" size="md" className="h-full group">
                  <CardContent className="p-6">
                    <div className="flex items-start gap-4">
                      <div className={`flex-shrink-0 w-12 h-12 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform duration-500 ${layer.color === 'primary' ? 'bg-primary/15 text-primary border border-primary/20' : ''} ${layer.color === 'accent' ? 'bg-accent/15 text-accent border border-accent/20' : ''} ${layer.color === 'tertiary' ? 'bg-tertiary/15 text-tertiary border border-tertiary/20' : ''}`}>
                        <layer.icon className="w-6 h-6" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="text-lg font-semibold mb-1 text-foreground">{layer.name}</h3>
                        <code className="text-xs text-primary bg-primary/10 px-2 py-0.5 rounded font-mono inline-block mb-3">{layer.path}</code>
                        <p className="text-sm text-muted-foreground mb-2 leading-relaxed">{layer.description}</p>
                        <p className="text-xs text-muted-foreground/70 italic">{layer.details}</p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </FadeIn>
            ))}
          </div>
        </div>
      </section>

      {/* Runtime Interfaces */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="gradient" />

        <div className="container-wide relative z-10">
          <div className="grid lg:grid-cols-2 gap-16 items-center">
            <FadeIn direction="left">
              <SectionHeader
                badge={{ label: "Runtime Interfaces", icon: Server }}
                title="Clear Access Points"
                highlight="Access Points"
                description="NetChain exposes distinct interfaces for peer traffic, application reads, health inspection, and event subscriptions."
                align="left"
                className="mb-10"
              />

              <div className="grid grid-cols-2 gap-4">
                {endpoints.map((endpoint, index) => (
                  <FadeIn key={endpoint.port} delay={index * 100}>
                    <div className="p-5 rounded-xl bg-surface-elevated border border-border hover:border-primary/30 transition-all duration-300 text-center group">
                      <div className={`text-3xl font-bold font-mono mb-2 transition-colors ${endpoint.color === 'primary' ? 'text-primary' : ''} ${endpoint.color === 'accent' ? 'text-accent' : ''} ${endpoint.color === 'tertiary' ? 'text-tertiary' : ''}`}>
                        :{endpoint.port}
                      </div>
                      <div className="text-sm font-semibold text-foreground mb-1">{endpoint.name}</div>
                      <div className="text-xs text-muted-foreground">{endpoint.description}</div>
                    </div>
                  </FadeIn>
                ))}
              </div>
            </FadeIn>

            {/* Terminal Preview */}
            <FadeIn direction="right">
              <div className="rounded-xl overflow-hidden border border-border bg-code-bg shadow-2xl">
                <div className="flex items-center justify-between px-4 py-3 bg-surface-elevated border-b border-border">
                  <div className="flex gap-2">
                    <div className="w-3 h-3 rounded-full bg-red-500/80" />
                    <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
                    <div className="w-3 h-3 rounded-full bg-green-500/80" />
                  </div>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground font-mono">
                    <Terminal className="w-3.5 h-3.5" />
                    netchain.log
                  </div>
                </div>
                <div className="p-5 font-mono text-xs leading-loose space-y-1">
                  <div className="text-muted-foreground"><span className="text-tertiary">INFO</span> <span className="text-muted-foreground/60">netchain:</span> using data directory: ./data</div>
                  <div className="text-muted-foreground"><span className="text-tertiary">INFO</span> <span className="text-muted-foreground/60">netchain::net:</span> P2P listening on /ip4/0.0.0.0/tcp/<span className="text-primary">30333</span></div>
                  <div className="text-muted-foreground"><span className="text-tertiary">INFO</span> <span className="text-muted-foreground/60">netchain::net:</span> RPC server listening on 127.0.0.1:<span className="text-primary">8545</span></div>
                  <div className="text-muted-foreground"><span className="text-tertiary">INFO</span> <span className="text-muted-foreground/60">netchain::net:</span> WebSocket server on 127.0.0.1:<span className="text-primary">8546</span></div>
                  <div className="text-muted-foreground"><span className="text-tertiary">INFO</span> <span className="text-muted-foreground/60">netchain::net:</span> Monitoring on 127.0.0.1:<span className="text-primary">9090</span></div>
                  <div className="text-muted-foreground"><span className="text-accent">INFO</span> <span className="text-muted-foreground/60">netchain::poi:</span> PoI aggregator initialized</div>
                  <div className="text-muted-foreground"><span className="text-accent">INFO</span> <span className="text-muted-foreground/60">netchain::node:</span> Block producer started</div>
                  <div className="pt-3 mt-3 border-t border-border"><span className="text-tertiary">INFO</span> <span className="text-muted-foreground/60">netchain:</span> Node ready. Validator mode: <span className="text-primary font-semibold">active</span></div>
                </div>
              </div>
            </FadeIn>
          </div>
        </div>
      </section>

      {/* Tech Stack */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="subtle" />

        <div className="container-wide relative z-10">
          <SectionHeader
            badge={{ label: "Stack", icon: Code }}
            title="Built With"
            highlight="With"
            description="Industry-standard technologies chosen for reliability and performance."
          />

          <div className="flex flex-wrap justify-center gap-4">
            {techStack.map((tech, index) => (
              <FadeIn key={tech.name} delay={index * 80}>
                <div className="flex items-center gap-4 px-6 py-4 rounded-xl bg-surface-elevated border border-border hover:border-primary/30 transition-all duration-300 group">
                  <div className="w-10 h-10 rounded-lg bg-primary/10 border border-primary/20 text-primary flex items-center justify-center group-hover:scale-110 transition-transform">
                    <tech.icon className="w-5 h-5" />
                  </div>
                  <div>
                    <span className="font-semibold text-foreground">{tech.name}</span>
                    <span className="text-muted-foreground ml-2 text-sm">— {tech.description}</span>
                  </div>
                </div>
              </FadeIn>
            ))}
          </div>
        </div>
      </section>

      {/* Proof of Internet Deep Dive */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="gradient" />

        <div className="container-wide relative z-10">
          <div className="max-w-4xl mx-auto">
            <SectionHeader
              badge={{ label: "Core Innovation", icon: Sparkles }}
              title="Proof of Internet: The Breakthrough"
              highlight="The Breakthrough"
              description="Understanding the consensus mechanism that sets NetChain apart."
            />

            <div className="space-y-4">
              {poiSteps.map((item, index) => (
                <FadeIn key={item.title} delay={index * 100} direction="up">
                  <div className="flex gap-5 p-6 rounded-xl bg-surface-elevated border border-border hover:border-primary/30 transition-all duration-300 group">
                    <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-gradient-to-br from-primary to-accent text-white flex items-center justify-center font-bold text-lg group-hover:scale-110 transition-transform">
                      {index + 1}
                    </div>
                    <div>
                      <h3 className="font-semibold text-lg mb-2 text-foreground">{item.title}</h3>
                      <p className="text-muted-foreground leading-relaxed">{item.content}</p>
                    </div>
                  </div>
                </FadeIn>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* CTA */}
      <CtaSection
        badge={{ label: "Open Source" }}
        title="Explore the Codebase"
        description="NetChain is 100% open source. Dive into the implementation details on GitHub."
        primaryAction={{ label: "View on GitHub", href: "https://github.com/netchain" }}
        secondaryAction={{ label: "Read Documentation", href: "/docs" }}
      />
    </>
  );
}
