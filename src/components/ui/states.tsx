import * as React from "react";
import { cn } from "@/lib/utils";

const PILL_TONES = {
  neutral: "text-text-2 border-line bg-[color-mix(in_oklab,var(--text-1)_4%,transparent)]",
  ok: "text-ok border-[color-mix(in_oklab,var(--ok)_30%,transparent)] bg-[color-mix(in_oklab,var(--ok)_10%,transparent)]",
  warn: "text-warn border-[color-mix(in_oklab,var(--warn)_30%,transparent)] bg-[color-mix(in_oklab,var(--warn)_10%,transparent)]",
  err: "text-err border-[color-mix(in_oklab,var(--err)_30%,transparent)] bg-[color-mix(in_oklab,var(--err)_10%,transparent)]",
  accent: "text-accent border-[color-mix(in_oklab,var(--accent)_35%,transparent)] bg-accent-soft",
} as const;

export function Pill({
  tone = "neutral",
  children,
  className,
  dot = false,
}: {
  tone?: keyof typeof PILL_TONES;
  children: React.ReactNode;
  className?: string;
  dot?: boolean;
}) {
  return (
    <span
      className={cn(
        "inline-flex h-[22px] items-center gap-1.5 rounded-full border px-2 text-[11px] font-medium",
        PILL_TONES[tone],
        className,
      )}
    >
      {dot ? <span className="size-1.5 rounded-full bg-current" /> : null}
      {children}
    </span>
  );
}

export function Empty({
  icon,
  title,
  description,
  action,
  className,
}: {
  icon?: React.ReactNode;
  title: string;
  description?: React.ReactNode;
  action?: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("flex flex-col items-center justify-center px-6 py-12 text-center", className)}>
      {icon ? (
        <div className="mb-3 flex size-9 items-center justify-center rounded-[10px] border border-line bg-[color-mix(in_oklab,var(--text-1)_4%,transparent)] text-text-3 [&_svg]:size-4">
          {icon}
        </div>
      ) : null}
      <div className="text-[13px] font-medium text-text-1">{title}</div>
      {description ? (
        <div className="mt-1 max-w-[340px] text-[12px] leading-relaxed text-text-3">{description}</div>
      ) : null}
      {action ? <div className="mt-4">{action}</div> : null}
    </div>
  );
}

export function Skeleton({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "animate-pulse rounded-[6px] bg-[color-mix(in_oklab,var(--text-1)_7%,transparent)]",
        className,
      )}
    />
  );
}

export function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="inline-flex h-[18px] min-w-[18px] items-center justify-center rounded-[4px] border border-line bg-[color-mix(in_oklab,var(--text-1)_5%,transparent)] px-1 font-mono text-[10px] text-text-3">
      {children}
    </kbd>
  );
}

export function Spinner({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "inline-block size-3.5 animate-spin rounded-full border-[1.5px] border-[color-mix(in_oklab,var(--text-1)_20%,transparent)] border-t-accent",
        className,
      )}
    />
  );
}
