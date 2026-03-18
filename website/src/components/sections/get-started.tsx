import { ArrowRight, Terminal } from "lucide-react";

export function GetStarted() {
  return (
    <section id="get-started" className="py-20 sm:py-28">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="mx-auto max-w-3xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
            Start Running a Node
          </h2>
          <p className="mt-4 text-lg text-muted-foreground">
            NetChain is open source. Clone the repository, build with Cargo, and
            start a hybrid consensus node in minutes.
          </p>

          {/* Code block */}
          <div className="mx-auto mt-10 max-w-xl overflow-hidden rounded-xl border border-border/50 bg-card text-left">
            <div className="flex items-center gap-2 border-b border-border/50 px-4 py-3">
              <Terminal
                className="size-4 text-muted-foreground"
                aria-hidden="true"
              />
              <span className="text-sm font-medium text-muted-foreground">
                Terminal
              </span>
            </div>
            <div className="p-4 font-mono text-sm">
              <div className="space-y-2">
                <p>
                  <span className="text-muted-foreground">$</span>{" "}
                  <span className="text-foreground">
                    git clone https://github.com/example/netchain.git
                  </span>
                </p>
                <p>
                  <span className="text-muted-foreground">$</span>{" "}
                  <span className="text-foreground">cd netchain</span>
                </p>
                <p>
                  <span className="text-muted-foreground">$</span>{" "}
                  <span className="text-foreground">cargo build</span>
                </p>
                <p>
                  <span className="text-muted-foreground">$</span>{" "}
                  <span className="text-foreground">
                    cargo run --bin netchain
                  </span>
                </p>
              </div>
            </div>
          </div>

          {/* Docker alternative */}
          <p className="mt-6 text-sm text-muted-foreground">
            Or use Docker:{" "}
            <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">
              docker compose up --build
            </code>
          </p>

          {/* CTA */}
          <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
            <a
              href="https://github.com/example/netchain"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex h-11 cursor-pointer items-center justify-center gap-2 rounded-lg bg-primary px-6 text-base font-semibold text-primary-foreground transition-colors hover:bg-primary/80 focus-visible:outline-2 focus-visible:outline-ring"
            >
              View Documentation
              <ArrowRight className="size-4" aria-hidden="true" />
            </a>
            <a
              href="https://github.com/example/netchain"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex h-11 cursor-pointer items-center justify-center gap-2 rounded-lg border border-border bg-background px-6 text-base font-semibold text-foreground transition-colors hover:bg-muted focus-visible:outline-2 focus-visible:outline-ring"
            >
              GitHub Repository
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
