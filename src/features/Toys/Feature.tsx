import type { FeVCToyFeature } from "../../../src-tauri/bindings/FeVCToyFeature";
import FeatureForm from "./FeatureForm";
import { ChevronUpIcon } from "@heroicons/react/20/solid";
import classNames from "classnames";
import { AnimatePresence } from "framer-motion";
import { useState } from "react";

export function Feature({
  toyId,
  feature,
}: {
  toyId: number;
  feature: FeVCToyFeature;
}) {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <>
      <button
        className="flex justify-between items-center pl-2 pr-2"
        onClick={() => setIsOpen((b) => !b)}
      >
        <div className="flex items-center">
          <div
            className={
              feature.feature_enabled ? "text-gray-200" : "text-gray-500"
            }
          >
            {feature.feature_type} {feature.feature_index}
          </div>
        </div>
        <ChevronUpIcon
          className={classNames(
            isOpen && "rotate-180 transform",
            "h-6 w-6 text-zinc-400"
          )}
        />
      </button>
      <AnimatePresence>
        {isOpen && <FeatureForm toyId={toyId} toyFeature={feature} />}
      </AnimatePresence>
    </>
  );
}
