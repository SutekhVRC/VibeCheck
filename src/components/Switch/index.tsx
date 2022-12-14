import { Switch } from "@headlessui/react";
import Tooltip from "../Tooltip";

export default function ({
  isEnabled,
  toggleIsEnabled,
}: {
  isEnabled: boolean;
  toggleIsEnabled: () => void;
}) {
  return (
    <Tooltip text={isEnabled ? "Running" : "Off"} delay={500}>
      <Switch
        checked={isEnabled}
        onChange={toggleIsEnabled}
        className={`${isEnabled ? "bg-green-600" : "bg-red-600"}
            relative inline-flex h-6 w-10 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out`}
      >
        <span className="sr-only">Enable</span>
        <span
          aria-hidden="true"
          className={`${isEnabled ? "translate-x-4" : "translate-x-0"}
            inline-block h-5 w-5 rounded-full bg-white transition duration-200 ease-in-out`}
        />
      </Switch>
    </Tooltip>
  );
}