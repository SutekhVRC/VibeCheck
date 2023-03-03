import Tooltip from ".";

type TooltipProps = {
  text: string;
  tooltip: string;
};

export default function TooltipLabel({ text, tooltip }: TooltipProps) {
  return (
    <Tooltip text={tooltip}>
      <label className="justify-self-start">{text}</label>
    </Tooltip>
  );
}
