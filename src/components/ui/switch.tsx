import * as React from "react";
import * as SwitchPrimitive from "@radix-ui/react-switch";
import { cn } from "@/lib/utils";

export const Switch = React.forwardRef<
  React.ElementRef<typeof SwitchPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof SwitchPrimitive.Root>
>(function Switch({ className, ...props }, ref) {
  return (
    <SwitchPrimitive.Root
      ref={ref}
      className={cn(
        "relative inline-flex h-[18px] w-[32px] shrink-0 items-center rounded-full border border-line transition-colors duration-150",
        "data-[state=unchecked]:bg-[color-mix(in_oklab,var(--text-1)_10%,transparent)]",
        "data-[state=checked]:border-transparent data-[state=checked]:bg-accent",
        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
        "disabled:opacity-40",
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb className="pointer-events-none block size-[14px] translate-x-[2px] rounded-full bg-white shadow-sm transition-transform duration-150 data-[state=checked]:translate-x-[16px] data-[state=checked]:bg-[var(--on-accent)]" />
    </SwitchPrimitive.Root>
  );
});
