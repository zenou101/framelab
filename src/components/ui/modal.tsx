import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function Modal({
  open,
  onOpenChange,
  title,
  description,
  children,
  footer,
  width = 460,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: React.ReactNode;
  description?: React.ReactNode;
  children?: React.ReactNode;
  footer?: React.ReactNode;
  width?: number;
}) {
  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="anim-fade fixed inset-0 z-40 bg-black/50 backdrop-blur-[2px]" />
        <DialogPrimitive.Content
          style={{ width }}
          className={cn(
            "anim-rise fixed top-1/2 left-1/2 z-50 -translate-x-1/2 -translate-y-1/2",
            "max-h-[86vh] overflow-y-auto rounded-[14px] border border-line bg-elevated shadow-[var(--shadow-pop)]",
          )}
        >
          <div className="flex items-start justify-between gap-6 px-5 pt-4 pb-3">
            <div>
              <DialogPrimitive.Title className="text-[14px] font-medium text-text-1">{title}</DialogPrimitive.Title>
              {description ? (
                <DialogPrimitive.Description className="mt-1 text-[12px] leading-relaxed text-text-2">
                  {description}
                </DialogPrimitive.Description>
              ) : null}
            </div>
            <DialogPrimitive.Close asChild>
              <Button variant="ghost" size="iconSm" aria-label="Close dialog">
                <X />
              </Button>
            </DialogPrimitive.Close>
          </div>
          {children ? <div className="px-5 pb-4">{children}</div> : null}
          {footer ? (
            <div className="flex items-center justify-end gap-2 border-t border-line px-5 py-3">{footer}</div>
          ) : null}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}
