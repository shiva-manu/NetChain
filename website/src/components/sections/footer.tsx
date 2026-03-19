import { ArrowUpRight, Github } from "lucide-react";
import { Link } from "react-router-dom";

import { footerGroups, REPOSITORY_URL } from "@/content/site";

function BrandMark() {
  return (
    <div className="relative flex size-11 items-center justify-center rounded-2xl border border-border/80 bg-card">
      <div
        className="absolute inset-1 rounded-xl border border-primary/18 bg-[radial-gradient(circle_at_top,_color-mix(in_oklab,var(--accent)_28%,transparent),_transparent_62%)]"
        aria-hidden="true"
      />
      <div className="relative grid grid-cols-2 gap-1">
        <span className="size-2.5 rounded-full bg-primary" />
        <span className="size-2.5 rounded-full bg-accent" />
        <span className="size-2.5 rounded-full bg-foreground/18" />
        <span className="size-2.5 rounded-full bg-primary/35" />
      </div>
    </div>
  );
}

const runtimeInterfaces = [
  { label: "P2P", value: "0.0.0.0:30333" },
  { label: "RPC", value: "127.0.0.1:8545" },
  { label: "Monitoring", value: "127.0.0.1:9090" },
  { label: "WebSocket", value: "127.0.0.1:8546" },
];

export function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="border-t border-border/70 pb-[max(1.5rem,env(safe-area-inset-bottom))] pt-14">
      <div className="site-grid">
        <div className="grid gap-10 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_minmax(0,1fr)]">
          <div className="max-w-md">
            <div className="flex items-center gap-4">
              <BrandMark />
              <div>
                <p className="font-heading text-2xl text-foreground">NetChain</p>
                <p className="mt-1 text-[0.72rem] font-semibold uppercase tracking-[0.24em] text-muted-foreground">
                  Infrastructure Research Project
                </p>
              </div>
            </div>
            <p className="mt-5 text-sm leading-7 text-muted-foreground text-pretty sm:text-[0.98rem]">
              NetChain is an experimental Rust Layer-1 that treats measured internet
              performance as a first-class input to validator selection while keeping
              governance, staking, telemetry, and operator workflows inside the same
              codebase.
            </p>
            <a
              href={REPOSITORY_URL}
              target="_blank"
              rel="noreferrer noopener"
              className="mt-6 inline-flex items-center gap-2 rounded-full border border-border bg-card/90 px-4 py-2 text-sm font-semibold text-foreground transition-colors hover:bg-card"
            >
              <Github className="size-4" aria-hidden="true" />
              View Source
              <ArrowUpRight className="size-4" aria-hidden="true" />
            </a>
          </div>

          <div className="grid gap-8 sm:grid-cols-2 lg:col-span-2">
            {footerGroups.map((group) => (
              <div key={group.title}>
                <h2 className="text-xs font-semibold uppercase tracking-[0.24em] text-muted-foreground">
                  {group.title}
                </h2>
                <ul className="mt-4 space-y-3">
                  {group.links.map((link) => (
                    <li key={link.label}>
                      {link.href ? (
                        <a
                          href={link.href}
                          target="_blank"
                          rel="noreferrer noopener"
                          className="inline-flex items-center gap-2 text-sm font-semibold text-foreground transition-colors hover:text-primary"
                        >
                          {link.label}
                          <ArrowUpRight className="size-4" aria-hidden="true" />
                        </a>
                      ) : (
                        <Link
                          to={link.to ?? "/"}
                          className="text-sm font-semibold text-foreground transition-colors hover:text-primary"
                        >
                          {link.label}
                        </Link>
                      )}
                    </li>
                  ))}
                </ul>
              </div>
            ))}

            <div>
              <h2 className="text-xs font-semibold uppercase tracking-[0.24em] text-muted-foreground">
                Default Local Interfaces
              </h2>
              <div className="mt-4 grid gap-3">
                {runtimeInterfaces.map((item) => (
                  <div key={item.label} className="surface-card px-4 py-3">
                    <p className="text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground">
                      {item.label}
                    </p>
                    <p className="mt-2 font-mono text-sm text-foreground">{item.value}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        <div className="story-rule my-8" aria-hidden="true" />

        <div className="flex flex-col gap-2 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
          <p>© {currentYear} NetChain. Experimental protocol codebase for research and development.</p>
          <p>Rust node, wallet CLI, explorer telemetry, and governance surfaces in one repository.</p>
        </div>
      </div>
    </footer>
  );
}
