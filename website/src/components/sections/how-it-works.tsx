import { Gauge, Users, BarChart3, CheckCircle2 } from "lucide-react";
import { cn } from "@/lib/utils";

const steps = [
  {
    icon: Gauge,
    step: "01",
    title: "Measure Network Performance",
    description:
      "Each node continuously measures its internet performance metrics: download speed, upload speed, latency, uptime, and connection stability.",
    color: "from-cyan-500 to-blue-500",
  },
  {
    icon: Users,
    step: "02",
    title: "Identity, Stake & Peer Attestation",
    description:
      "Nodes announce their metrics, stake, and identity signals. Peers verify those claims through challenge-response attestations and quorum thresholds.",
    color: "from-emerald-500 to-teal-500",
  },
  {
    icon: BarChart3,
    step: "03",
    title: "Aggregate Trust & Apply Slashing",
    description:
      "Verified metrics are aggregated into epoch-based trust scores. Reputation decay and slash history reduce the weight of misbehaving validators.",
    color: "from-violet-500 to-purple-500",
  },
  {
    icon: CheckCircle2,
    step: "04",
    title: "Select & Produce Blocks",
    description:
      "Validators are selected from the hybrid trust score plus stake. Higher performance and better behavior both raise the chance of producing the next block.",
    color: "from-orange-500 to-amber-500",
  },
];

export function HowItWorks() {
  return (
    <section
      id="how-it-works"
      className="relative overflow-hidden py-24 sm:py-32"
    >
      {/* Background */}
      <div className="pointer-events-none absolute inset-0 -z-10" aria-hidden="true">
        {/* Gradient mesh */}
        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-muted/30 to-transparent" />
        {/* Accent glow */}
        <div className="absolute left-1/2 top-0 h-[600px] w-[800px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/5 blur-[120px]" />
      </div>

      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-3xl text-center">
          <span className="mb-4 inline-block text-sm font-semibold uppercase tracking-wider text-primary">
            How It Works
          </span>
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl lg:text-5xl" style={{ textWrap: "balance" }}>
            Hybrid Consensus{" "}
            <span className="text-gradient">Explained</span>
          </h2>
          <p className="mt-6 text-lg leading-relaxed text-muted-foreground">
            A consensus model that rewards real infrastructure while keeping
            stake, identity, reputation, and slashing in the selection loop.
          </p>
        </div>

        {/* Steps - Timeline layout */}
        <div className="mx-auto mt-20 max-w-5xl">
          <div className="relative">
            {/* Vertical timeline line - desktop */}
            <div
              className="absolute left-8 top-0 hidden h-full w-px md:block lg:left-1/2 lg:-translate-x-1/2"
              aria-hidden="true"
            >
              <div className="h-full w-full bg-gradient-to-b from-primary/50 via-accent/50 to-primary/50" />
            </div>

            <div className="space-y-12 lg:space-y-0">
              {steps.map((step, index) => (
                <div
                  key={step.step}
                  className={cn(
                    "relative opacity-0 animate-fade-in-up",
                    "lg:flex lg:items-center lg:gap-12",
                    index % 2 === 0 ? "lg:flex-row" : "lg:flex-row-reverse"
                  )}
                  style={{ 
                    animationDelay: `${index * 150}ms`,
                    animationFillMode: "forwards"
                  }}
                >
                  {/* Step number - center on desktop */}
                  <div 
                    className={cn(
                      "absolute left-0 top-0 md:left-4 lg:left-1/2 lg:-translate-x-1/2",
                      "flex size-16 items-center justify-center"
                    )}
                  >
                    <div 
                      className={cn(
                        "relative flex size-16 items-center justify-center rounded-2xl",
                        "bg-gradient-to-br shadow-lg",
                        step.color
                      )}
                    >
                      <span className="text-xl font-bold text-white">
                        {step.step}
                      </span>
                      {/* Glow effect */}
                      <div 
                        className={cn(
                          "absolute inset-0 -z-10 rounded-2xl blur-xl opacity-50",
                          "bg-gradient-to-br",
                          step.color
                        )}
                        aria-hidden="true"
                      />
                    </div>
                  </div>

                  {/* Content card */}
                  <div 
                    className={cn(
                      "ml-24 md:ml-28 lg:ml-0 lg:w-[calc(50%-4rem)]",
                      index % 2 === 0 ? "lg:pr-8 lg:text-right" : "lg:pl-8 lg:text-left"
                    )}
                  >
                    <div 
                      className={cn(
                        "group rounded-2xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm",
                        "transition-all duration-300 hover:border-border hover:bg-card/50 hover:shadow-xl"
                      )}
                    >
                      <div 
                        className={cn(
                          "mb-4 flex items-center gap-3",
                          index % 2 === 0 ? "lg:flex-row-reverse" : ""
                        )}
                      >
                        <div 
                          className={cn(
                            "flex size-10 items-center justify-center rounded-lg",
                            "bg-gradient-to-br text-white",
                            step.color
                          )}
                        >
                          <step.icon className="size-5" aria-hidden="true" />
                        </div>
                        <h3 className="text-lg font-semibold text-foreground">
                          {step.title}
                        </h3>
                      </div>
                      <p className="text-muted-foreground leading-relaxed">
                        {step.description}
                      </p>
                    </div>
                  </div>

                  {/* Spacer for alternating layout */}
                  <div className="hidden lg:block lg:w-[calc(50%-4rem)]" aria-hidden="true" />
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Visual diagram placeholder */}
        <div className="mx-auto mt-20 max-w-4xl">
          <div className="relative overflow-hidden rounded-3xl border border-border/50 bg-card/30 p-8 backdrop-blur-sm">
            {/* Header */}
            <div className="mb-8 text-center">
              <h3 className="text-xl font-semibold text-foreground">
                Trust Score Calculation
              </h3>
              <p className="mt-2 text-sm text-muted-foreground">
                How validator selection weight is computed
              </p>
            </div>

            {/* Formula visualization */}
            <div className="flex flex-wrap items-center justify-center gap-4 font-mono text-sm">
              <div className="flex flex-col items-center rounded-xl border border-primary/30 bg-primary/5 px-4 py-3">
                <span className="text-xs text-muted-foreground">Internet Score</span>
                <span className="mt-1 text-lg font-bold text-primary">PoI</span>
              </div>
              <span className="text-2xl text-muted-foreground">×</span>
              <div className="flex flex-col items-center rounded-xl border border-accent/30 bg-accent/5 px-4 py-3">
                <span className="text-xs text-muted-foreground">Stake Weight</span>
                <span className="mt-1 text-lg font-bold text-accent">Stake</span>
              </div>
              <span className="text-2xl text-muted-foreground">×</span>
              <div className="flex flex-col items-center rounded-xl border border-border/50 bg-muted/30 px-4 py-3">
                <span className="text-xs text-muted-foreground">Reputation</span>
                <span className="mt-1 text-lg font-bold text-foreground">Rep</span>
              </div>
              <span className="text-2xl text-muted-foreground">×</span>
              <div className="flex flex-col items-center rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-3">
                <span className="text-xs text-muted-foreground">Slash Penalty</span>
                <span className="mt-1 text-lg font-bold text-destructive">1-S</span>
              </div>
              <span className="text-2xl text-muted-foreground">=</span>
              <div className="flex flex-col items-center rounded-xl border border-primary/50 bg-gradient-to-br from-primary/10 to-accent/10 px-6 py-3">
                <span className="text-xs text-muted-foreground">Selection Weight</span>
                <span className="mt-1 text-lg font-bold text-gradient">Score</span>
              </div>
            </div>

            {/* Decorative elements */}
            <div 
              className="pointer-events-none absolute -bottom-20 -right-20 h-40 w-40 rounded-full bg-primary/10 blur-3xl"
              aria-hidden="true"
            />
          </div>
        </div>
      </div>
    </section>
  );
}
