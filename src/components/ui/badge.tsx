import * as React from "react";
import { cn } from "../../lib/utils";

type BadgeVariant = "default" | "subtle" | "danger";

const badgeVariants: Record<BadgeVariant, string> = {
  default: "bg-[var(--accent-soft)] text-[var(--accent)]",
  subtle: "bg-[var(--panel-muted)] text-[var(--muted-foreground)]",
  danger: "bg-[var(--danger-muted)] text-[var(--danger)]",
};

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

export function Badge({ className, variant = "subtle", ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium leading-none",
        badgeVariants[variant],
        className,
      )}
      {...props}
    />
  );
}
