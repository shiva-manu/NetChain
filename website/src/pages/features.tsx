import {
  Zap,
  Shield,
  Globe,
  BarChart3,
  Network,
  Lock,
  Server,
  Code,
  Wallet,
  Users,
  Check,
  Sparkles,
  Layers,
  Cpu,
} from "lucide-react";
import { SEO } from "@/components/seo";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { FadeIn } from "@/components/ui/fade-in";
import { SectionHeader } from "@/components/sections/section-header";
import { SectionBackground } from "@/components/sections/section-background";
import { CtaSection } from "@/components/sections/cta-section";

const coreFeatures = [
  {
    icon: Network,
    title: "Proof of Internet Consensus",
    description: "Revolutionary consensus mechanism that validates network performance metrics. Validators are selected based on actual delivery quality, not just stake.",
    highlights: ["Real-time performance monitoring", "Anti-gaming protections", "Fair validator selection"],
    gradient: "from-cyan-500 to-blue-500",
  },
  {
    icon: Zap,
    title: "High-Performance Runtime",
    description: "Built in Rust for maximum efficiency. Process thousands of transactions per second with sub-second finality.",
    highlights: ["10,000+ TPS capacity", "<1 second block time", "Optimized memory usage"],
    gradient: "from-violet-500 to-purple-500",
  },
  {
    icon: Shield,
    title: "Enterprise-Grade Security",
    description: "Military-grade cryptography protects every transaction. Ed25519 signatures and AES-256-GCM encryption ensure data integrity.",
    highlights: ["Ed25519 digital signatures", "AES-256-GCM encryption", "Hardware wallet support"],
    gradient: "from-emerald-500 to-teal-500",
  },
  {
    icon: Globe,
    title: "Global Network",
    description: "Designed for worldwide deployment with intelligent peer discovery and optimized gossip protocols.",
    highlights: ["Automatic peer discovery", "Geographic optimization", "Low-latency propagation"],
    gradient: "from-orange-500 to-amber-500",
  },
];

const technicalFeatures = [
  { icon: Server, title: "Modular Architecture", description: "Clean separation between chain, networking, node, and consensus layers enables easy customization." },
  { icon: Code, title: "Developer-First APIs", description: "Comprehensive JSON-RPC and WebSocket APIs with TypeScript SDK for seamless integration." },
  { icon: BarChart3, title: "Real-time Monitoring", description: "Built-in Prometheus metrics, health endpoints, and live WebSocket feeds for complete observability." },
  { icon: Wallet, title: "Native Wallet", description: "First-party CLI wallet with encrypted key storage, HD derivation, and transaction signing." },
  { icon: Users, title: "On-Chain Governance", description: "Proposal system with stake-weighted voting enables community-driven protocol evolution." },
  { icon: Lock, title: "Slashing Protection", description: "Built-in mechanisms detect and penalize malicious behavior, ensuring network integrity." },
];

const comparisonData = [
  { feature: "Consensus", netchain: "Proof of Internet", others: "PoW / PoS" },
  { feature: "Selection Criteria", netchain: "Network performance + stake", others: "Stake only" },
  { feature: "Gaming Resistance", netchain: "High (peer attestation)", others: "Low to Medium" },
  { feature: "Energy Efficiency", netchain: "Very High", others: "Low (PoW) / High (PoS)" },
  { feature: "True Decentralization", netchain: "Yes", others: "Stake concentration" },
  { feature: "Open Source", netchain: "100%", others: "Varies" },
];

export function FeaturesPage() {
  return (
    <>
      <SEO
        title="Features - NetChain"
        description="Explore NetChain's revolutionary features: Proof of Internet consensus, high-performance runtime, enterprise security, and more."
      />

      {/* Hero Section */}
      <section className="relative pt-32 pb-24 overflow-hidden">
        <SectionBackground variant="gradient" />
        <div className="absolute inset-0 bg-grid-fine opacity-30" />

        <div className="container-wide relative z-10">
          <FadeIn direction="up">
            <div className="max-w-4xl">
              <SectionHeader
                badge={{ label: "Features", icon: Sparkles }}
                title="Built for the Future of Decentralized Networks"
                highlight="Decentralized Networks"
                description="Every component of NetChain is engineered for performance, security, and developer experience. Discover what makes us different."
                align="left"
                className="mb-0"
              />
            </div>
          </FadeIn>
        </div>
      </section>

      {/* Core Features */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="subtle" />

        <div className="container-wide relative z-10">
          <SectionHeader
            badge={{ label: "Core Capabilities", icon: Layers }}
            title="Foundational Technologies"
            highlight="Technologies"
            description="The technologies that power NetChain's unique approach to blockchain."
          />

          <div className="grid lg:grid-cols-2 gap-6">
            {coreFeatures.map((feature, index) => (
              <FadeIn key={feature.title} delay={index * 100} direction="up">
                <Card variant="default" size="lg" className="h-full group">
                  <CardContent className="p-8">
                    <div className="flex items-start gap-6">
                      <div className={`flex-shrink-0 w-14 h-14 rounded-xl flex items-center justify-center transition-all duration-500 group-hover:scale-110 bg-gradient-to-br ${feature.gradient} p-[1px]`}>
                        <div className="w-full h-full rounded-[10px] bg-card flex items-center justify-center">
                          <feature.icon className="w-7 h-7 text-white" />
                        </div>
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="text-2xl font-semibold mb-3 text-foreground">{feature.title}</h3>
                        <p className="text-muted-foreground mb-5 leading-relaxed">{feature.description}</p>
                        <ul className="space-y-2.5">
                          {feature.highlights.map((highlight) => (
                            <li key={highlight} className="flex items-center gap-3">
                              <div className="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                                <Check className="w-3 h-3 text-primary" />
                              </div>
                              <span className="text-sm text-foreground/80">{highlight}</span>
                            </li>
                          ))}
                        </ul>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </FadeIn>
            ))}
          </div>
        </div>
      </section>

      {/* Technical Features Grid */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="gradient" />

        <div className="container-wide relative z-10">
          <SectionHeader
            badge={{ label: "Technical Excellence", icon: Cpu }}
            title="Built by Engineers, for Engineers"
            highlight="Engineers"
            description="Every detail considered for the best developer experience."
          />

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-5">
            {technicalFeatures.map((feature, index) => (
              <FadeIn key={feature.title} delay={index * 80} direction="up">
                <Card variant="glass" size="md" className="h-full group">
                  <CardHeader>
                    <div className="w-12 h-12 rounded-xl bg-primary/10 border border-primary/20 text-primary flex items-center justify-center mb-4 group-hover:scale-110 group-hover:bg-primary/20 transition-all duration-500">
                      <feature.icon className="w-6 h-6" />
                    </div>
                    <CardTitle className="text-lg">{feature.title}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <CardDescription className="text-base leading-relaxed">
                      {feature.description}
                    </CardDescription>
                  </CardContent>
                </Card>
              </FadeIn>
            ))}
          </div>
        </div>
      </section>

      {/* Comparison Table */}
      <section className="py-24 relative overflow-hidden">
        <SectionBackground variant="subtle" />

        <div className="container-wide relative z-10">
          <SectionHeader
            badge={{ label: "Comparison", icon: BarChart3 }}
            title="How We Compare"
            highlight="Compare"
            description="Proof of Internet consensus offers unique advantages over traditional approaches."
          />

          <FadeIn direction="up" delay={200}>
            <div className="max-w-4xl mx-auto">
              <div className="bg-surface-elevated border border-border rounded-2xl overflow-hidden">
                <div className="overflow-x-auto">
                  <table className="w-full">
                    <thead>
                      <tr className="bg-card border-b border-border">
                        <th className="text-left px-6 py-5 font-semibold text-foreground">Feature</th>
                        <th className="text-left px-6 py-5 font-semibold text-primary">NetChain</th>
                        <th className="text-left px-6 py-5 font-semibold text-muted-foreground">Others</th>
                      </tr>
                    </thead>
                    <tbody>
                      {comparisonData.map((row, index) => (
                        <tr
                          key={row.feature}
                          className={`border-b border-border/50 last:border-0 transition-colors hover:bg-primary/5 ${index % 2 === 0 ? "bg-transparent" : "bg-card/50"}`}
                        >
                          <td className="px-6 py-4 font-medium text-foreground">{row.feature}</td>
                          <td className="px-6 py-4">
                            <span className="inline-flex items-center gap-2 text-primary">
                              <Check className="w-4 h-4" />
                              {row.netchain}
                            </span>
                          </td>
                          <td className="px-6 py-4 text-muted-foreground">{row.others}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </FadeIn>
        </div>
      </section>

      {/* CTA Section */}
      <CtaSection
        badge={{ label: "Ready to Start" }}
        title="Ready to Experience NetChain?"
        description="Start building on the next generation of blockchain infrastructure today."
        primaryAction={{ label: "Get Started", href: "/get-started" }}
        secondaryAction={{ label: "View Technology", href: "/technology" }}
      />
    </>
  );
}
