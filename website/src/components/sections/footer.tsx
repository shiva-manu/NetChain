import { Link } from "react-router-dom";

import { Separator } from "@/components/ui/separator";

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
      { label: "Documentation", to: "/get-started" },
      { label: "Explorer", to: "/dashboard" },
      { label: "GitHub", href: "https://github.com/example/netchain" },
      { label: "RPC Reference", to: "/technology" },
      { label: "WebSocket API", to: "/technology" },
    ],
  },
  {
    title: "Network",
    links: [
      { label: "P2P: Port 30333", to: "/technology" },
      { label: "RPC: Port 8545", to: "/technology" },
      { label: "WS: Port 8546", to: "/technology" },
      { label: "Metrics: Port 9090", to: "/technology" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-border/40 bg-muted/20">
      <div className="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          <div className="col-span-2 md:col-span-1">
            <Link
              to="/"
              className="flex items-center gap-2.5 text-foreground transition-opacity hover:opacity-80"
            >
              <div className="flex size-8 items-center justify-center rounded-lg bg-primary">
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
              <span className="text-lg font-bold tracking-tight">NetChain</span>
            </Link>
            <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
              Layer-1 blockchain prototype powered by hybrid consensus and
              Proof of Internet telemetry.
            </p>
          </div>

          {footerLinks.map((group) => (
            <div key={group.title}>
              <h3 className="text-sm font-semibold text-foreground">
                {group.title}
              </h3>
              <ul className="mt-3 space-y-2">
                {group.links.map((link) => (
                  <li key={link.label}>
                    {"href" in link ? (
                      <a
                        href={link.href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sm text-muted-foreground transition-colors hover:text-foreground"
                      >
                        {link.label}
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

        <Separator className="my-8" />

        <div className="flex flex-col items-center justify-between gap-4 sm:flex-row">
          <p className="text-sm text-muted-foreground">
            NetChain is an experimental prototype. Use at your own risk.
          </p>
          <p className="text-sm text-muted-foreground">
            Built with Rust. Open source under the project license.
          </p>
        </div>
      </div>
    </footer>
  );
}
