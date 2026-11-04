import { PawPrint } from "lucide-react";

const PAW_DELAYS = ["0ms", "200ms", "400ms"];

// Three little paws hopping in turn
export default function Loading() {
  return (
    <div className="ml-2 flex items-center gap-0.5">
      {PAW_DELAYS.map((delay) => (
        <PawPrint
          key={delay}
          className="h-3 w-3 animate-bounce"
          style={{ animationDelay: delay }}
          strokeWidth={2.5}
        />
      ))}
    </div>
  );
}
