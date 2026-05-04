import {
  ArrowRight,
  Zap,
  Shield,
  Globe,
  Network,
  CheckCircle2,
  Github,
  Play,
  Terminal,
  ChevronRight,
  Activity,
  Lock,
  Layers,
  Sparkles,
} from "lucide-react";
import { SEO } from "@/components/seo";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { FadeIn } from "@/components/ui/fade-in";
import { NetworkVisualization } from "@/components/ui/network-visualization";
import { SectionHeader } from "@/components/sections/section-header";
import { SectionBackground } from "@/components/sections/section-background";

const howItWorksSteps = [
  {
    icon: Activity,
    step: "01",
    title: "Measure",
    description: "Continuous monitoring of download speed, upload bandwidth, latency, and uptime metrics",
  },
  {
    icon: Lock,
    step: "02",
    title: "Verify",
    description: "Peer attestation ensures all reported metrics are accurate and tamper-proof",
  },
  {
    icon: Layers,
    step: "03",
    title: "Select",
    description: "Best-performing nodes earn higher validator weight and block production priority",
  },
  {
    icon: Sparkles,
    step: "04",
    title: "Reward",
    description: "Fair distribution of rewards proportional to actual network contribution",
  },
];

const codeSnippets = [
  { label: "Install CLI", command: "cargo install netchain", delay: 0 },
  { label: "Initialize Node", command: "netchain init --mainnet", delay: 600 },
  { label: "Start Validator", command: "netchain start --validator", delay: 1200 },
];

export function HomePage() {
  return (
    <>
      <SEO
        title="NetChain - Next Generation Blockchain Infrastructure"
        description="NetChain introduces Proof of Internet — a revolutionary consensus mechanism that validates real network performance. Experience the future of decentralized infrastructure."
      />

      {/* Hero Section */}
      <section className="relative min-h-[100vh] flex items-center pt-20 overflow-hidden">
        <SectionBackground variant="gradient" />
        <div className="absolute inset-0 bg-grid-fine opacity-30" />

        <div className="container-wide relative z-10 py-24">
          <div className="grid lg:grid-cols-2 gap-16 items-center">
            <div className="text-left">
              <FadeIn direction="down" delay={0}>
                <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-primary/10 border border-primary/20 text-primary text-sm font-medium mb-8">
                  <Sparkles className="w-4 h-4" />
                  <span>Live on Testnet v2.0</span>
                </div>
              </FadeIn>

              <FadeIn direction="up" delay={100}>
                <h1 className="text-6xl md:text-7xl lg:text-8xl font-bold tracking-tight mb-8 leading-[1.1]">
                  Future of
                  <br />
                  <span className="text-gradient">Blockchain</span>
                  <br />
                  Redefined.
                </h1>
              </FadeIn>

              <FadeIn direction="up" delay={200}>
                <p className="text-lg md:text-xl text-muted-foreground max-w-xl mb-12 leading-relaxed">
                  NetChain introduces <span className="text-foreground font-semibold">Proof of Internet</span> —
                  a revolutionary consensus mechanism that validates real network performance for a truly decentralized world.
                </p>
              </FadeIn>

              <FadeIn direction="up" delay={300}>
                <div className="flex flex-col sm:flex-row items-center gap-4">
                  <Button
                    variant="premium"
                    size="xl"
                    rightIcon={<ArrowRight className="w-5 h-5" />}
                    href="/get-started"
                    className="w-full sm:w-auto px-10"
                  >
                    Start Building
                  </Button>
                  <Button
                    variant="glass"
                    size="xl"
                    leftIcon={<Play className="w-5 h-5 fill-current" />}
                    href="/dashboard"
                    className="w-full sm:w-auto px-10"
                  >
                    Explorer
                  </Button>
                </div>
              </FadeIn>

              <FadeIn direction="up" delay={400}>
                <div className="flex items-center gap-8 mt-16 opacity-60 hover:opacity-100 transition-all duration-500">
                  <div className="flex items-center gap-2">
                    <Github className="w-5 h-5" />
                    <span className="font-semibold">Open Source</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Shield className="w-5 h-5" />
                    <span className="font-semibold">Secure</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Zap className="w-5 h-5" />
                    <span className="font-semibold">Fast</span>
                  </div>
                </div>
              </FadeIn>
            </div>

            <FadeIn direction="left" delay={500} className="relative hidden lg:block">
              <div className="relative z-10 rounded-[2.5rem] shadow-2xl rotate-3 hover:rotate-0 transition-transform duration-700 overflow-hidden border border-border">
                <div className="bg-[#050505] rounded-[2rem] overflow-hidden aspect-[4/3] relative group">
                  <div className="absolute inset-0">
                    <NetworkVisualization className="opacity-40" />
                  </div>

                  <div className="absolute top-8 left-8 flex items-center gap-2">
                    <Activity className="w-5 h-5 text-primary" />
                    <span className="font-bold text-sm tracking-tight text-foreground/90 uppercase">Dashboard</span>
                  </div>

                  <div className="absolute bottom-8 left-8 right-8 grid grid-cols-2 gap-4">
                    <div className="bg-foreground/5 backdrop-blur-sm border border-foreground/10 p-5 rounded-2xl">
                      <div className="text-[10px] text-muted-foreground uppercase tracking-widest mb-1.5 font-bold">Total Throughput</div>
                      <div className="text-2xl font-bold flex items-baseline gap-1.5">
                        10,000+ <span className="text-primary text-xs uppercase font-medium">TPS</span>
                      </div>
                    </div>
                    <div className="bg-foreground/5 backdrop-blur-sm border border-foreground/10 p-5 rounded-2xl">
                      <div className="text-[10px] text-muted-foreground uppercase tracking-widest mb-1.5 font-bold">Block Finality</div>
                      <div className="text-2xl font-bold flex items-baseline gap-1.5">
                        {"<"} 1.0 <span className="text-accent text-xs uppercase font-medium">SEC</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div className="absolute -inset-4 bg-primary/10 blur-3xl rounded-[3rem] -z-10" />
            </FadeIn>
          </div>
        </div>
      </section>

      {/* Features Bento Grid */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="subtle" />
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full h-px bg-gradient-to-r from-transparent via-border to-transparent" />

        <div className="container-wide relative z-10">
          <FadeIn direction="up">
            <SectionHeader
              badge={{ label: "Next Generation Infrastructure", icon: Sparkles }}
              title="Built for the Limitless Future"
              highlight="Limitless Future"
              description="Every component engineered for maximum performance, military-grade security, and an exceptional developer experience."
            />
          </FadeIn>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Bento Box 1 - Main Feature */}
            <FadeIn delay={0} direction="up" className="lg:col-span-2">
              <Card variant="premium" className="h-full min-h-[400px] group">
                <div className="absolute top-0 right-0 w-64 h-64 bg-primary/10 blur-[80px] group-hover:bg-primary/20 transition-colors" />
                <CardHeader className="relative z-10 p-8">
                  <div className="w-14 h-14 rounded-2xl bg-primary/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-500">
                    <Zap className="w-7 h-7 text-primary" />
                  </div>
                  <CardTitle className="text-3xl mb-4">Proof of Internet</CardTitle>
                  <CardDescription className="text-lg max-w-md">
                    Revolutionary consensus mechanism that validates real-world network performance.
                    True decentralization based on contribution, not just capital.
                  </CardDescription>
                </CardHeader>
                <div className="mt-auto p-8 pt-0 relative z-10">
                  <div className="flex items-center gap-8 opacity-60 group-hover:opacity-100 transition-all duration-500">
                    <div className="text-center">
                      <div className="text-2xl font-bold">100+</div>
                      <div className="text-xs text-muted-foreground uppercase tracking-widest">Active Nodes</div>
                    </div>
                    <div className="text-center">
                      <div className="text-2xl font-bold">99.9%</div>
                      <div className="text-xs text-muted-foreground uppercase tracking-widest">Uptime</div>
                    </div>
                  </div>
                </div>
              </Card>
            </FadeIn>

            {/* Bento Box 2 - Security */}
            <FadeIn delay={100} direction="up">
              <Card variant="glass" className="h-full group">
                <CardHeader className="p-8">
                  <div className="w-12 h-12 rounded-xl bg-accent/10 flex items-center justify-center mb-6 group-hover:rotate-12 transition-transform">
                    <Shield className="w-6 h-6 text-accent" />
                  </div>
                  <CardTitle className="text-xl mb-3">Enterprise Security</CardTitle>
                  <CardDescription>
                    Ed25519 signatures with AES-256-GCM encryption protecting every single byte on the chain.
                  </CardDescription>
                </CardHeader>
              </Card>
            </FadeIn>

            {/* Bento Box 3 - Scale */}
            <FadeIn delay={200} direction="up">
              <Card variant="glass" className="h-full group">
                <CardHeader className="p-8">
                  <div className="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-6 group-hover:-translate-y-1 transition-transform">
                    <Globe className="w-6 h-6 text-primary" />
                  </div>
                  <CardTitle className="text-xl mb-3">Global Scale</CardTitle>
                  <CardDescription>
                    Intelligent peer discovery and optimized P2P gossip enabling worldwide deployment with sub-second latency.
                  </CardDescription>
                </CardHeader>
              </Card>
            </FadeIn>

            {/* Bento Box 4 - Governance */}
            <FadeIn delay={300} direction="up" className="lg:col-span-2">
              <Card variant="premium" className="h-full group overflow-hidden">
                <div className="absolute -bottom-10 -right-10 w-64 h-64 bg-accent/10 blur-[80px] group-hover:bg-accent/20 transition-colors" />
                <div className="grid md:grid-cols-2 gap-8 items-center p-8">
                  <div>
                    <div className="w-14 h-14 rounded-2xl bg-accent/10 flex items-center justify-center mb-6">
                      <Network className="w-7 h-7 text-accent" />
                    </div>
                    <CardTitle className="text-3xl mb-4">On-chain Governance</CardTitle>
                    <CardDescription className="text-lg">
                      Community-driven protocol upgrades through transparent voting and proposal systems. Your voice, codified.
                    </CardDescription>
                  </div>
                  <div className="relative">
                    <div className="bg-card/80 backdrop-blur-sm p-6 rounded-xl border border-border">
                      <div className="flex items-center justify-between mb-4">
                        <span className="font-bold">Upgrade Proposal #42</span>
                        <span className="px-2 py-0.5 rounded-full bg-tertiary/20 text-tertiary text-[10px] font-bold uppercase">Active</span>
                      </div>
                      <div className="h-2 w-full bg-muted rounded-full overflow-hidden mb-2">
                        <div className="h-full bg-accent w-[75%]" />
                      </div>
                      <div className="flex justify-between text-xs text-muted-foreground font-bold">
                        <span>75% YES</span>
                        <span>25% NO</span>
                      </div>
                    </div>
                  </div>
                </div>
              </Card>
            </FadeIn>
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="py-24 relative overflow-hidden">
        <div className="container-wide relative z-10">
          <div className="grid lg:grid-cols-2 gap-16 lg:gap-24 items-center">
            <FadeIn direction="right">
              <SectionHeader
                badge={{ label: "Protocol Lifecycle", icon: Activity }}
                title="Proof of Internet: A New Trust Layer"
                highlight="New Trust Layer"
                description="Unlike traditional PoS or PoW, NetChain measures actual network performance. Validators are selected based on their demonstrated ability to serve the network reliably."
                align="left"
                className="mb-10"
              />

              <div className="grid gap-4">
                {howItWorksSteps.map((step, index) => (
                  <FadeIn key={step.title} delay={index * 100} direction="right">
                    <div className="group flex gap-5 p-5 rounded-xl bg-card border border-border hover:border-primary/30 transition-all duration-500">
                      <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center text-primary group-hover:scale-110 transition-transform">
                        <step.icon className="w-6 h-6" />
                      </div>
                      <div className="flex-1">
                        <h4 className="font-bold text-lg mb-1">{step.title}</h4>
                        <p className="text-muted-foreground text-sm leading-relaxed">{step.description}</p>
                      </div>
                    </div>
                  </FadeIn>
                ))}
              </div>

              <div className="mt-10">
                <Button
                  variant="outline"
                  size="lg"
                  rightIcon={<ArrowRight className="w-5 h-5" />}
                  href="/technology"
                >
                  Deep Dive Technology
                </Button>
              </div>
            </FadeIn>

            <FadeIn direction="left" className="relative">
              <div className="relative aspect-square max-w-lg mx-auto">
                <div className="absolute inset-0 bg-primary/5 rounded-full blur-3xl animate-pulse" />
                <div className="relative z-10 w-full h-full rounded-2xl flex items-center justify-center overflow-hidden bg-card border border-border">
                  <div className="absolute inset-0 bg-grid-fine opacity-20" />
                  <Network className="w-32 h-32 text-primary/40" />

                  <div className="absolute top-10 left-10 bg-foreground/5 backdrop-blur-sm border border-foreground/10 p-4 rounded-xl">
                    <div className="flex items-center gap-2 text-tertiary font-bold">
                      <CheckCircle2 className="w-4 h-4" />
                      <span>Verified</span>
                    </div>
                  </div>
                  <div className="absolute bottom-20 right-10 bg-foreground/5 backdrop-blur-sm border border-foreground/10 p-4 rounded-xl">
                    <div className="text-xs text-muted-foreground font-bold uppercase mb-1">Latency</div>
                    <div className="text-xl font-bold">12ms</div>
                  </div>
                </div>
              </div>
            </FadeIn>
          </div>
        </div>
      </section>

      {/* Developer CTA */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="subtle" />

        <div className="container-wide relative z-10">
          <div className="grid lg:grid-cols-2 gap-16 lg:gap-24 items-center">
            <FadeIn direction="left">
              <div className="relative group">
                <div className="absolute -inset-1 bg-gradient-to-r from-primary to-accent rounded-2xl blur opacity-20 group-hover:opacity-40 transition duration-1000" />
                <div className="relative rounded-xl overflow-hidden border border-border bg-card">
                  <div className="flex items-center justify-between px-6 py-4 border-b border-border bg-surface-elevated">
                    <div className="flex gap-2">
                      <div className="w-3 h-3 rounded-full bg-destructive/50" />
                      <div className="w-3 h-3 rounded-full bg-warning/50" />
                      <div className="w-3 h-3 rounded-full bg-tertiary/50" />
                    </div>
                    <span className="text-xs font-medium text-muted-foreground tracking-widest uppercase">zsh — netchain</span>
                  </div>

                  <div className="p-8 font-mono text-sm space-y-8 bg-code-bg">
                    {codeSnippets.map((snippet, index) => (
                      <FadeIn key={index} delay={snippet.delay} direction="left">
                        <div>
                          <div className="text-muted-foreground/40 mb-2 text-xs italic">
                            {"# "}{snippet.label}
                          </div>
                          <div className="flex items-center gap-3">
                            <span className="text-primary font-bold">$</span>
                            <span className="text-foreground/90">{snippet.command}</span>
                            <span className="w-2 h-5 bg-primary/40 animate-pulse ml-1" />
                          </div>
                        </div>
                      </FadeIn>
                    ))}

                    <FadeIn delay={1800} direction="up">
                      <div className="pt-6 border-t border-border">
                        <div className="flex items-center gap-3 text-tertiary font-semibold">
                          <CheckCircle2 className="w-4 h-4" />
                          <span>Node operational on cluster mainnet-01</span>
                        </div>
                      </div>
                    </FadeIn>
                  </div>
                </div>
              </div>
            </FadeIn>

            <FadeIn direction="right">
              <SectionHeader
                badge={{ label: "Developer-First Protocol", icon: Terminal }}
                title="Built for Builders, By Builders"
                highlight="Builders"
                description="NetChain is completely open-source and built for performance. Our focus is on providing the most robust, developer-friendly infrastructure for the next generation of decentralized applications."
                align="left"
                className="mb-10"
              />

              <div className="flex flex-wrap gap-4">
                <Button
                  variant="default"
                  size="lg"
                  leftIcon={<Github className="w-5 h-5" />}
                  href="https://github.com/netchain"
                  className="px-8"
                >
                  View on GitHub
                </Button>
                <Button
                  variant="ghost"
                  size="lg"
                  rightIcon={<ChevronRight className="w-5 h-5" />}
                  href="/docs"
                >
                  Read Documentation
                </Button>
              </div>
            </FadeIn>
          </div>
        </div>
      </section>
    </>
  );
}
