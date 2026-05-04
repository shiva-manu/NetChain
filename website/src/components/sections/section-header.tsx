import type { LucideIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { FadeIn } from "@/components/ui/fade-in";
import { cn } from "@/lib/utils";

interface SectionHeaderProps {
  badge?: {
    label: string;
    icon?: LucideIcon;
    variant?: "default" | "secondary" | "outline" | "signal" | "terminal" | "ghost" | "glass";
  };
  title: string;
  highlight?: string;
  description?: string;
  align?: "left" | "center";
  className?: string;
}

export function SectionHeader({
  badge,
  title,
  highlight,
  description,
  align = "center",
  className,
}: SectionHeaderProps) {
  const titleContent = highlight ? (
    <>
      {title.split(highlight).map((part, i, arr) => (
        <span key={i}>
          {part}
          {i < arr.length - 1 && (
            <span className="text-gradient">{highlight}</span>
          )}
        </span>
      ))}
    </>
  ) : (
    title
  );

  return (
    <FadeIn direction="up">
      <div className={cn("mb-16", align === "center" && "text-center", className)}>
        {badge && (
          <Badge variant={badge.variant ?? "outline"} className="mb-6">
            {badge.icon && <badge.icon className="w-3.5 h-3.5" />}
            {badge.label}
          </Badge>
        )}
        <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-6">
          {titleContent}
        </h2>
        {description && (
          <p
            className={cn(
              "text-xl text-muted-foreground leading-relaxed",
              align === "center" ? "max-w-2xl mx-auto" : "max-w-2xl"
            )}
          >
            {description}
          </p>
        )}
      </div>
    </FadeIn>
  );
}
