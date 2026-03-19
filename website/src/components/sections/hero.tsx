import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import { ArrowRight, Zap, Github, Play, Code2, Network, Shield, Activity } from "lucide-react";

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

    const nodeCount = 30;
    const nodes: Node[] = [];
    const width = canvas.offsetWidth;
    const height = canvas.offsetHeight;

    // Initialize nodes
    for (let i = 0; i < nodeCount; i++) {
      nodes.push({
        x: Math.random() * width,
        y: Math.random() * height,
        vx: (Math.random() - 0.5) * 0.4,
        vy: (Math.random() - 0.5) * 0.4,
        radius: Math.random() * 2.5 + 1.5,
        connections: [],
      });
    }

    // Get CSS variable color
    const primaryColor = getComputedStyle(document.documentElement)
      .getPropertyValue('--primary')
      .trim();
    const accentColor = getComputedStyle(document.documentElement)
      .getPropertyValue('--accent')
      .trim();

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

          if (dist < 160) {
            const opacity = (1 - dist / 160) * 0.2;
            ctx.beginPath();
            ctx.moveTo(node.x, node.y);
            ctx.lineTo(other.x, other.y);
            // Use primary color with oklch
            ctx.strokeStyle = `oklch(${primaryColor} / ${opacity})`;
            ctx.lineWidth = 1;
            ctx.stroke();
          }
        });

        // Draw node with gradient
        const gradient = ctx.createRadialGradient(node.x, node.y, 0, node.x, node.y, node.radius);
        gradient.addColorStop(0, `oklch(${primaryColor} / 0.8)`);
        gradient.addColorStop(1, `oklch(${accentColor} / 0.4)`);
        
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
        ctx.fillStyle = gradient;
        ctx.fill();
        
        // Add glow
        ctx.shadowBlur = 8;
        ctx.shadowColor = `oklch(${primaryColor} / 0.5)`;
        ctx.shadowBlur = 0;
      });

      animationId = requestAnimationFrame(animate);
    };

    // Check for reduced motion preference - CRITICAL accessibility requirement
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
          if (dist < 160) {
            ctx.beginPath();
            ctx.moveTo(node.x, node.y);
            ctx.lineTo(other.x, other.y);
            ctx.strokeStyle = `oklch(${primaryColor} / 0.15)`;
            ctx.lineWidth = 1;
            ctx.stroke();
          }
        });
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
        ctx.fillStyle = `oklch(${primaryColor} / 0.5)`;
        ctx.fill();
      });
    }

    return () => {
      window.removeEventListener("resize", resizeCanvas);
      if (animationId) cancelAnimationFrame(animationId);
    };
  }, []);

  const stats = [
    { 
      value: "Rust", 
      label: "Built With", 
      icon: Code2,
      description: "High-performance systems language"
    },
    { 
      value: "PoI", 
      label: "Consensus", 
      icon: Activity,
      description: "Proof of Internet mechanism"
    },
    { 
      value: "P2P", 
      label: "libp2p Stack", 
      icon: Network,
      description: "Decentralized networking"
    },
    { 
      value: "6", 
      label: "Trust Signals", 
      icon: Shield,
      description: "Multi-dimensional validation"
    },
  ];

  return (
    <section className="relative min-h-dvh overflow-hidden" aria-labelledby="hero-title">
      {/* Network canvas background */}
      <canvas
        ref={canvasRef}
        className="pointer-events-none absolute inset-0 -z-10 h-full w-full opacity-50"
        aria-hidden="true"
      />
      
      {/* Gradient overlays */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        {/* Primary glow orb */}
        <div className="absolute left-1/4 top-1/4 h-[600px] w-[600px] -translate-x-1/2 -translate-y-1/2">
          <div className="h-full w-full rounded-full bg-primary/15 blur-[120px]" />
        </div>
        {/* Accent glow orb */}
        <div className="absolute right-1/4 top-1/2 h-[400px] w-[400px] translate-x-1/2">
          <div className="h-full w-full rounded-full bg-accent/10 blur-[100px]" />
        </div>
        {/* Bottom fade */}
        <div className="absolute inset-x-0 bottom-0 h-32 bg-gradient-to-t from-background to-transparent" />
      </div>

      <div className="container-site section-spacing-lg">
        <div className="mx-auto max-w-4xl text-center">
          {/* Announcement badge */}
          <div className="mb-8 inline-flex animate-fade-in-up opacity-0" style={{ animationDelay: "0ms", animationFillMode: "forwards" }}>
            <Badge
              variant="outline"
              className="gap-2 border-primary/30 bg-primary/5 px-4 py-2 text-sm font-medium backdrop-blur-sm transition-all duration-200 hover:border-primary/50 hover:bg-primary/10"
            >
              <span className="relative flex size-2" aria-hidden="true">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-accent opacity-75" />
                <span className="relative inline-flex size-2 rounded-full bg-accent" />
              </span>
              <span>Testnet Live — Join the Network</span>
              <ArrowRight className="size-3.5" aria-hidden="true" />
            </Badge>
          </div>

          {/* Headline */}
          <h1 
            id="hero-title"
            className="animate-fade-in-up text-5xl font-bold tracking-tight text-foreground opacity-0 sm:text-6xl lg:text-7xl"
            style={{ animationDelay: "100ms", animationFillMode: "forwards" }}
          >
            <span className="block">Consensus Powered by</span>
            <span className="mt-3 block bg-gradient-to-r from-primary via-accent to-primary bg-clip-text text-transparent bg-[length:200%_auto] animate-[gradient_8s_ease-in-out_infinite]">
              Real Internet Performance
            </span>
          </h1>

          {/* Subheadline */}
          <p 
            className="animate-fade-in-up mx-auto mt-8 max-w-2xl text-lg leading-relaxed text-muted-foreground opacity-0 sm:text-xl"
            style={{ animationDelay: "200ms", animationFillMode: "forwards" }}
          >
            NetChain introduces{" "}
            <span className="font-semibold text-foreground">Proof of Internet</span>
            {" "}— a hybrid consensus mechanism that rewards validators for 
            real-world network performance, stake, and reputation.
          </p>

          {/* CTA buttons - Touch target minimum 44px height */}
          <div 
            className="animate-fade-in-up mt-12 flex flex-col items-center justify-center gap-4 opacity-0 sm:flex-row sm:gap-5"
            style={{ animationDelay: "300ms", animationFillMode: "forwards" }}
          >
            <Link
              to="/get-started"
              className="btn-primary group relative w-full sm:w-auto"
            >
              <Zap className="size-4" aria-hidden="true" />
              <span>Start Building</span>
              <ArrowRight className="size-4 transition-transform duration-200 group-hover:translate-x-0.5" aria-hidden="true" />
              {/* Shine effect - subtle */}
              <div 
                className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/15 to-transparent transition-transform duration-500 group-hover:translate-x-full"
                aria-hidden="true"
              />
            </Link>
            
            <a
              href="https://github.com/shiva-manu/NetChain"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-secondary w-full sm:w-auto"
            >
              <Github className="size-4" aria-hidden="true" />
              <span>View Source</span>
            </a>
            
            <Link
              to="/dashboard"
              className="btn-outline w-full sm:w-auto"
            >
              <Play className="size-4" aria-hidden="true" />
              <span>Live Explorer</span>
            </Link>
          </div>

          {/* Stats - No emojis (CRITICAL: use SVG icons instead) */}
          <div 
            className="animate-fade-in-up mx-auto mt-20 grid max-w-3xl grid-cols-2 gap-px overflow-hidden rounded-2xl border border-border/50 bg-border/50 opacity-0 sm:grid-cols-4"
            style={{ animationDelay: "400ms", animationFillMode: "forwards" }}
            role="list"
            aria-label="Platform statistics"
          >
            {stats.map((stat) => {
              const Icon = stat.icon;
              return (
                <div 
                  key={stat.label} 
                  className="group relative flex flex-col items-center justify-center bg-card/80 px-6 py-8 backdrop-blur-sm transition-all duration-200 hover:bg-muted/30"
                  role="listitem"
                >
                  <Icon className="mb-3 size-6 text-primary" aria-hidden="true" />
                  <span className="text-2xl font-bold text-foreground sm:text-3xl">
                    {stat.value}
                  </span>
                  <span className="mt-1 text-sm font-medium text-muted-foreground">
                    {stat.label}
                  </span>
                  {/* Hover glow - subtle */}
                  <div 
                    className="absolute inset-0 -z-10 opacity-0 transition-opacity duration-200 group-hover:opacity-100"
                    style={{
                      background: `radial-gradient(circle at center, color-mix(in oklab, var(--primary) 8%, transparent), transparent 70%)`
                    }}
                    aria-hidden="true"
                  />
                </div>
              );
            })}
          </div>

          {/* Built for */}
          <div 
            className="animate-fade-in-up mt-20 opacity-0"
            style={{ animationDelay: "500ms", animationFillMode: "forwards" }}
          >
            <p className="mb-6 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
              Built for the Future of Web3
            </p>
            <div className="flex flex-wrap items-center justify-center gap-x-12 gap-y-6 opacity-60">
              {["DeFi Protocols", "NFT Platforms", "DAOs", "Enterprise", "Gaming"].map((item) => (
                <span 
                  key={item}
                  className="text-sm font-medium text-muted-foreground transition-colors duration-200 hover:text-foreground"
                >
                  {item}
                </span>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Scroll indicator */}
      <div 
        className="animate-fade-in-up absolute bottom-8 left-1/2 -translate-x-1/2 opacity-0" 
        style={{ animationDelay: "600ms", animationFillMode: "forwards" }}
        aria-label="Scroll to explore more content"
      >
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
