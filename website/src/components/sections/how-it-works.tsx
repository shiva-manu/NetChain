import { Gauge, Users, BarChart3, CheckCircle2 } from "lucide-react";

const steps = [
  {
    icon: Gauge,
    step: "01",
    title: "Measure Network Performance",
    description:
      "Each node continuously measures its internet performance metrics: download speed, upload speed, latency, uptime, and connection stability.",
  },
  {
    icon: Users,
    step: "02",
    title: "Identity, Stake & Peer Attestation",
    description:
      "Nodes announce their metrics, stake, and identity signals. Peers verify those claims through challenge-response attestations and quorum thresholds.",
  },
  {
    icon: BarChart3,
    step: "03",
    title: "Aggregate Trust & Apply Slashing",
    description:
      "Verified metrics are aggregated into epoch-based trust scores. Reputation decay and slash history reduce the weight of misbehaving validators.",
  },
  {
    icon: CheckCircle2,
    step: "04",
    title: "Select & Produce Blocks",
    description:
      "Validators are selected from the hybrid trust score plus stake. Higher performance and better behavior both raise the chance of producing the next block.",
  },
];

export function HowItWorks() {
  return (
    <section
      id="how-it-works"
      className="border-y border-border/40 bg-muted/30 py-20 sm:py-28"
    >
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
            How Hybrid Consensus Works
          </h2>
          <p className="mt-4 text-lg text-muted-foreground">
            A consensus model that rewards real infrastructure while keeping
            stake, identity, reputation, and slashing in the selection loop.
          </p>
        </div>

        {/* Steps */}
        <div className="mx-auto mt-16 max-w-4xl">
          <div className="relative">
            {/* Vertical line */}
            <div
              className="absolute left-6 top-0 hidden h-full w-px bg-border md:block"
              aria-hidden="true"
            />

            <div className="space-y-12">
              {steps.map((step, index) => (
                <div key={step.step} className="relative flex gap-6 md:gap-8">
                  {/* Step number circle */}
                  <div className="relative z-10 flex size-12 shrink-0 items-center justify-center rounded-full border-2 border-primary/30 bg-background text-sm font-bold text-primary">
                    {step.step}
                  </div>

                  {/* Content */}
                  <div className="flex-1 pb-2">
                    <div className="flex items-center gap-3 mb-2">
                      <step.icon
                        className="size-5 text-accent"
                        aria-hidden="true"
                      />
                      <h3 className="text-lg font-semibold text-foreground">
                        {step.title}
                      </h3>
                    </div>
                    <p className="text-muted-foreground leading-relaxed">
                      {step.description}
                    </p>
                    {index < steps.length - 1 && (
                      <div className="mt-6 block h-px w-full bg-border/50 md:hidden" />
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
