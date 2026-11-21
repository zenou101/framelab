import * as React from "react";
import { Info } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import Sparkline from "@/components/charts/Sparkline";

export function MetricCard({
  label,
  value,
  unit,
  series,
  max,
  available = true,
  reason,
  source,
  method,
  footer,
  ceilingNote,
  className,
}: {
  label: string;
  value: string;
  unit?: string;
  series?: number[];
  max?: number;
  available?: boolean;
  reason?: string | null;
  source?: string;
  method?: string | null;
  footer?: React.ReactNode;
  ceilingNote?: string;
  className?: string;
}) {
  const detail = available
    ? [source, method, ceilingNote].filter(Boolean).join(" · ")
    : (reason ?? "Not available on this system");
  return (
    <div
      className={cn(
        "group relative overflow-hidden rounded-[var(--radius-card)] border border-line bg-card",
        "px-4 pt-3 pb-0 transition-colors hover:border-line-strong",
        className,
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="text-[10.5px] font-medium tracking-[0.055em] text-text-3 uppercase">{label}</span>
        <Tooltip content={detail}>
          <button
            type="button"
            aria-label={`${label} source`}
            className="text-text-3 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
          >
            <Info className="size-3" />
          </button>
        </Tooltip>
      </div>
      <div className="mt-1.5 flex items-baseline gap-1">
        {available ? (
          <>
            <span className="text-[25px] leading-none font-medium tracking-[-0.02em] tabular-nums text-text-1">
              {value}
            </span>
            {unit ? <span className="text-[11.5px] text-text-3">{unit}</span> : null}
          </>
        ) : (
          <span className="text-[13px] leading-tight text-text-3">Unavailable</span>
        )}
      </div>
      {available ? null : (
        <p className="mt-1 line-clamp-2 text-[10.5px] leading-snug text-text-3">{reason}</p>
      )}
      <div className="mt-2 -mx-4 h-[34px]">
        {series && series.length > 1 ? (
          <Sparkline values={series} max={max} className="w-full" height={34} />
        ) : (
          <div className="h-[34px] border-t border-line" />
        )}
      </div>
      {footer ? <div className="-mx-4 border-t border-line px-4 py-1.5 text-[10.5px] text-text-3">{footer}</div> : null}
    </div>
  );
}
