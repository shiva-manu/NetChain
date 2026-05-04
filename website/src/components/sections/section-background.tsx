import { cn } from "@/lib/utils";

interface SectionBackgroundProps {
  variant?: "gradient" | "grid" | "subtle" | "none";
  className?: string;
}

export function SectionBackground({ variant = "gradient", className }: SectionBackgroundProps) {
  if (variant === "none") return null;

  return (
    <div className={cn("absolute inset-0 overflow-hidden pointer-events-none", className)}>
      {variant === "gradient" && (
        <>
          <div className="absolute top-0 left-1/4 w-[500px] h-[500px] bg-primary/5 rounded-full blur-[120px]" />
          <div className="absolute bottom-0 right-1/4 w-[400px] h-[400px] bg-accent/5 rounded-full blur-[100px]" />
        </>
      )}
      {variant === "grid" && (
        <div className="absolute inset-0 bg-grid-pattern opacity-30" />
      )}
      {variant === "subtle" && (
        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-primary/[0.02] to-transparent" />
      )}
    </div>
  );
}
