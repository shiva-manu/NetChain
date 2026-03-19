import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { ArrowRight, Zap, Github, Play } from "lucide-react";

export function Hero() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Animated network visualization
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const resizeCanvas = () => {
      canvas.width = canvas.offsetWidth * window.devicePixelRatio;
      canvas.height = canvas.offsetHeight * window.devicePixelRatio;
      ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    };

    resizeCanvas();
    window.addEventListener("resize", resizeCanvas);

    // Nodes for the network visualization
    interface Node {
      x: number;
      y: number;
      vx: number;
      vy: number;
      radius: number;
      connections: number[];
    }

    const nodeCount = 25;
    const nodes: Node[] = [];
    const width = canvas.offsetWidth;
    const height = canvas.offsetHeight;

    // Initialize nodes
    for (let i = 0; i < nodeCount; i++) {
      nodes.push({
        x: Math.random() * width,
        y: Math.random() * height,
        vx: (Math.random() - 0.5) * 0.3,
        vy: (Math.random() - 0.5) * 0.3,
        radius: Math.random() * 2 + 1,
        connections: [],
      });
    }

    // Animation loop
    let animationId: number;
    const animate = () => {
      ctx.clearRect(0, 0, width, height);

      // Update and draw nodes
      nodes.forEach((node, i) => {
        // Update position
        node.x += node.vx;
        node.y += node.vy;

        // Bounce off edges
        if (node.x < 0 || node.x > width) node.vx *= -1;
        if (node.y < 0 || node.y > height) node.vy *= -1;

        // Keep in bounds
        node.x = Math.max(0, Math.min(width, node.x));
        node.y = Math.max(0, Math.min(height, node.y));

        // Draw connections to nearby nodes
        nodes.forEach((other, j) => {
          if (i >= j) return;
          const dx = other.x - node.x;
          const dy = other.y - node.y;
          const dist = Math.sqrt(dx * dx + dy * dy);

          if (dist < 150) {
            const opacity = (1 - dist / 150) * 0.15;
            ctx.beginPath();
            ctx.moveTo(node.x, node.y);
            ctx.lineTo(other.x, other.y);
            ctx.strokeStyle = `rgba(0, 210, 190, ${opacity})`;
            ctx.lineWidth = 0.5;
            ctx.stroke();
          }
        });

        // Draw node
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
        ctx.fillStyle = "rgba(0, 210, 190, 0.6)";
        ctx.fill();
      });

      animationId = requestAnimationFrame(animate);
    };

    // Check for reduced motion preference
    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (!prefersReducedMotion) {
      animate();
    } else {
      // Draw static version
      nodes.forEach((node, i) => {
        nodes.forEach((other, j) => {
          if (i >= j) return;
          const dx = other.x - node.x;
          const dy = other.y - node.y;
          const dist = Math.sqrt(dx * dx + dy * dy);
          if (dist < 150) {
            ctx.beginPath();
            ctx.moveTo(node.x, node.y);
            ctx.lineTo(other.x, other.y);
            ctx.strokeStyle = "rgba(0, 210, 190, 0.1)";
            ctx.lineWidth = 0.5;
            ctx.stroke();
          }
        });
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
        ctx.fillStyle = "rgba(0, 210, 190, 0.4)";
        ctx.fill();
      });
    }

    return () => {
      window.removeEventListener("resize", resizeCanvas);
      cancelAnimationFrame(animationId);
    };
  }, []);

  return (
    <section className="relative min-h-[90vh] overflow-hidden">
      {/* Network canvas background */}
      <canvas
        ref={canvasRef}
        className="pointer-events-none absolute inset-0 -z-10 h-full w-full opacity-60"
        aria-hidden="true"
      />
      
      {/* Gradient overlays */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        {/* Primary glow orb */}
        <div className="absolute left-1/4 top-1/4 h-[600px] w-[600px] -translate-x-1/2 -translate-y-1/2">
          <div className="h-full w-full rounded-full bg-primary/20 blur-[120px]" />
        </div>
        {/* Accent glow orb */}
        <div className="absolute right-1/4 top-1/2 h-[400px] w-[400px] translate-x-1/2">
          <div className="h-full w-full rounded-full bg-accent/15 blur-[100px]" />
        </div>
        {/* Bottom fade */}
        <div className="absolute inset-x-0 bottom-0 h-32 bg-gradient-to-t from-background to-transparent" />
      </div>

      <div className="mx-auto max-w-7xl px-4 pb-20 pt-24 sm:px-6 sm:pb-32 sm:pt-32 lg:px-8 lg:pb-40 lg:pt-40">
        <div className="mx-auto max-w-4xl text-center">
          {/* Announcement badge */}
          <div className="mb-8 inline-flex opacity-0 animate-fade-in-up" style={{ animationDelay: "0ms", animationFillMode: "forwards" }}>
            <Badge
              variant="outline"
              className="gap-2 border-primary/30 bg-primary/5 px-4 py-2 text-sm font-medium backdrop-blur-sm transition-colors hover:border-primary/50 hover:bg-primary/10"
            >
              <span className="relative flex size-2">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-accent opacity-75" />
                <span className="relative inline-flex size-2 rounded-full bg-accent" />
              </span>
              Testnet Live — Join the Network
              <ArrowRight className="size-3.5" aria-hidden="true" />
            </Badge>
          </div>

          {/* Headline */}
          <h1 
            className="text-4xl font-extrabold tracking-tight text-foreground opacity-0 animate-fade-in-up sm:text-5xl lg:text-7xl"
            style={{ animationDelay: "100ms", animationFillMode: "forwards" }}
          >
            <span className="block">Consensus Powered by</span>
            <span className="mt-2 block text-gradient-animated">
              Real Internet Performance
            </span>
          </h1>

          {/* Subheadline */}
          <p 
            className="mx-auto mt-8 max-w-2xl text-lg leading-relaxed text-muted-foreground opacity-0 animate-fade-in-up sm:text-xl"
            style={{ animationDelay: "200ms", animationFillMode: "forwards" }}
          >
            NetChain introduces{" "}
            <span className="font-semibold text-foreground">Proof of Internet</span>
            {" "}— a hybrid consensus mechanism that rewards validators for 
            real-world network performance, stake, and reputation.
          </p>

          {/* CTA buttons */}
          <div 
            className="mt-12 flex flex-col items-center justify-center gap-4 opacity-0 animate-fade-in-up sm:flex-row sm:gap-5"
            style={{ animationDelay: "300ms", animationFillMode: "forwards" }}
          >
            <Link
              to="/get-started"
              className="group relative flex h-12 w-full items-center justify-center gap-2 overflow-hidden rounded-xl bg-gradient-to-r from-primary to-accent px-8 text-base font-semibold text-primary-foreground shadow-lg shadow-primary/20 transition-all duration-300 hover:shadow-xl hover:shadow-primary/30 sm:w-auto"
            >
              <Zap className="size-4" aria-hidden="true" />
              <span>Start Building</span>
              <ArrowRight className="size-4 transition-transform duration-300 group-hover:translate-x-0.5" aria-hidden="true" />
              {/* Shine effect */}
              <div 
                className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/20 to-transparent transition-transform duration-700 group-hover:translate-x-full"
                aria-hidden="true"
              />
            </Link>
            
            <a
              href="https://github.com/shiva-manu/NetChain"
              target="_blank"
              rel="noopener noreferrer"
              className="flex h-12 w-full items-center justify-center gap-2 rounded-xl border border-border/50 bg-background/50 px-8 text-base font-semibold text-foreground backdrop-blur-sm transition-all duration-300 hover:border-border hover:bg-muted/50 sm:w-auto"
            >
              <Github className="size-4" aria-hidden="true" />
              View Source
            </a>
            
            <Link
              to="/dashboard"
              className="flex h-12 w-full items-center justify-center gap-2 rounded-xl border border-primary/30 bg-primary/5 px-8 text-base font-semibold text-primary backdrop-blur-sm transition-all duration-300 hover:border-primary/50 hover:bg-primary/10 sm:w-auto"
            >
              <Play className="size-4" aria-hidden="true" />
              Live Explorer
            </Link>
          </div>

          {/* Stats */}
          <div 
            className="mx-auto mt-20 grid max-w-3xl grid-cols-2 gap-px overflow-hidden rounded-2xl border border-border/50 bg-border/50 opacity-0 animate-fade-in-up sm:grid-cols-4"
            style={{ animationDelay: "400ms", animationFillMode: "forwards" }}
          >
            {[
              { value: "Rust", label: "Built With", icon: "🦀" },
              { value: "PoI", label: "Consensus", icon: "⚡" },
              { value: "P2P", label: "libp2p Stack", icon: "🌐" },
              { value: "6", label: "Trust Signals", icon: "🛡️" },
            ].map((stat) => (
              <div 
                key={stat.label} 
                className="group relative flex flex-col items-center justify-center bg-background/80 px-6 py-8 backdrop-blur-sm transition-colors hover:bg-muted/30"
              >
                <span className="mb-2 text-2xl" aria-hidden="true">{stat.icon}</span>
                <span className="text-2xl font-bold text-foreground sm:text-3xl">
                  {stat.value}
                </span>
                <span className="mt-1 text-sm text-muted-foreground">
                  {stat.label}
                </span>
                {/* Hover glow */}
                <div 
                  className="absolute inset-0 -z-10 opacity-0 transition-opacity group-hover:opacity-100"
                  style={{
                    background: `radial-gradient(circle at center, color-mix(in oklab, var(--primary) 10%, transparent), transparent 70%)`
                  }}
                  aria-hidden="true"
                />
              </div>
            ))}
          </div>

          {/* Trusted by / Built for */}
          <div 
            className="mt-20 opacity-0 animate-fade-in-up"
            style={{ animationDelay: "500ms", animationFillMode: "forwards" }}
          >
            <p className="mb-6 text-sm font-medium uppercase tracking-wider text-muted-foreground">
              Built for the Future of Web3
            </p>
            <div className="flex flex-wrap items-center justify-center gap-x-12 gap-y-6 opacity-50">
              {["DeFi Protocols", "NFT Platforms", "DAOs", "Enterprise", "Gaming"].map((item) => (
                <span 
                  key={item}
                  className="text-sm font-medium text-muted-foreground transition-colors hover:text-foreground"
                >
                  {item}
                </span>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Scroll indicator */}
      <div className="absolute bottom-8 left-1/2 -translate-x-1/2 opacity-0 animate-fade-in-up" style={{ animationDelay: "600ms", animationFillMode: "forwards" }}>
        <div className="flex flex-col items-center gap-2">
          <span className="text-xs text-muted-foreground">Scroll to explore</span>
          <div className="h-10 w-6 rounded-full border border-border/50 p-1">
            <div className="mx-auto h-2 w-1 animate-bounce rounded-full bg-primary" />
          </div>
        </div>
      </div>
    </section>
  );
}
