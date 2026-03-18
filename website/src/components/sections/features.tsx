import { Card, CardContent } from "@/components/ui/card";
import {
  Globe,
  Shield,
  Vote,
  Blocks,
  Wifi,
  Database,
  Activity,
  Lock,
} from "lucide-react";

const features = [
  {
    icon: Globe,
    title: "Hybrid Consensus",
    description:
      "Validator selection blends internet performance, stake, identity, reputation, slashing, and attestation quorum.",
  },
  {
    icon: Shield,
    title: "Multi-Party Attestations",
    description:
      "Challenge-response attestations and peer-verified metric announcements make fabricated scores harder to sustain.",
  },
  {
    icon: Vote,
    title: "On-Chain Governance",
    description:
      "Stake-weighted proposal voting with real-time parameter changes. No node restarts required for governance actions.",
  },
  {
    icon: Blocks,
    title: "Native Staking",
    description:
      "First-class staking and unstaking transaction types built directly into the protocol layer.",
  },
  {
    icon: Wifi,
    title: "libp2p Networking",
    description:
      "Gossip-based peer discovery and block propagation through a robust libp2p networking stack with mDNS support.",
  },
  {
    icon: Database,
    title: "Persistent Storage",
    description:
      "Embedded sled database for reliable local block and state persistence with no external dependencies.",
  },
  {
    icon: Activity,
    title: "Monitoring & Metrics",
    description:
      "Built-in health checks, Prometheus-style metrics, and WebSocket subscriptions for real-time event streaming.",
  },
  {
    icon: Lock,
    title: "Ed25519 Cryptography",
    description:
      "Secure wallet and transaction signing with Ed25519 elliptic curve cryptography and Argon2 key derivation.",
  },
];

export function Features() {
  return (
    <section id="features" className="py-20 sm:py-28">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
            Everything You Need for a Modern Blockchain
          </h2>
          <p className="mt-4 text-lg text-muted-foreground">
            Built from scratch in Rust with performance, security, and
            decentralization as first-class priorities.
          </p>
        </div>

        {/* Feature cards */}
        <div className="mx-auto mt-16 grid max-w-6xl grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
          {features.map((feature) => (
            <Card
              key={feature.title}
              className="group border-border/50 bg-card/50 transition-all duration-200 hover:border-primary/30 hover:shadow-lg hover:shadow-primary/5"
            >
              <CardContent className="p-6">
                <div className="mb-4 flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary transition-colors group-hover:bg-primary/15">
                  <feature.icon className="size-5" aria-hidden="true" />
                </div>
                <h3 className="mb-2 font-semibold text-foreground">
                  {feature.title}
                </h3>
                <p className="text-sm leading-relaxed text-muted-foreground">
                  {feature.description}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
}
