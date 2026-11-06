import { ButtonHTMLAttributes, forwardRef } from "react";
import { cn } from "../lib/utils";

const variants = {
  // The one main action on a screen: accent gradient
  primary:
    "bg-gradient-to-r from-theme-primary to-theme-secondary text-theme-on-accent shadow-theme-button hover:brightness-110 disabled:from-theme-raised disabled:to-theme-raised",
  // Everything else: outlined, quieter
  secondary:
    "border border-theme-border bg-theme-raised text-theme-text hover:border-theme-secondary hover:text-theme-secondary disabled:hover:border-theme-border",
  // Destructive actions
  danger:
    "border border-theme-danger/60 bg-theme-danger/10 text-theme-danger hover:bg-theme-danger/20 disabled:hover:bg-theme-danger/10",
};

export type ButtonVariant = keyof typeof variants;

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
};

// forwardRef so Radix triggers (popovers, tooltips) can anchor to it
const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "primary", className, children, ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      {...props}
      className={cn(
        "m-2 rounded-full px-4 py-1 font-semibold transition-all duration-200 hover:-translate-y-0.5 active:translate-y-0 disabled:cursor-not-allowed disabled:text-theme-muted disabled:shadow-none disabled:hover:translate-y-0",
        variants[variant],
        className,
      )}
    >
      {children}
    </button>
  );
});

export default Button;
