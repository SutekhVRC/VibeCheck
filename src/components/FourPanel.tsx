import { ArrowRightLeft } from "lucide-react";
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
  two?: ReactNode;
  three?: ReactNode;
  four?: ReactNode;
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
          <ArrowRightLeft className="h-4" />
        </div>
      ) : (
        label
      )}
      <div>{two}</div>
      <div className="col-span-2 flex w-full flex-col text-right md:col-span-1">
        {three}
      </div>
      <div className="hidden text-right md:inline">{four}</div>
    </>
  );
}
