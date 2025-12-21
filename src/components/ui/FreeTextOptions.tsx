import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { XCircle } from "lucide-react";
import * as React from "react";

export interface FreeTextOptionProps extends Omit<
  React.HTMLAttributes<HTMLDivElement>,
  "onChange"
> {
  values: string[];
  onChange: (values: string[]) => void;
  placeholder?: string;
  validator?: RegExp;
  disabled?: boolean;
}

export const FreeTextOptions = React.forwardRef<
  HTMLDivElement,
  FreeTextOptionProps
>(
  (
    {
      values,
      onChange,
      placeholder = "Add item...",
      validator,
      disabled = false,
      className,
      ...props
    },
    ref,
  ) => {
    const [inputValue, setInputValue] = React.useState("");
    const [announce, setAnnounce] = React.useState("");
    const inputRef = React.useRef<HTMLInputElement | null>(null);

    React.useEffect(() => {
      if (!announce) return;
      const t = setTimeout(() => setAnnounce(""), 1000);
      return () => clearTimeout(t);
    }, [announce]);

    const addItem = () => {
      const trimmed = inputValue.trim();
      if (!trimmed || disabled) return;
      if (values.includes(trimmed)) {
        setAnnounce("Duplicate items not allowed");
        return;
      }
      if (validator && !validator.test(trimmed)) {
        setAnnounce("Invalid input");
        return;
      }
      onChange([...values, trimmed]);
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
        className={cn("flex flex-wrap items-center gap-2", className)}
        {...props}
      >
        <div className="sr-only" aria-live="polite">
          {announce}
        </div>

        {values.map((v) => (
          <Badge key={v} className="inline-flex items-center gap-2">
            <span className="max-w-[200px] truncate">{v}</span>
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
        ))}

        <input
          ref={inputRef}
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder={placeholder}
          aria-label={placeholder}
          disabled={disabled}
          className="min-w-[140px] flex-1 bg-transparent p-1 outline-none"
        />
      </div>
    );
  },
);
FreeTextOptions.displayName = "FreeTextOptions";