import { ReactNode } from "react";
import { TooltipLabel } from "../layout/Tooltip";
import { ArrowsRightLeftIcon } from "@heroicons/react/24/solid";

export default function FourPanel({
  text,
  tooltip,
  flipped,
  two,
  three,
  four,
}: {
  text: string;
  tooltip?: string;
  flipped?: boolean;
  two?: ReactNode | null;
  three?: ReactNode | null;
  four?: string | null;
}) {
  const label = tooltip ? (
    <TooltipLabel text={text} tooltip={tooltip} />
  ) : (
    <label>{text}</label>
  );
  return (
    <>
      {flipped ? (
        <div className="flex items-center gap-2">
          {label}
          <ArrowsRightLeftIcon className="h-4" />
        </div>
      ) : (
        label
      )}
      {two ?? <div></div>}
      {three ?? <div></div>}
      {four ? <div className="text-right">{four}</div> : <div></div>}
    </>
  );
}
