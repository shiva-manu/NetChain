import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { ArrowRight, ArrowUpRight } from "lucide-react";
import { Link } from "react-router-dom";

import { cn } from "@/lib/utils";

export type MarketingAction = {
  label: string;
  to?: string;
  href?: string;
};

export type HeroMetric = {
  label: string;
  value: string;
};

export type StatItem = {
  value: string;
  label: string;
  detail: string;
};

export type InsightItem = {
  icon: LucideIcon;
  eyebrow?: string;
  title: string;
  description: string;
  meta?: string;
};

export type ProcessItem = {
  step: string;
  title: string;
  description: string;
};

export type CommandItem = {
  label: string;
  command: string;
  description: string;
};

export type ChecklistItem = {
  title: string;
  description: string;
};

function ActionLink({
  action,
  variant = "primary",
  className,
}: {
  action: MarketingAction;
  variant?: "primary" | "secondary" | "tertiary";
  className?: string;
}) {
  const sharedClassName = cn(
    "inline-flex items-center justify-center gap-2 rounded-full px-5 py-3 text-sm font-semibold transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring",
    variant === "primary" &&
      "bg-primary text-primary-foreground shadow-[0_20px_45px_-24px_color-mix(in_oklab,var(--primary)_55%,transparent)] hover:bg-primary/90",
    variant === "secondary" &&
      "border border-border bg-card/88 text-foreground hover:bg-card",
    variant === "tertiary" &&
      "text-foreground/80 hover:text-foreground",
    className,
  );

  const icon = variant === "tertiary" ? (
    <ArrowUpRight className="size-4" aria-hidden="true" />
  ) : (
    <ArrowRight className="size-4" aria-hidden="true" />
  );

  if (action.href) {
    return (
      <a
        href={action.href}
        target="_blank"
        rel="noreferrer noopener"
        className={sharedClassName}
      >
        {action.label}
        {icon}
      </a>
    );
  }

  return (
    <Link to={action.to ?? "/"} className={sharedClassName}>
      {action.label}
      {icon}
    </Link>
  );
}

export function SectionHeading({
  eyebrow,
  title,
  description,
  align = "left",
}: {
  eyebrow: string;
  title: string;
  description: string;
  align?: "left" | "center";
}) {
  return (
    <div className={cn("max-w-3xl", align === "center" && "mx-auto text-center")}>
      <p className="eyebrow">{eyebrow}</p>
      <h2 className="mt-4 font-heading text-3xl leading-tight text-foreground text-balance sm:text-4xl">
        {title}
      </h2>
      <p className="mt-4 text-base leading-8 text-muted-foreground text-pretty sm:text-lg">
        {description}
      </p>
    </div>
  );
}

export function PageHero({
  eyebrow,
  title,
  description,
  primaryAction,
  secondaryAction,
  tertiaryAction,
  metrics,
  aside,
}: {
  eyebrow: string;
  title: string;
  description: string;
  primaryAction: MarketingAction;
  secondaryAction?: MarketingAction;
  tertiaryAction?: MarketingAction;
  metrics: HeroMetric[];
  aside: ReactNode;
}) {
  return (
    <section className="border-b border-border/60 pb-18 pt-12 sm:pb-24 sm:pt-18">
      <div className="site-grid grid gap-12 lg:grid-cols-[minmax(0,1.08fr)_minmax(320px,0.92fr)] lg:items-center">
        <div>
          <p className="eyebrow">{eyebrow}</p>
          <h1 className="mt-6 max-w-4xl font-heading text-5xl leading-[0.95] tracking-[-0.03em] text-foreground text-balance sm:text-6xl lg:text-7xl">
            {title}
          </h1>
          <p className="mt-6 max-w-2xl text-lg leading-8 text-muted-foreground text-pretty sm:text-xl">
            {description}
          </p>

          <div className="mt-9 flex flex-wrap items-center gap-3">
            <ActionLink action={primaryAction} />
            {secondaryAction ? (
              <ActionLink action={secondaryAction} variant="secondary" />
            ) : null}
            {tertiaryAction ? (
              <ActionLink action={tertiaryAction} variant="tertiary" />
            ) : null}
          </div>

          <dl className="mt-10 grid gap-3 sm:grid-cols-2 xl:max-w-2xl">
            {metrics.map((metric) => (
              <div key={metric.label} className="surface-card px-5 py-4">
                <dt className="text-xs font-semibold uppercase tracking-[0.26em] text-muted-foreground">
                  {metric.label}
                </dt>
                <dd className="mt-2 text-base font-semibold text-foreground">
                  {metric.value}
                </dd>
              </div>
            ))}
          </dl>
        </div>

        <div>{aside}</div>
      </div>
    </section>
  );
}

export function InsightGrid({
  items,
  columns = 3,
}: {
  items: InsightItem[];
  columns?: 2 | 3 | 4;
}) {
  return (
    <div
      className={cn(
        "grid gap-4 md:grid-cols-2",
        columns === 2 && "xl:grid-cols-2",
        columns === 3 && "xl:grid-cols-3",
        columns === 4 && "xl:grid-cols-4",
      )}
    >
      {items.map((item) => (
        <article key={item.title} className="surface-card h-full px-6 py-6">
          <div className="flex size-12 items-center justify-center rounded-2xl border border-border/70 bg-secondary/70 text-primary">
            <item.icon className="size-5" aria-hidden="true" />
          </div>
          {item.eyebrow ? (
            <p className="mt-5 text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground">
              {item.eyebrow}
            </p>
          ) : null}
          <h3 className="mt-3 font-heading text-2xl leading-tight text-foreground text-balance">
            {item.title}
          </h3>
          <p className="mt-3 text-sm leading-7 text-muted-foreground text-pretty sm:text-[0.98rem]">
            {item.description}
          </p>
          {item.meta ? (
            <p className="mt-5 text-sm font-semibold text-foreground/82">{item.meta}</p>
          ) : null}
        </article>
      ))}
    </div>
  );
}

export function StatGrid({ items }: { items: StatItem[] }) {
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      {items.map((item) => (
        <article key={item.label} className="surface-card px-6 py-5">
          <p className="text-4xl font-heading text-foreground sm:text-5xl">
            {item.value}
          </p>
          <h3 className="mt-3 text-sm font-semibold uppercase tracking-[0.22em] text-muted-foreground">
            {item.label}
          </h3>
          <p className="mt-3 text-sm leading-7 text-muted-foreground">{item.detail}</p>
        </article>
      ))}
    </div>
  );
}

export function ProcessList({ items }: { items: ProcessItem[] }) {
  return (
    <ol className="grid gap-4 lg:grid-cols-2">
      {items.map((item) => (
        <li key={item.step} className="surface-card px-6 py-6">
          <div className="flex items-start gap-4">
            <div className="flex size-12 shrink-0 items-center justify-center rounded-full border border-border/70 bg-secondary/72 font-heading text-lg text-primary">
              {item.step}
            </div>
            <div className="min-w-0">
              <p className="text-xs font-semibold uppercase tracking-[0.24em] text-muted-foreground">
                Protocol Step
              </p>
              <h3 className="mt-2 font-heading text-2xl leading-tight text-foreground text-balance">
                {item.title}
              </h3>
              <p className="mt-3 text-sm leading-7 text-muted-foreground text-pretty sm:text-[0.98rem]">
                {item.description}
              </p>
            </div>
          </div>
        </li>
      ))}
    </ol>
  );
}

export function CommandGrid({ items }: { items: CommandItem[] }) {
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      {items.map((item) => (
        <article key={item.label} className="surface-card overflow-hidden">
          <div className="border-b border-border/70 px-6 py-4">
            <p className="text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground">
              {item.label}
            </p>
          </div>
          <div className="px-6 py-6">
            <pre className="overflow-x-auto rounded-2xl border border-border/70 bg-foreground px-4 py-4 text-sm leading-7 text-background">
              <code>{item.command}</code>
            </pre>
            <p className="mt-4 text-sm leading-7 text-muted-foreground text-pretty">
              {item.description}
            </p>
          </div>
        </article>
      ))}
    </div>
  );
}

export function ChecklistGrid({ items }: { items: ChecklistItem[] }) {
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      {items.map((item) => (
        <article key={item.title} className="surface-card px-6 py-6">
          <div className="flex items-center gap-3">
            <div
              className="flex size-9 items-center justify-center rounded-full border border-border/70 bg-secondary/70 text-sm font-semibold text-primary"
              aria-hidden="true"
            >
              ✓
            </div>
            <h3 className="font-heading text-2xl leading-tight text-foreground text-balance">
              {item.title}
            </h3>
          </div>
          <p className="mt-4 text-sm leading-7 text-muted-foreground text-pretty sm:text-[0.98rem]">
            {item.description}
          </p>
        </article>
      ))}
    </div>
  );
}

export function CtaBanner({
  eyebrow,
  title,
  description,
  primaryAction,
  secondaryAction,
}: {
  eyebrow: string;
  title: string;
  description: string;
  primaryAction: MarketingAction;
  secondaryAction?: MarketingAction;
}) {
  return (
    <section className="pb-20 pt-6 sm:pb-24">
      <div className="site-grid">
        <div className="surface-card relative overflow-hidden px-7 py-8 sm:px-10 sm:py-10">
          <div
            className="pointer-events-none absolute inset-y-0 right-0 hidden w-1/3 bg-[radial-gradient(circle_at_center,_color-mix(in_oklab,var(--accent)_18%,transparent),_transparent_70%)] lg:block"
            aria-hidden="true"
          />
          <div className="max-w-3xl">
            <p className="eyebrow">{eyebrow}</p>
            <h2 className="mt-4 font-heading text-3xl leading-tight text-foreground text-balance sm:text-4xl">
              {title}
            </h2>
            <p className="mt-4 text-base leading-8 text-muted-foreground text-pretty sm:text-lg">
              {description}
            </p>
          </div>
          <div className="mt-8 flex flex-wrap gap-3">
            <ActionLink action={primaryAction} />
            {secondaryAction ? (
              <ActionLink action={secondaryAction} variant="secondary" />
            ) : null}
          </div>
        </div>
      </div>
    </section>
  );
}
