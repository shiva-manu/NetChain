import type { LucideIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { FadeIn } from "@/components/ui/fade-in";
import { cn } from "@/lib/utils";

interface CtaSectionProps {
  badge?: {
    label: string;
    icon?: LucideIcon;
  };
  title: string;
  description: string;
  primaryAction: { label: string; href: string };
  secondaryAction?: { label: string; href: string };
  className?: string;
}

export function CtaSection({
  badge,
  title,
  description,
  primaryAction,
  secondaryAction,
  className,
}: CtaSectionProps) {
  return (
    <section className={cn("section-padding", className)}>
      <FadeIn direction="up">
        <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-primary/10 via-primary/5 to-transparent border border-primary/20 p-12 md:p-16 text-center">
          <div className="absolute top-0 left-1/3 w-[300px] h-[300px] bg-primary/10 rounded-full blur-[80px] pointer-events-none" />
          <div className="absolute bottom-0 right-1/3 w-[200px] h-[200px] bg-accent/10 rounded-full blur-[60px] pointer-events-none" />

          <div className="relative z-10 space-y-6">
            {badge && (
              <Badge variant="signal">
                {badge.icon && <badge.icon className="w-3.5 h-3.5" />}
                {badge.label}
              </Badge>
            )}
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
              {title}
            </h2>
            <p className="text-lg text-muted-foreground max-w-xl mx-auto">
              {description}
            </p>
            <div className="flex flex-wrap justify-center gap-4 pt-2">
              <Button variant="default" size="lg" href={primaryAction.href}>
                {primaryAction.label}
              </Button>
              {secondaryAction && (
                <Button variant="outline" size="lg" href={secondaryAction.href}>
                  {secondaryAction.label}
                </Button>
              )}
            </div>
          </div>
        </div>
      </FadeIn>
    </section>
  );
}
