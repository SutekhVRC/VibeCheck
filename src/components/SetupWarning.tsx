import { cn } from "@/lib/utils";
import { AlertTriangle } from "lucide-react";
import { ReactNode } from "react";

/**
 * A setup problem found by comparing a setting with the current avatar,
 * shown under the setting. Warns rather than blocks: the avatar may simply
 * be a different one from the one the toy was set up for.
 */
export default function SetupWarning({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      role="status"
      className={cn(
        "mt-1.5 flex items-start gap-1.5 text-xs text-theme-warning",
        className,
      )}
    >
      <AlertTriangle className="mt-px h-3.5 w-3.5 shrink-0" />
      <span>{children}</span>
    </div>
  );
}
