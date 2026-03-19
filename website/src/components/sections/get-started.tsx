import { useState } from "react";
import { Link } from "react-router-dom";
import { ArrowRight, Terminal, Copy, Check, Github, Book, Zap } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

const codeSnippets = [
  { label: "Clone", code: "git clone https://github.com/shiva-manu/NetChain.git" },
  { label: "Navigate", code: "cd NetChain" },
  { label: "Build", code: "cargo build --release" },
  { label: "Run", code: "./target/release/netchain" },
];

type QuickLink = {
  icon: LucideIcon;
  title: string;
  description: string;
  color: string;
} & ({ to: string; href?: never } | { href: string; to?: never });

const quickLinks: QuickLink[] = [
  {
    icon: Book,
    title: "Documentation",
    description: "Learn how to build on NetChain",
    to: "/docs",
    color: "from-cyan-500 to-blue-500",
  },
  {
    icon: Github,
    title: "GitHub",
    description: "View source and contribute",
    href: "https://github.com/shiva-manu/NetChain",
    color: "from-violet-500 to-purple-500",
  },
  {
    icon: Zap,
    title: "Live Explorer",
    description: "Watch the network in real-time",
    to: "/dashboard",
    color: "from-orange-500 to-amber-500",
  },
];

function CodeBlock() {
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  const copyToClipboard = (code: string, index: number) => {
    navigator.clipboard.writeText(code);
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 2000);
  };

  const copyAll = () => {
    const allCode = codeSnippets.map(s => s.code).join("\n");
    navigator.clipboard.writeText(allCode);
    setCopiedIndex(-1);
    setTimeout(() => setCopiedIndex(null), 2000);
  };

  return (
    <div className="overflow-hidden rounded-2xl border border-border/50 bg-card/30 backdrop-blur-sm">
      {/* Terminal header */}
      <div className="flex items-center justify-between border-b border-border/50 bg-muted/30 px-4 py-3">
        <div className="flex items-center gap-3">
          <div className="flex gap-1.5" aria-hidden="true">
            <div className="size-3 rounded-full bg-red-500/80" />
            <div className="size-3 rounded-full bg-yellow-500/80" />
            <div className="size-3 rounded-full bg-green-500/80" />
          </div>
          <div className="flex items-center gap-2">
            <Terminal className="size-4 text-muted-foreground" aria-hidden="true" />
            <span className="font-mono text-sm text-muted-foreground">terminal</span>
          </div>
        </div>
        <button
          onClick={copyAll}
          className="flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          aria-label="Copy all commands"
        >
          {copiedIndex === -1 ? (
            <>
              <Check className="size-3" />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <Copy className="size-3" />
              <span>Copy all</span>
            </>
          )}
        </button>
      </div>

      {/* Code lines */}
      <div className="p-4 font-mono text-sm">
        <div className="space-y-2">
          {codeSnippets.map((snippet, index) => (
            <div
              key={index}
              className="group flex items-center gap-3"
            >
              <span className="select-none text-muted-foreground">$</span>
              <code className="flex-1 text-foreground">{snippet.code}</code>
              <button
                onClick={() => copyToClipboard(snippet.code, index)}
                className="opacity-0 transition-opacity group-hover:opacity-100"
                aria-label={`Copy "${snippet.code}"`}
              >
                {copiedIndex === index ? (
                  <Check className="size-4 text-accent" />
                ) : (
                  <Copy className="size-4 text-muted-foreground hover:text-foreground" />
                )}
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* Footer */}
      <div className="border-t border-border/50 bg-muted/20 px-4 py-3">
        <p className="text-center text-xs text-muted-foreground">
          Or use Docker:{" "}
          <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-foreground">
            docker compose up --build
          </code>
        </p>
      </div>
    </div>
  );
}

export function GetStarted() {
  return (
    <section id="get-started" className="relative py-24 sm:py-32">
      {/* Background */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        <div className="absolute inset-0 bg-gradient-to-t from-primary/5 via-transparent to-transparent" />
        <div className="absolute bottom-0 left-1/2 h-[500px] w-[800px] -translate-x-1/2 rounded-full bg-primary/5 blur-[120px]" />
      </div>

      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-3xl text-center">
          <span className="mb-4 inline-block text-sm font-semibold uppercase tracking-wider text-primary">
            Get Started
          </span>
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl lg:text-5xl" style={{ textWrap: "balance" }}>
            Start Running a{" "}
            <span className="text-gradient">Node in Minutes</span>
          </h2>
          <p className="mt-6 text-lg leading-relaxed text-muted-foreground">
            NetChain is open source. Clone the repository, build with Cargo, and
            start a hybrid consensus node in minutes.
          </p>
        </div>

        {/* Code block */}
        <div 
          className="mx-auto mt-12 max-w-2xl opacity-0 animate-fade-in-up"
          style={{ animationDelay: "100ms", animationFillMode: "forwards" }}
        >
          <CodeBlock />
        </div>

        {/* Quick links */}
        <div 
          className="mx-auto mt-16 grid max-w-4xl grid-cols-1 gap-5 sm:grid-cols-3 opacity-0 animate-fade-in-up"
          style={{ animationDelay: "200ms", animationFillMode: "forwards" }}
        >
          {quickLinks.map((link) => {
            const content = (
              <div
                className={cn(
                  "group relative flex flex-col items-center overflow-hidden rounded-2xl border border-border/50 bg-card/30 p-6 text-center backdrop-blur-sm",
                  "transition-all duration-300 hover:border-border hover:bg-card/50 hover:shadow-xl"
                )}
              >
                <div 
                  className={cn(
                    "mb-4 flex size-14 items-center justify-center rounded-xl",
                    "bg-gradient-to-br text-white shadow-lg",
                    link.color
                  )}
                >
                  <link.icon className="size-6" aria-hidden="true" />
                </div>
                <h3 className="mb-2 text-lg font-semibold text-foreground">
                  {link.title}
                </h3>
                <p className="text-sm text-muted-foreground">
                  {link.description}
                </p>
                <ArrowRight 
                  className="mt-4 size-5 text-primary opacity-0 transition-all duration-300 group-hover:translate-x-1 group-hover:opacity-100" 
                  aria-hidden="true"
                />
                {/* Glow */}
                <div 
                  className={cn(
                    "pointer-events-none absolute -bottom-20 h-40 w-40 rounded-full blur-3xl opacity-0 transition-opacity duration-300 group-hover:opacity-20",
                    "bg-gradient-to-br",
                    link.color
                  )}
                  aria-hidden="true"
                />
              </div>
            );

            if (link.to) {
              return (
                <Link key={link.title} to={link.to}>
                  {content}
                </Link>
              );
            }

            return (
              <a
                key={link.title}
                href={link.href}
                target="_blank"
                rel="noopener noreferrer"
              >
                {content}
              </a>
            );
          })}
        </div>

        {/* CTA */}
        <div 
          className="mt-16 text-center opacity-0 animate-fade-in-up"
          style={{ animationDelay: "300ms", animationFillMode: "forwards" }}
        >
          <p className="mb-6 text-muted-foreground">
            Ready to build the future of decentralized consensus?
          </p>
          <div className="flex flex-col items-center justify-center gap-4 sm:flex-row">
            <Link
              to="/docs"
              className="group inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-primary to-accent px-8 py-3 font-semibold text-primary-foreground shadow-lg shadow-primary/20 transition-all duration-300 hover:shadow-xl hover:shadow-primary/30"
            >
              <span>View Full Documentation</span>
              <ArrowRight className="size-4 transition-transform duration-300 group-hover:translate-x-0.5" aria-hidden="true" />
            </Link>
            <a
              href="https://github.com/shiva-manu/NetChain"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 rounded-xl border border-border px-8 py-3 font-semibold text-foreground transition-colors hover:bg-muted"
            >
              <Github className="size-4" aria-hidden="true" />
              <span>Star on GitHub</span>
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
