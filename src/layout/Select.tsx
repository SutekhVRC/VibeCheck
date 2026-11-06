import ThemeIcon from "@/theme/ThemeIcon";
import * as SelectPrimitive from "@radix-ui/react-select";
import { ChevronDown, ChevronUp } from "lucide-react";
import { cn } from "../lib/utils";

/*
 * Themed dropdown built on Radix Select.
 * Unlike a native <select>, the open list is rendered by us (not WebKitGTK),
 * so it follows the pastel theme on every platform.
 */
export function Select<T extends string>({
  value,
  onValueChange,
  options,
  name,
}: {
  value: T;
  onValueChange: (value: T) => void;
  options: readonly T[];
  name?: string;
}) {
  return (
    <SelectPrimitive.Root
      value={value}
      onValueChange={(v) => onValueChange(v as T)}
      name={name}
    >
      <SelectPrimitive.Trigger
        className="theme-input group flex w-full cursor-pointer items-center justify-between gap-2 text-left data-[state=open]:border-theme-primary data-[state=open]:ring-2 data-[state=open]:ring-theme-primary/30"
        aria-label={name}
      >
        <span className="truncate">
          <SelectPrimitive.Value />
        </span>
        <SelectPrimitive.Icon asChild>
          <ChevronDown className="h-4 w-4 shrink-0 text-theme-secondary transition-transform duration-200 group-data-[state=open]:rotate-180" />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>

      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          position="popper"
          sideOffset={6}
          className="z-50 max-h-[min(18rem,var(--radix-select-content-available-height))] min-w-[var(--radix-select-trigger-width)] overflow-hidden rounded-2xl border border-theme-border bg-theme-surface text-sm text-theme-text shadow-theme-pop data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95"
        >
          <SelectPrimitive.ScrollUpButton className="flex justify-center py-1 text-theme-secondary">
            <ChevronUp className="h-4 w-4" />
          </SelectPrimitive.ScrollUpButton>
          <SelectPrimitive.Viewport className="p-1">
            {options.map((option) => (
              <SelectPrimitive.Item
                key={option}
                value={option}
                className={cn(
                  "relative flex cursor-pointer select-none items-center rounded-xl py-1 pl-8 pr-3 outline-none transition-colors",
                  "data-[highlighted]:bg-theme-primary/20 data-[highlighted]:text-theme-primary",
                  "data-[state=checked]:font-semibold",
                )}
              >
                <SelectPrimitive.ItemIndicator className="absolute left-2.5 flex items-center">
                  <ThemeIcon className="h-3.5 w-3.5 text-theme-primary" />
                </SelectPrimitive.ItemIndicator>
                <SelectPrimitive.ItemText>{option}</SelectPrimitive.ItemText>
              </SelectPrimitive.Item>
            ))}
          </SelectPrimitive.Viewport>
          <SelectPrimitive.ScrollDownButton className="flex justify-center py-1 text-theme-secondary">
            <ChevronDown className="h-4 w-4" />
          </SelectPrimitive.ScrollDownButton>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
}
