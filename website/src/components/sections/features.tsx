import { useRef } from "react";
import {
  Globe,
  Shield,
  Vote,
  Blocks,
  Wifi,
  Database,
  Activity,
  Lock,
  ArrowRight,
} from "lucide-react";
import { cn } from "@/lib/utils";

const features = [
  {
    icon: Globe,
    title: "Hybrid Consensus",
    description:
      "Validator selection blends internet performance, stake, identity, reputation, slashing, and attestation quorum.",
    gradient: "from-cyan-500 to-blue-500",
    delay: 0,
  },
  {
    icon: Shield,
    title: "Multi-Party Attestations",
    description:
      "Challenge-response attestations and peer-verified metric announcements make fabricated scores harder to sustain.",
    gradient: "from-emerald-500 to-teal-500",
    delay: 1,
  },
  {
    icon: Vote,
    title: "On-Chain Governance",
    description:
      "Stake-weighted proposal voting with real-time parameter changes. No node restarts required for governance actions.",
    gradient: "from-violet-500 to-purple-500",
    delay: 2,
  },
  {
    icon: Blocks,
    title: "Native Staking",
    description:
      "First-class staking and unstaking transaction types built directly into the protocol layer.",
    gradient: "from-orange-500 to-amber-500",
    delay: 3,
  },
  {
    icon: Wifi,
    title: "libp2p Networking",
    description:
      "Gossip-based peer discovery and block propagation through a robust libp2p networking stack with mDNS support.",
    gradient: "from-pink-500 to-rose-500",
    delay: 4,
  },
  {
    icon: Database,
    title: "Persistent Storage",
    description:
      "Embedded sled database for reliable local block and state persistence with no external dependencies.",
    gradient: "from-blue-500 to-indigo-500",
    delay: 5,
  },
  {
    icon: Activity,
    title: "Monitoring & Metrics",
    description:
      "Built-in health checks, Prometheus-style metrics, and WebSocket subscriptions for real-time event streaming.",
    gradient: "from-teal-500 to-cyan-500",
    delay: 6,
  },
  {
    icon: Lock,
    title: "Ed25519 Cryptography",
    description:
      "Secure wallet and transaction signing with Ed25519 elliptic curve cryptography and Argon2 key derivation.",
    gradient: "from-red-500 to-orange-500",
    delay: 7,
  },
];

function FeatureCard({
  feature,
  index,
}: {
  feature: (typeof features)[0];
  index: number;
}) {
  const cardRef = useRef<HTMLDivElement>(null);

  return (
    <div
      ref={cardRef}
      className={cn(
        "group relative overflow-hidden rounded-2xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm transition-all duration-500",
        "hover:border-border hover:bg-card/50 hover:shadow-xl hover:shadow-primary/5",
        "opacity-0 animate-fade-in-up"
      )}
      style={{ 
        animationDelay: `${index * 100}ms`,
        animationFillMode: "forwards"
      }}
    >
      {/* Gradient border on hover */}
      <div 
        className={cn(
          "absolute inset-0 -z-10 rounded-2xl opacity-0 transition-opacity duration-500 group-hover:opacity-100",
          "bg-gradient-to-br",
          feature.gradient
        )}
        style={{ padding: "1px" }}
        aria-hidden="true"
      >
        <div className="h-full w-full rounded-2xl bg-card" />
      </div>

      {/* Icon */}
      <div 
        className={cn(
          "mb-5 inline-flex size-12 items-center justify-center rounded-xl bg-gradient-to-br",
          feature.gradient,
          "text-white shadow-lg transition-transform duration-300 group-hover:scale-110"
        )}
      >
        <feature.icon className="size-6" aria-hidden="true" />
      </div>

      {/* Content */}
      <h3 className="mb-3 text-lg font-semibold text-foreground">
        {feature.title}
      </h3>
      <p className="text-sm leading-relaxed text-muted-foreground">
        {feature.description}
      </p>

      {/* Learn more link */}
      <div className="mt-4 flex items-center gap-1 text-sm font-medium text-primary opacity-0 transition-all duration-300 group-hover:opacity-100">
        <span>Learn more</span>
        <ArrowRight className="size-3 transition-transform duration-300 group-hover:translate-x-1" aria-hidden="true" />
      </div>

      {/* Subtle glow effect */}
      <div 
        className={cn(
          "pointer-events-none absolute -bottom-20 -right-20 size-40 rounded-full blur-3xl transition-opacity duration-500",
          "bg-gradient-to-br opacity-0 group-hover:opacity-20",
          feature.gradient
        )}
        aria-hidden="true"
      />
    </div>
  );
}

export function Features() {
  return (
    <section id="features" className="relative py-24 sm:py-32">
      {/* Background decorations */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        <div className="absolute left-0 top-1/4 h-[400px] w-[400px] rounded-full bg-primary/5 blur-[100px]" />
        <div className="absolute bottom-1/4 right-0 h-[300px] w-[300px] rounded-full bg-accent/5 blur-[100px]" />
      </div>

      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-3xl text-center">
          <span className="mb-4 inline-block text-sm font-semibold uppercase tracking-wider text-primary">
            Features
          </span>
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl lg:text-5xl" style={{ textWrap: "balance" }}>
            Everything You Need for a{" "}
            <span className="text-gradient">Modern Blockchain</span>
          </h2>
          <p className="mt-6 text-lg leading-relaxed text-muted-foreground">
            Built from scratch in Rust with performance, security, and
            decentralization as first-class priorities.
          </p>
        </div>

        {/* Feature grid - Bento style */}
        <div className="mx-auto mt-16 grid max-w-6xl grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {features.map((feature, index) => (
            <FeatureCard key={feature.title} feature={feature} index={index} />
          ))}
        </div>

        {/* Bottom CTA */}
        <div className="mt-16 text-center">
          <p className="mb-6 text-muted-foreground">
            Ready to build on NetChain?
          </p>
          <a
            href="/docs"
            className="group inline-flex items-center gap-2 text-primary transition-colors hover:text-primary/80"
          >
            <span className="font-medium">Explore the documentation</span>
            <ArrowRight className="size-4 transition-transform duration-300 group-hover:translate-x-1" aria-hidden="true" />
          </a>
        </div>
      </div>
    </section>
  );
}
