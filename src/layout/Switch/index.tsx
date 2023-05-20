import { Switch as HeadlessSwitch } from "@headlessui/react";
import classNames from "classnames";

const themeColors = {
  "red-green": {
    inactive: "bg-red-600",
    active: "bg-green-600",
  },
  "yellow-blue": {
    inactive: "bg-yellow-300",
    active: "bg-blue-600",
  },
};

const sizes = {
  small: {
    container: "h-4 w-8 border-2",
    switch: "h-3 w-3",
    translateLeft: "translate-x-4",
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
  toggleIsEnabled,
  theme = "red-green",
  size = "medium",
}: {
  isEnabled: boolean;
  toggleIsEnabled: (checked: boolean) => void;
  theme?: "red-green" | "yellow-blue";
  size?: "small" | "medium" | "large";
}) {
  const selectedTheme = themeColors[theme];
  return (
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
  );
}
