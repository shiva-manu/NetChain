import { type HTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center gap-1.5 text-xs font-medium tracking-wide transition-colors duration-150",
  {
    variants: {
      variant: {
        default:
          "bg-primary text-primary-foreground",
        secondary:
          "bg-secondary text-secondary-foreground",
        outline:
          "border border-border text-foreground",
        destructive:
          "bg-destructive text-destructive-foreground",
        success:
          "bg-success/10 text-success border border-success/20",
        warning:
          "bg-warning/10 text-warning border border-warning/20",
        signal:
          "bg-primary/10 text-primary border border-primary/30",
        ghost:
          "text-muted-foreground",
        terminal:
          "border border-primary/30 text-primary bg-transparent",
        glass:
          "bg-foreground/5 backdrop-blur-sm text-foreground/80 border border-foreground/10",
      },
      size: {
        default: "h-6 px-2.5",
        sm: "h-5 px-2 text-[10px]",
        lg: "h-7 px-3",
        xl: "h-8 px-4",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export interface BadgeProps
  extends HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, size, ...props }: BadgeProps) {
  return (
    <div
      className={cn(badgeVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Badge, badgeVariants };
