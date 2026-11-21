import * as SelectPrimitive from "@radix-ui/react-select";
import { Check, ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";

export interface SelectOption<T extends string> {
  value: T;
  label: string;
  hint?: string;
  disabled?: boolean;
}

export function Select<T extends string>({
  value,
  onValueChange,
  options,
  placeholder = "Select…",
  className,
  ariaLabel,
  size = "md",
}: {
  value: T | null;
  onValueChange: (value: T) => void;
  options: SelectOption<T>[];
  placeholder?: string;
  className?: string;
  ariaLabel?: string;
  size?: "sm" | "md";
}) {
  return (
    <SelectPrimitive.Root value={value ?? undefined} onValueChange={(v) => onValueChange(v as T)}>
      <SelectPrimitive.Trigger
        aria-label={ariaLabel}
        className={cn(
          "flex w-full items-center justify-between gap-2 rounded-[8px] border border-line bg-elevated px-2.5 text-left text-[12.5px] text-text-1 transition-colors",
          "hover:border-line-strong data-[placeholder]:text-text-3",
          "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
          "disabled:opacity-40",
          size === "sm" ? "h-7" : "h-8",
          className,
        )}
      >
        <SelectPrimitive.Value placeholder={<span className="text-text-3">{placeholder}</span>} />
        <SelectPrimitive.Icon>
          <ChevronDown className="size-3.5 text-text-3" />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          position="popper"
          sideOffset={4}
          className="anim-pop z-50 max-h-[320px] min-w-[--radix-select-trigger-width] overflow-hidden rounded-[10px] border border-line bg-elevated p-1 shadow-[var(--shadow-pop)]"
        >
          <SelectPrimitive.Viewport>
            {options.map((option) => (
              <SelectPrimitive.Item
                key={option.value}
                value={option.value}
                disabled={option.disabled}
                className={cn(
                  "relative flex cursor-default select-none flex-col gap-0.5 rounded-[6px] px-2 py-1.5 pr-7 text-[12.5px] text-text-2 outline-none",
                  "data-[highlighted]:bg-[color-mix(in_oklab,var(--text-1)_7%,transparent)] data-[highlighted]:text-text-1",
                  "data-[state=checked]:text-text-1 data-[disabled]:opacity-40",
                )}
              >
                <SelectPrimitive.ItemText>{option.label}</SelectPrimitive.ItemText>
                {option.hint ? <span className="text-[11px] text-text-3">{option.hint}</span> : null}
                <SelectPrimitive.ItemIndicator className="absolute top-1.5 right-2">
                  <Check className="size-3.5 text-accent" />
                </SelectPrimitive.ItemIndicator>
              </SelectPrimitive.Item>
            ))}
          </SelectPrimitive.Viewport>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
}
