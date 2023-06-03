import { Switch as HeadlessSwitch, SwitchProps } from "@headlessui/react";
import classNames from "classnames";

const themeColors = {
  "white-cyan": {
    inactive: "bg-slate-300",
    active: "bg-cyan-600",
  },
  "red-green": {
    inactive: "bg-red-600",
    active: "bg-green-600",
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

export default function Switch(
  props: SwitchProps<"button"> & {
    theme?: keyof typeof themeColors;
    size?: keyof typeof sizes;
  }
) {
  const theme = props.theme ?? "white-cyan";
  const size = props.size ?? "medium";
  const selectedTheme = themeColors[theme];
  return (
    <HeadlessSwitch
      {...props}
      className={classNames(
        props.checked ? selectedTheme.active : selectedTheme.inactive,
        sizes[size].container,
        "relative inline-flex cursor-pointer rounded-full border-transparent transition-colors duration-200 ease-in-out disabled:bg-gray-700 disabled:cursor-not-allowed"
      )}
    >
      <span className="sr-only">Enable</span>
      <span
        aria-hidden="true"
        className={classNames(
          props.checked ? sizes[size].translateLeft : "translate-x-0",
          sizes[size].switch,
          props.disabled ? "bg-gray-600" : "bg-white",
          "inline-block rounded-full transition duration-200 ease-in-out"
        )}
      />
    </HeadlessSwitch>
  );
}
