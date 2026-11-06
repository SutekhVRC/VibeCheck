import { cn } from "@/lib/utils";
import { ReactNode } from "react";

/**
 * Heading for a main-panel page: an icon in a soft tinted tile, a title in
 * solid text and an optional muted subtitle, with optional actions on the
 * right. Solid text (not gradient-clipped) so descenders never get cut off.
 */
export default function PageHeader({
  icon,
  title,
  subtitle,
  actions,
  className,
}: {
  icon: ReactNode;
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex shrink-0 flex-wrap items-center justify-between gap-3",
        className,
      )}
    >
      <div className="flex min-w-0 items-center gap-3">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-theme-primary/15 text-theme-primary">
          {icon}
        </div>
        <div className="min-w-0 text-left">
          <h1 className="truncate text-2xl font-bold leading-snug text-theme-text">
            {title}
          </h1>
          {subtitle && (
            <p className="truncate text-sm text-theme-muted">{subtitle}</p>
          )}
        </div>
      </div>
      {actions}
    </div>
  );
}
