import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { XCircle } from "lucide-react";
import * as React from "react";
import { toast } from "sonner";

export interface FreeTextOptionProps extends Omit<
  React.HTMLAttributes<HTMLDivElement>,
  "onChange"
> {
  values: string[];
  onChange: (values: string[]) => void;
  placeholder?: string;
  transform?: (input: string) => string;
  validator?: {
    re: RegExp;
    message: string;
  };
  disabled?: boolean;
  /** Values to mark with a warning colour, e.g. ones that match nothing */
  warnValues?: string[];
  /** Tooltip for a marked value */
  warnTooltip?: string;
  /** Extra content after a value's text and a tooltip for it, if any */
  decorate?: (value: string) => ValueDecoration | null | undefined;
}

/** What `decorate` adds to a value */
export type ValueDecoration = {
  /** Shown after the value's text, e.g. an icon */
  after?: React.ReactNode;
  /** The value's tooltip, unless it's marked with a warning */
  title?: string;
};

export const FreeTextOptions = React.forwardRef<
  HTMLDivElement,
  FreeTextOptionProps
>(
  (
    {
      values,
      onChange,
      placeholder = "Add item...",
      transform,
      validator,
      disabled = false,
      warnValues = [],
      warnTooltip,
      decorate,
      className,
      ...props
    },
    ref,
  ) => {
    const [inputValue, setInputValue] = React.useState("");
    const inputRef = React.useRef<HTMLInputElement | null>(null);

    const addItem = () => {
      const trimmed = inputValue.trim();
      const parsed = transform ? transform(trimmed) : trimmed;
      if (!parsed || disabled) return;
      if (values.includes(parsed)) {
        toast.error("Duplicate items not allowed");
        return;
      }
      if (validator && !validator.re.test(parsed)) {
        toast.error(`Invalid input: ${validator.message}`);
        return;
      }
      onChange([...values, parsed]);
      setInputValue("");
      if (inputRef.current) inputRef.current.focus();
    };

    const removeItem = (value: string) => {
      if (disabled) return;
      onChange(values.filter((v) => v !== value));
      if (inputRef.current) inputRef.current.focus();
    };

    const onKeyDown: React.KeyboardEventHandler<HTMLInputElement> = (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        addItem();
      } else if (e.key === "Backspace" && inputValue === "") {
        if (values.length === 0) return;
        e.preventDefault();
        removeItem(values[values.length - 1]);
      }
    };

    return (
      <div
        ref={ref}
        className={cn(
          "theme-input flex flex-wrap items-center gap-2 px-2 py-1 focus-within:border-theme-primary focus-within:ring-2 focus-within:ring-theme-primary/30",
          className,
        )}
        {...props}
      >
        {values.map((v) => {
          const warned = warnValues.includes(v);
          const decoration = decorate?.(v);
          return (
            <Badge
              key={v}
              className={cn(
                "inline-flex items-center gap-2",
                warned &&
                  "bg-theme-warning/20 text-theme-warning ring-1 ring-theme-warning/60",
              )}
              title={warned ? warnTooltip : decoration?.title}
            >
              <span className="inline-flex max-w-[200px] items-center gap-1">
                <span className="truncate">{v}</span>
                {decoration?.after}
              </span>
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  removeItem(v);
                }}
                aria-label={`Remove ${v}`}
                className="ml-1 rounded-sm p-0.5 focus:outline-none focus:ring-1 focus:ring-ring"
              >
                <XCircle className="h-4 w-4" />
              </button>
            </Badge>
          );
        })}

        <input
          ref={inputRef}
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder={placeholder}
          aria-label={placeholder}
          disabled={disabled}
          className="min-w-[140px] flex-1 appearance-none bg-transparent p-0.5 text-theme-text outline-none placeholder:text-theme-muted"
        />
      </div>
    );
  },
);
FreeTextOptions.displayName = "FreeTextOptions";
