import { ArrowsRightLeftIcon } from "@heroicons/react/24/solid";
import { ReactNode } from "react";
import { TooltipLabel } from "../layout/Tooltip";

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
      <div>{two}</div>
      <div className="text-right w-full col-span-2 md:col-span-1">{three}</div>
      <div className="text-right hidden md:inline">{four}</div>
    </>
  );
}
