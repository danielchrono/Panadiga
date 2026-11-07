import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap rounded-xl text-xs font-mono font-bold ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-cyan-500 text-slate-950 hover:bg-cyan-400 shadow-md shadow-cyan-950/20",
        destructive: "bg-rose-500/10 text-rose-300 border border-rose-500/30 hover:bg-rose-500/20",
        outline: "border border-slate-700 bg-transparent text-slate-200 hover:bg-slate-800 hover:text-slate-100",
        secondary: "bg-slate-800 text-slate-200 hover:bg-slate-700",
        ghost: "hover:bg-slate-800 text-slate-400 hover:text-slate-100",
        purple: "bg-purple-600 text-white hover:bg-purple-500 shadow-md shadow-purple-950/50",
        emerald: "bg-emerald-600 text-white hover:bg-emerald-500 shadow-md shadow-emerald-950/50",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 rounded-lg px-3",
        lg: "h-10 rounded-xl px-6",
        icon: "h-9 w-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);
Button.displayName = "Button";

export { Button, buttonVariants };
