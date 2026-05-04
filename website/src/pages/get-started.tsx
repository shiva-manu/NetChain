import { Link } from "react-router-dom";
import {
  ArrowRight,
  Check,
  CheckCircle2,
  Copy,
  Rocket,
  Terminal,
  Sparkles,
  Zap,
  BookOpen,
} from "lucide-react";
import { useState } from "react";

import { SEO } from "@/components/seo";
import { FadeIn } from "@/components/ui/fade-in";
import { Button } from "@/components/ui/button";
import { SectionHeader } from "@/components/sections/section-header";
import { SectionBackground } from "@/components/sections/section-background";

const steps = [
  { number: 1, title: "Clone the repository", description: "Get the source code from GitHub", command: "git clone https://github.com/netchain-project/netchain.git && cd netchain" },
  { number: 2, title: "Build the binaries", description: "Compile the node and wallet using Cargo", command: "cargo build --release" },
  { number: 3, title: "Start the node", description: "Launch the blockchain node with default configuration", command: "cargo run --bin netchain --release" },
  { number: 4, title: "Open the explorer", description: "View chain state in your browser", command: "open http://localhost:3000/dashboard", note: "Or navigate to /dashboard in this website while the node is running" },
];

const verificationChecks = [
  { endpoint: "http://127.0.0.1:8545", name: "JSON-RPC", description: "Application interface for queries and transactions" },
  { endpoint: "http://127.0.0.1:8546", name: "WebSocket", description: "Real-time event subscriptions" },
  { endpoint: "http://127.0.0.1:9090/health", name: "Health Check", description: "Node status and readiness" },
  { endpoint: "http://127.0.0.1:9090/metrics", name: "Metrics", description: "Prometheus-compatible metrics" },
];

const nextSteps = [
  { title: "Explore the Dashboard", description: "View blocks, transactions, and network state in real-time", href: "/dashboard", icon: Sparkles },
  { title: "Read the Documentation", description: "Learn about commands, interfaces, and configuration options", href: "/docs", icon: BookOpen },
  { title: "Understand the Technology", description: "Deep dive into Proof of Internet consensus and architecture", href: "/technology", icon: Zap },
];

function StepCard({ step, isCompleted, onToggle, index }: { step: (typeof steps)[0]; isCompleted: boolean; onToggle: () => void; index: number }) {
  const [copied, setCopied] = useState(false);
  const copyCommand = () => { navigator.clipboard.writeText(step.command); setCopied(true); setTimeout(() => setCopied(false), 2000); };

  return (
    <FadeIn delay={index * 100} direction="up">
      <div className={`group relative overflow-hidden rounded-xl border transition-all duration-500 ${isCompleted ? "border-primary/40 bg-gradient-to-br from-primary/10 via-primary/5 to-transparent shadow-lg shadow-primary/10" : "border-border bg-card hover:border-primary/30 hover:bg-surface-hover"}`}>
        <div className="relative p-6">
          <div className="mb-5 flex items-start justify-between">
            <div className="flex items-center gap-4">
              <button onClick={onToggle} className={`flex h-11 w-11 items-center justify-center rounded-xl border-2 transition-all duration-300 ${isCompleted ? "border-primary bg-gradient-to-br from-primary to-accent text-white shadow-lg shadow-primary/30" : "border-border text-muted-foreground hover:border-primary/60 hover:text-primary"}`}>
                {isCompleted ? <Check className="h-5 w-5" /> : <span className="font-mono text-sm font-bold">{step.number}</span>}
              </button>
              <div>
                <h3 className={`text-lg font-semibold transition-colors ${isCompleted ? "text-primary" : "text-foreground"}`}>{step.title}</h3>
                <p className="text-sm text-muted-foreground">{step.description}</p>
              </div>
            </div>
          </div>

          <div className="overflow-hidden rounded-xl border border-border bg-code-bg">
            <div className="flex items-center justify-between border-b border-border bg-surface-elevated px-4 py-2.5">
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-red-500/70" />
                <div className="h-3 w-3 rounded-full bg-yellow-500/70" />
                <div className="h-3 w-3 rounded-full bg-green-500/70" />
              </div>
              <span className="flex items-center gap-1.5 text-xs text-muted-foreground font-mono"><Terminal className="w-3 h-3" /> terminal</span>
            </div>
            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3 overflow-hidden">
                <span className="text-primary font-mono">$</span>
                <code className="truncate font-mono text-sm text-foreground/90">{step.command}</code>
              </div>
              <button onClick={copyCommand} className="ml-4 flex-shrink-0 rounded-lg p-2 text-muted-foreground transition-all hover:bg-foreground/10 hover:text-primary" aria-label="Copy command">
                {copied ? <CheckCircle2 className="h-4 w-4 text-tertiary" /> : <Copy className="h-4 w-4" />}
              </button>
            </div>
          </div>

          {step.note && <p className="mt-4 text-xs text-muted-foreground italic">{step.note}</p>}
        </div>
      </div>
    </FadeIn>
  );
}

export function GetStartedPage() {
  const [completedSteps, setCompletedSteps] = useState<number[]>([]);
  const toggleStep = (stepNumber: number) => setCompletedSteps((prev) => prev.includes(stepNumber) ? prev.filter((n) => n !== stepNumber) : [...prev, stepNumber]);
  const progress = (completedSteps.length / steps.length) * 100;

  return (
    <div className="relative min-h-screen">
      <SEO title="Get Started | NetChain" description="Set up and run a NetChain node in minutes. Follow our step-by-step guide to build, launch, and verify your local blockchain node." keywords="NetChain setup, run blockchain node, cargo build, get started guide" />

      {/* Hero Section */}
      <section className="relative overflow-hidden pt-32 pb-20">
        <SectionBackground variant="gradient" />
        <div className="absolute inset-0 bg-grid-fine opacity-30" />

        <div className="container-wide relative z-10">
          <FadeIn direction="up" className="mx-auto max-w-3xl text-center">
            <SectionHeader
              badge={{ label: "Quick Start Guide", icon: Rocket }}
              title="Up and running in minutes"
              highlight="in minutes"
              description="Four steps from clone to running node. No complex setup, no external dependencies beyond Rust. Build, run, verify."
            />

            <FadeIn delay={200} direction="up" className="mx-auto mt-12 max-w-md">
              <div className="rounded-xl border border-border bg-surface-elevated p-6">
                <div className="flex items-center justify-between text-sm mb-3">
                  <span className="text-muted-foreground">Progress</span>
                  <span className="font-semibold text-foreground">{completedSteps.length} of {steps.length} completed</span>
                </div>
                <div className="relative h-3 overflow-hidden rounded-full bg-muted">
                  <div className="relative h-full rounded-full bg-gradient-to-r from-primary via-accent to-primary transition-all duration-700 ease-out" style={{ width: `${progress}%` }}>
                    <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent animate-shimmer bg-[length:200%_100%]" />
                  </div>
                </div>
                {progress === 100 && (
                  <div className="mt-3 flex items-center justify-center gap-2 text-tertiary">
                    <CheckCircle2 className="h-4 w-4" />
                    <span className="text-sm font-medium">All steps completed!</span>
                  </div>
                )}
              </div>
            </FadeIn>
          </FadeIn>
        </div>
      </section>

      {/* Steps Section */}
      <section className="relative py-20">
        <div className="container-wide">
          <div className="mx-auto max-w-2xl">
            <FadeIn direction="up" className="mb-10">
              <h2 className="text-2xl font-bold text-foreground">Setup Steps</h2>
              <p className="mt-2 text-muted-foreground">Click each step to mark it complete as you go</p>
            </FadeIn>
            <div className="space-y-5">
              {steps.map((step, index) => (
                <StepCard key={step.number} step={step} isCompleted={completedSteps.includes(step.number)} onToggle={() => toggleStep(step.number)} index={index} />
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Verification Section */}
      <section className="relative py-20">
        <SectionBackground variant="subtle" />
        <div className="container-wide relative z-10">
          <div className="mx-auto max-w-2xl">
            <FadeIn direction="up" className="mb-10">
              <SectionHeader badge={{ label: "Verification", icon: Terminal }} title="Confirm the node is running" description="Once started, verify these endpoints are accessible" align="left" className="mb-0" />
            </FadeIn>
            <div className="grid gap-4 sm:grid-cols-2">
              {verificationChecks.map((check, index) => (
                <FadeIn key={check.name} delay={index * 100} direction="up">
                  <div className="group rounded-xl border border-border bg-card p-5 transition-all duration-300 hover:border-primary/30 hover:bg-surface-hover">
                    <div className="mb-3 flex items-center gap-2">
                      <div className="h-2 w-2 rounded-full bg-tertiary animate-pulse" />
                      <span className="font-semibold text-foreground">{check.name}</span>
                    </div>
                    <p className="mb-3 text-sm text-muted-foreground">{check.description}</p>
                    <code className="inline-block rounded-lg bg-primary/10 border border-primary/20 px-3 py-1.5 font-mono text-xs text-primary">{check.endpoint}</code>
                  </div>
                </FadeIn>
              ))}
            </div>
            <FadeIn delay={400} direction="up" className="mt-8">
              <div className="rounded-xl border border-tertiary/20 bg-tertiary/5 p-5">
                <p className="text-sm text-foreground/80">
                  <strong className="text-tertiary">Tip:</strong> Use <code className="rounded bg-foreground/10 px-2 py-0.5 font-mono text-primary text-xs">curl http://127.0.0.1:9090/health</code> to quickly verify the node is responding.
                </p>
              </div>
            </FadeIn>
          </div>
        </div>
      </section>

      {/* Next Steps Section */}
      <section className="relative py-20">
        <div className="container-wide">
          <div className="mx-auto max-w-2xl">
            <FadeIn direction="up" className="mb-10 text-center">
              <h2 className="text-2xl font-bold text-foreground">What's next?</h2>
              <p className="mt-2 text-muted-foreground">Your node is running. Here's where to go from here.</p>
            </FadeIn>
            <div className="space-y-4">
              {nextSteps.map((item, index) => (
                <FadeIn key={item.title} delay={index * 100} direction="up">
                  <Link to={item.href} className="group flex items-center justify-between rounded-xl border border-border bg-card p-6 transition-all duration-300 hover:border-primary/40 hover:bg-surface-hover hover:shadow-lg hover:shadow-primary/5">
                    <div className="flex items-center gap-4">
                      <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 border border-primary/20 text-primary transition-all duration-300 group-hover:scale-110 group-hover:bg-primary/20">
                        <item.icon className="h-6 w-6" />
                      </div>
                      <div>
                        <h3 className="font-semibold text-foreground group-hover:text-primary transition-colors">{item.title}</h3>
                        <p className="mt-1 text-sm text-muted-foreground">{item.description}</p>
                      </div>
                    </div>
                    <ArrowRight className="h-5 w-5 text-muted-foreground transition-all group-hover:translate-x-2 group-hover:text-primary" />
                  </Link>
                </FadeIn>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative py-20">
        <div className="container-wide text-center">
          <FadeIn direction="up">
            <h2 className="text-2xl font-bold text-foreground sm:text-3xl">Need more details?</h2>
            <p className="mx-auto mt-4 max-w-xl text-muted-foreground">The documentation covers all commands, configuration options, and interface specifications in detail.</p>
            <div className="mt-8 flex flex-col items-center justify-center gap-4 sm:flex-row">
              <Button size="lg" className="px-8" href="/docs">Read the Docs <ArrowRight className="ml-2 h-4 w-4" /></Button>
              <Button variant="outline" size="lg" className="px-8" href="/dashboard">Open Explorer</Button>
            </div>
          </FadeIn>
        </div>
      </section>
    </div>
  );
}
