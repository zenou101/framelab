import * as React from "react";
import * as DropdownPrimitive from "@radix-ui/react-dropdown-menu";
import { cn } from "@/lib/utils";

export type MenuEntry =
  | {
      kind?: "item";
      label: string;
      icon?: React.ReactNode;
      onSelect: () => void;
      destructive?: boolean;
      disabled?: boolean;
    }
  | { kind: "separator" };

export function Menu({
  trigger,
  entries,
  align = "end",
}: {
  trigger: React.ReactNode;
  entries: MenuEntry[];
  align?: "start" | "end";
}) {
  return (
    <DropdownPrimitive.Root>
      <DropdownPrimitive.Trigger asChild>{trigger}</DropdownPrimitive.Trigger>
      <DropdownPrimitive.Portal>
        <DropdownPrimitive.Content
          align={align}
          sideOffset={5}
          className="anim-pop z-50 min-w-[180px] rounded-[10px] border border-line bg-elevated p-1 shadow-[var(--shadow-pop)]"
        >
          {entries.map((entry, index) =>
            entry.kind === "separator" ? (
              <DropdownPrimitive.Separator key={`sep-${index}`} className="my-1 h-px bg-line" />
            ) : (
              <DropdownPrimitive.Item
                key={`${entry.label}-${index}`}
                disabled={entry.disabled}
                onSelect={entry.onSelect}
                className={cn(
                  "flex cursor-default select-none items-center gap-2 rounded-[6px] px-2 py-1.5 text-[12.5px] outline-none data-[disabled]:opacity-40 [&_svg]:size-3.5",
                  entry.destructive
                    ? "text-err data-[highlighted]:bg-[color-mix(in_oklab,var(--err)_12%,transparent)]"
                    : "text-text-2 data-[highlighted]:bg-[color-mix(in_oklab,var(--text-1)_7%,transparent)] data-[highlighted]:text-text-1",
                )}
              >
                {entry.icon}
                {entry.label}
              </DropdownPrimitive.Item>
            ),
          )}
        </DropdownPrimitive.Content>
      </DropdownPrimitive.Portal>
    </DropdownPrimitive.Root>
  );
}
