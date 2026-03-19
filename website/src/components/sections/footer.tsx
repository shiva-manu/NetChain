import { Link } from "react-router-dom";
import { Github, Twitter, MessageCircle, ArrowUpRight } from "lucide-react";
import { cn } from "@/lib/utils";

type InternalLink = {
  label: string;
  to: string;
};

type ExternalLink = {
  label: string;
  href: string;
};

type FooterLink = InternalLink | ExternalLink;

const footerLinks: Array<{
  title: string;
  links: FooterLink[];
}> = [
  {
    title: "Protocol",
    links: [
      { label: "Features", to: "/features" },
      { label: "How It Works", to: "/how-it-works" },
      { label: "Technology", to: "/technology" },
      { label: "Governance", to: "/governance" },
    ],
  },
  {
    title: "Developers",
    links: [
      { label: "Documentation", to: "/docs" },
      { label: "Get Started", to: "/get-started" },
      { label: "Explorer", to: "/dashboard" },
      { label: "GitHub", href: "https://github.com/shiva-manu/NetChain" },
    ],
  },
  {
    title: "Network",
    links: [
      { label: "RPC Endpoint", href: "https://api.netchain.me/rpc" },
      { label: "WebSocket", href: "wss://api.netchain.me/ws" },
      { label: "Metrics", href: "https://api.netchain.me/metrics" },
      { label: "Health", href: "https://api.netchain.me/health" },
    ],
  },
];

const socialLinks = [
  { 
    icon: Github, 
    label: "GitHub", 
    href: "https://github.com/shiva-manu/NetChain" 
  },
  { 
    icon: Twitter, 
    label: "Twitter", 
    href: "https://twitter.com/netchain" 
  },
  { 
    icon: MessageCircle, 
    label: "Discord", 
    href: "https://discord.gg/netchain" 
  },
];

export function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="relative border-t border-border/40">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        <div className="absolute inset-0 bg-gradient-to-t from-muted/30 to-transparent" />
      </div>

      <div className="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4 lg:grid-cols-5">
          {/* Brand column */}
          <div className="col-span-2 md:col-span-4 lg:col-span-2">
            <Link
              to="/"
              className="group inline-flex items-center gap-3 transition-opacity hover:opacity-90"
            >
              <div className="relative flex size-10 items-center justify-center">
                <div 
                  className="absolute inset-0 rounded-xl bg-gradient-to-br from-primary to-accent opacity-20 blur-md transition-opacity group-hover:opacity-40" 
                  aria-hidden="true"
                />
                <div className="relative flex size-10 items-center justify-center rounded-xl bg-gradient-to-br from-primary to-accent">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    className="size-5 text-primary-foreground"
                    aria-hidden="true"
                  >
                    <path
                      d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"
                      fill="currentColor"
                    />
                  </svg>
                </div>
              </div>
              <span className="text-xl font-bold tracking-tight text-foreground">
                NetChain
              </span>
            </Link>
            
            <p className="mt-4 max-w-sm text-sm leading-relaxed text-muted-foreground">
              Layer-1 blockchain powered by Proof of Internet — a hybrid consensus 
              mechanism that rewards validators for real-world network performance.
            </p>

            {/* Social links */}
            <div className="mt-6 flex items-center gap-3">
              {socialLinks.map((social) => (
                <a
                  key={social.label}
                  href={social.href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={cn(
                    "flex size-10 items-center justify-center rounded-lg border border-border/50",
                    "text-muted-foreground transition-all duration-300",
                    "hover:border-primary/50 hover:bg-primary/5 hover:text-primary"
                  )}
                  aria-label={social.label}
                >
                  <social.icon className="size-4" aria-hidden="true" />
                </a>
              ))}
            </div>
          </div>

          {/* Link columns */}
          {footerLinks.map((group) => (
            <div key={group.title}>
              <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-foreground">
                {group.title}
              </h3>
              <ul className="space-y-3">
                {group.links.map((link) => (
                  <li key={link.label}>
                    {"href" in link ? (
                      <a
                        href={link.href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="group inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
                      >
                        {link.label}
                        <ArrowUpRight 
                          className="size-3 opacity-0 transition-all duration-200 group-hover:opacity-100" 
                          aria-hidden="true"
                        />
                      </a>
                    ) : (
                      <Link
                        to={link.to}
                        className="text-sm text-muted-foreground transition-colors hover:text-foreground"
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
        <div className="mt-12 flex flex-col items-center justify-between gap-4 border-t border-border/40 pt-8 sm:flex-row">
          <p className="text-sm text-muted-foreground">
            &copy; {currentYear} NetChain. Open source under the project license.
          </p>
          <p className="text-sm text-muted-foreground">
            <span className="inline-flex items-center gap-1.5">
              Built with
              <span className="text-foreground" aria-label="Rust">🦀</span>
              Rust
            </span>
            <span className="mx-2 text-border">|</span>
            <span className="text-amber-500">⚠️</span>
            {" "}Experimental prototype
          </p>
        </div>
      </div>
    </footer>
  );
}
