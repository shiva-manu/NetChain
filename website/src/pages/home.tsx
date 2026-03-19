import {
  type LucideIcon,
  ArrowRight,
  Blocks,
  Gauge,
  LayoutDashboard,
  Sparkles,
  Terminal,
  Vote,
} from "lucide-react";
import { Link } from "react-router-dom";

import { SEO } from "@/components/seo";
import { Hero } from "@/components/sections/hero";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";

type RouteCard = {
  title: string;
  description: string;
  to: string;
  icon: LucideIcon;
};

const routeCards: RouteCard[] = [
  {
    title: "Features",
    description:
      "See the protocol building blocks, security model, and runtime capabilities.",
    to: "/features",
    icon: Blocks,
  },
  {
    title: "How It Works",
    description:
      "Follow the hybrid trust flow from measurement to validator selection.",
    to: "/how-it-works",
    icon: Gauge,
  },
  {
    title: "Technology",
    description:
      "Review the Rust, libp2p, sled, RPC, and event-streaming stack.",
    to: "/technology",
    icon: Sparkles,
  },
  {
    title: "Governance",
    description:
      "Inspect staking, proposal voting, and the configurable protocol parameters.",
    to: "/governance",
    icon: Vote,
  },
  {
    title: "Get Started",
    description:
      "Run a node locally or with Docker and start exploring the protocol.",
    to: "/get-started",
    icon: Terminal,
  },
  {
    title: "Explorer",
    description:
      "Open the live dashboard for blocks, proposals, wallet data, and telemetry.",
    to: "/dashboard",
    icon: LayoutDashboard,
  },
];

function RouteCardTile({ card }: { card: RouteCard }) {
  return (
    <Link to={card.to} className="group block h-full">
      <Card className="h-full border-border/50 bg-card/50 transition-all duration-200 hover:-translate-y-0.5 hover:border-primary/30 hover:shadow-lg hover:shadow-primary/5">
        <CardContent className="flex h-full flex-col p-6">
          <div className="flex items-start justify-between gap-4">
            <div className="flex size-11 items-center justify-center rounded-2xl bg-primary/10 text-primary transition-colors group-hover:bg-primary/15">
              <card.icon className="size-5" aria-hidden="true" />
            </div>
            <ArrowRight className="size-4 text-muted-foreground transition-transform group-hover:translate-x-0.5 group-hover:text-foreground" />
          </div>
          <Badge variant="secondary" className="mt-5 w-fit">
            Route
          </Badge>
          <h3 className="mt-3 text-lg font-semibold text-foreground">
            {card.title}
          </h3>
          <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
            {card.description}
          </p>
        </CardContent>
      </Card>
    </Link>
  );
}

export function HomePage() {
  return (
    <div>
      <SEO
        title="NetChain - Proof of Internet Blockchain | Revolutionary Layer-1 Network"
        description="NetChain is the world's first Proof of Internet (PoI) blockchain. Validators are selected based on real network performance: download speed, upload speed, latency, and uptime. Join the decentralized network revolution."
        keywords="NetChain, Proof of Internet, PoI blockchain, Layer-1 blockchain, network performance blockchain, decentralized validator, cryptocurrency, Web3, DeFi, distributed ledger"
      />
      <Hero />

      <section className="border-y border-border/40 bg-muted/20 py-20 sm:py-28">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="mx-auto max-w-3xl text-center">
            <Badge variant="secondary" className="gap-1.5 px-3 py-1 text-sm">
              <Sparkles className="size-3.5 text-accent" aria-hidden="true" />
              Multi-page navigation
            </Badge>
            <h2 className="mt-6 text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
              Each major area now has its own route
            </h2>
            <p className="mt-4 text-lg leading-relaxed text-muted-foreground">
              The old section anchors are gone. Use the nav, footer, or cards
              below to jump directly into the page you need.
            </p>
          </div>

          <div className="mt-12 grid gap-6 md:grid-cols-2 xl:grid-cols-3">
            {routeCards.map((card) => (
              <RouteCardTile key={card.to} card={card} />
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
