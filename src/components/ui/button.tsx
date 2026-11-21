import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonStyles = cva(
  "relative inline-flex select-none items-center justify-center gap-2 whitespace-nowrap rounded-[8px] font-medium transition-[background-color,color,border-color,filter,opacity] duration-150 disabled:pointer-events-none disabled:opacity-40 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        primary: "bg-accent text-[var(--on-accent)] hover:brightness-110 active:brightness-95",
        secondary:
          "border border-line bg-elevated text-text-1 hover:border-line-strong hover:bg-[color-mix(in_oklab,var(--text-1)_4%,var(--elevated))]",
        ghost: "text-text-2 hover:bg-[color-mix(in_oklab,var(--text-1)_6%,transparent)] hover:text-text-1",
        subtle:
          "bg-[color-mix(in_oklab,var(--text-1)_6%,transparent)] text-text-1 hover:bg-[color-mix(in_oklab,var(--text-1)_10%,transparent)]",
        danger:
          "border border-[color-mix(in_oklab,var(--err)_38%,transparent)] text-err hover:bg-[color-mix(in_oklab,var(--err)_12%,transparent)]",
      },
      size: {
        xs: "h-[26px] px-2 text-[11.5px] [&_svg]:size-3.5",
        sm: "h-7 px-2.5 text-[12px] [&_svg]:size-3.5",
        md: "h-8 px-3 text-[12.5px] [&_svg]:size-4",
        lg: "h-9 px-4 text-[13px] [&_svg]:size-4",
        icon: "h-8 w-8 [&_svg]:size-4",
        iconSm: "h-7 w-7 [&_svg]:size-3.5",
      },
    },
    defaultVariants: { variant: "secondary", size: "md" },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonStyles> {
  asChild?: boolean;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { className, variant, size, asChild = false, ...props },
  ref,
) {
  const Comp = asChild ? Slot : "button";
  return <Comp ref={ref} className={cn(buttonStyles({ variant, size }), className)} {...props} />;
});
