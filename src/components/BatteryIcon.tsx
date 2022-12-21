import {
  Battery0Icon,
  Battery100Icon,
  Battery50Icon,
} from "@heroicons/react/20/solid";
import Tooltip from "../layout/Tooltip";

export default function ({ battery }: { battery: number }) {
  return (
    <Tooltip text={`${battery * 100}%`}>
      <div
        style={{
          color: `hsl(${battery * 120}, 75%, 50%)`,
        }}
      >
        {battery <= 0.15 ? (
          <Battery0Icon className="h-8" />
        ) : battery <= 0.5 ? (
          <Battery50Icon className="h-8" />
        ) : (
          <Battery100Icon className="h-8" />
        )}
      </div>
    </Tooltip>
  );
}
