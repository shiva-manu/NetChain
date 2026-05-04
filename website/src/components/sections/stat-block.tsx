import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface StatBlockProps {
  label: string;
  value: string | number;
  suffix?: string;
  icon?: LucideIcon;
  variant?: "default" | "primary" | "accent" | "tertiary";
  className?: string;
}

const variantStyles = {
  default: "text-foreground",
  primary: "text-primary",
  accent: "text-accent",
  tertiary: "text-tertiary",
};

export function StatBlock({
  label,
  value,
  suffix,
  icon: Icon,
  variant = "default",
  className,
}: StatBlockProps) {
  return (
    <div className={cn("space-y-1", className)}>
      <div className="flex items-center gap-2 text-muted-foreground text-sm">
        {Icon && <Icon className="w-4 h-4" />}
        <span>{label}</span>
      </div>
      <div className={cn("text-2xl font-bold tracking-tight tabular-nums", variantStyles[variant])}>
        {value}
        {suffix && <span className="text-base font-normal text-muted-foreground ml-1">{suffix}</span>}
      </div>
    </div>
  );
}
