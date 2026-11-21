import * as React from "react";
import { cn } from "@/lib/utils";

export function Panel({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("rounded-[var(--radius-card)] border border-line bg-card", className)}
      {...props}
    />
  );
}

export function PanelHeader({
  title,
  hint,
  action,
  className,
}: {
  title: React.ReactNode;
  hint?: React.ReactNode;
  action?: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("flex items-start justify-between gap-4 px-4 pt-3.5 pb-3", className)}>
      <div className="min-w-0">
        <div className="text-[12.5px] font-medium tracking-[-0.01em] text-text-1">{title}</div>
        {hint ? <div className="mt-0.5 text-[11.5px] text-text-3">{hint}</div> : null}
      </div>
      {action ? <div className="shrink-0">{action}</div> : null}
    </div>
  );
}

export function SectionLabel({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={cn("flex items-center gap-3", className)}>
      <span className="text-[10.5px] font-medium tracking-[0.06em] text-text-3 uppercase">{children}</span>
      <span className="h-px flex-1 bg-line" />
    </div>
  );
}
