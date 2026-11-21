import * as React from "react";
import { cn } from "@/lib/utils";

export function Segmented<T extends string>({
  value,
  onChange,
  options,
  className,
  ariaLabel,
}: {
  value: T;
  onChange: (value: T) => void;
  options: { value: T; label: React.ReactNode; title?: string }[];
  className?: string;
  ariaLabel?: string;
}) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className={cn(
        "inline-flex h-7 items-center gap-0.5 rounded-[8px] border border-line bg-[color-mix(in_oklab,var(--text-1)_4%,transparent)] p-0.5",
        className,
      )}
    >
      {options.map((option) => {
        const active = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            title={option.title}
            aria-pressed={active}
            onClick={() => onChange(option.value)}
            className={cn(
              "inline-flex h-6 items-center rounded-[6px] px-2 text-[11.5px] font-medium transition-colors",
              active
                ? "bg-elevated text-text-1 shadow-[0_1px_2px_rgba(0,0,0,0.18)]"
                : "text-text-3 hover:text-text-2",
            )}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
