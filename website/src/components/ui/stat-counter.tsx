import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import { AnimatedSection } from "./animated-section";

interface StatCounterProps {
  value: string | number;
  label: string;
  icon?: React.ReactNode;
  delay?: number;
  className?: string;
}

export function StatCounter({
  value,
  label,
  icon,
  delay = 0,
  className,
}: StatCounterProps) {
  const [displayValue, setDisplayValue] = useState(0);
  const [hasAnimated, setHasAnimated] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated) {
          setHasAnimated(true);
          
          // Parse numeric value from string like "10,000+"
          const numericValue = typeof value === "string" 
            ? parseFloat(value.replace(/[^0-9.]/g, ""))
            : value;
          
          const duration = 2000;
          const steps = 60;
          const increment = numericValue / steps;
          let current = 0;

          const timer = setInterval(() => {
            current += increment;
            if (current >= numericValue) {
              setDisplayValue(numericValue);
              clearInterval(timer);
            } else {
              setDisplayValue(Math.floor(current));
            }
          }, duration / steps);

          return () => clearInterval(timer);
        }
      },
      { threshold: 0.5 }
    );

    if (ref.current) {
      observer.observe(ref.current);
    }

    return () => {
      if (ref.current) {
        observer.unobserve(ref.current);
      }
    };
  }, [value, hasAnimated]);

  // Format the display value
  const formatValue = () => {
    if (typeof value === "string") {
      const suffix = value.replace(/[0-9.]/g, "");
      return `${displayValue.toLocaleString()}${suffix}`;
    }
    return displayValue.toLocaleString();
  };

  return (
    <AnimatedSection animation="fade-up" delay={delay}>
      <div
        ref={ref}
        className={cn(
          "group relative p-6 rounded-2xl bg-gradient-to-b from-card/50 to-card/30",
          "border border-border/50 hover:border-primary/30",
          "transition-all duration-500 hover:shadow-lg hover:shadow-primary/10",
          "hover:-translate-y-1",
          className
        )}
      >
        {/* Glow effect on hover */}
        <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-primary/0 via-primary/5 to-accent/0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 blur-xl" />
        
        <div className="relative">
          {icon && (
            <div className="mb-3 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-primary/10 text-primary group-hover:scale-110 transition-transform duration-300">
              {icon}
            </div>
          )}
          
          <div className="text-3xl sm:text-4xl lg:text-5xl font-bold font-heading tracking-tight bg-gradient-to-r from-foreground via-foreground to-muted-foreground bg-clip-text text-transparent">
            {formatValue()}
          </div>
          
          <div className="mt-2 text-sm font-medium text-muted-foreground">
            {label}
          </div>
        </div>
      </div>
    </AnimatedSection>
  );
}
