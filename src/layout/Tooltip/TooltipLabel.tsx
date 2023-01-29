import Tooltip from ".";

export default function TooltipLabel({
  text,
  tooltip,
}: {
  text: string;
  tooltip: string;
}) {
  return (
    <Tooltip text={tooltip}>
      <label className="justify-self-start">{text}</label>
    </Tooltip>
  );
}
