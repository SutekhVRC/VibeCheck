import { Switch as HeadlessSwitch } from "@headlessui/react";
import Tooltip from "../Tooltip";
import classNames from "classnames";

const themeColors = {
  "red-green": {
    active: "bg-red-600",
    inactive: "bg-green-600",
  },
  "blue-yellow": {
    active: "bg-blue-600",
    inactive: "bg-yellow-600",
  },
};

const sizes = {
  small: {
    container: "h-4 w-6 border-2",
    switch: "h-3 w-3",
    translateLeft: "translate-x-2",
  },
  medium: {
    container: "h-6 w-12 border-2",
    switch: "h-5 w-5",
    translateLeft: "translate-x-6",
  },
  large: {
    container: "h-10 w-20 border-4",
    switch: "h-8 w-8",
    translateLeft: "translate-x-10",
  },
};

export default function Switch({
  isEnabled,
  theme = "red-green",
  size = "medium",
  toggleIsEnabled,
}: {
  isEnabled: boolean;
  theme?: "red-green" | "blue-yellow";
  size?: "small" | "medium" | "large";
  toggleIsEnabled: () => void;
}) {
  const selectedTheme = themeColors[theme];
  return (
    <Tooltip text={isEnabled ? "OSC Enabled" : "OSC Disabled"} delay={250}>
      <HeadlessSwitch
        checked={isEnabled}
        onChange={toggleIsEnabled}
        className={classNames(
          isEnabled ? selectedTheme.active : selectedTheme.inactive,
          sizes[size].container,
          "relative inline-flex cursor-pointer rounded-full border-transparent transition-colors duration-200 ease-in-out"
        )}
      >
        <span className="sr-only">Enable</span>
        <span
          aria-hidden="true"
          className={classNames(
            isEnabled ? sizes[size].translateLeft : "translate-x-0",
            sizes[size].switch,
            "inline-block rounded-full bg-white transition duration-200 ease-in-out"
          )}
        />
      </HeadlessSwitch>
    </Tooltip>
  );
}
