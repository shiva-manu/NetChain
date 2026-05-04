import { Link } from "react-router-dom";
import { Blocks, Github, ExternalLink } from "lucide-react";

const footerLinks = [
  {
    title: "Product",
    links: [
      { href: "/features", label: "Features" },
      { href: "/technology", label: "Technology" },
      { href: "/dashboard", label: "Explorer" },
      { href: "/faucet", label: "Faucet" },
    ],
  },
  {
    title: "Developers",
    links: [
      { href: "/docs", label: "Documentation" },
      { href: "/get-started", label: "Get Started" },
      { href: "https://github.com/anthropics/netchain", label: "GitHub", external: true },
    ],
  },
  {
    title: "Network",
    links: [
      { href: "/dashboard", label: "Live Explorer" },
      { href: "https://api.netchain.me/health", label: "Health Status", external: true },
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-border bg-background">
      <div className="container-wide py-16">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-12">
          {/* Brand column */}
          <div className="col-span-2 md:col-span-1 space-y-4">
            <Link to="/" className="flex items-center gap-2.5">
              <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-primary text-primary-foreground">
                <Blocks className="w-4 h-4" />
              </div>
              <span className="font-semibold text-lg tracking-tight">
                NetChain
              </span>
            </Link>
            <p className="text-sm text-muted-foreground leading-relaxed max-w-xs">
              A Layer-1 blockchain with Proof of Internet consensus,
              selecting validators based on real network performance.
            </p>
            <div className="flex items-center gap-3 pt-1">
              <a
                href="https://github.com/anthropics/netchain"
                target="_blank"
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-foreground transition-colors"
                aria-label="GitHub"
              >
                <Github className="w-5 h-5" />
              </a>
            </div>
          </div>

          {/* Link columns */}
          {footerLinks.map((group) => (
            <div key={group.title} className="space-y-4">
              <h4 className="text-sm font-semibold text-foreground">
                {group.title}
              </h4>
              <ul className="space-y-2.5">
                {group.links.map((link) => (
                  <li key={link.href + link.label}>
                    {link.external ? (
                      <a
                        href={link.href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sm text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1"
                      >
                        {link.label}
                        <ExternalLink className="w-3 h-3" />
                      </a>
                    ) : (
                      <Link
                        to={link.href}
                        className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                      >
                        {link.label}
                      </Link>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom bar */}
        <div className="mt-14 pt-6 border-t border-border flex flex-col sm:flex-row items-center justify-between gap-4">
          <p className="text-xs text-muted-foreground font-mono">
            &copy; {new Date().getFullYear()} NetChain. Open source.
          </p>
          <div className="flex items-center gap-1.5">
            <span className="inline-block w-2 h-2 rounded-full bg-success animate-pulse" />
            <span className="text-xs text-muted-foreground font-mono">
              Testnet v2.0 Live
            </span>
          </div>
        </div>
      </div>
    </footer>
  );
}
