import type { FeVCToyFeature } from "../../../src-tauri/bindings/FeVCToyFeature";
import FeatureForm from "./FeatureForm";
import { ChevronUpIcon } from "@heroicons/react/20/solid";
import { Disclosure } from "@headlessui/react";

export function Feature({
  toyId,
  feature,
}: {
  toyId: number;
  feature: FeVCToyFeature;
}) {
  return (
    <Disclosure>
      {({ open }) => (
        <>
          <Disclosure.Button>
            <div className="flex justify-between items-center pl-2 pr-2">
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
                className={`${
                  open ? "rotate-180 transform" : ""
                } h-6 w-6 text-zinc-400`}
              />
            </div>
          </Disclosure.Button>
          <Disclosure.Panel>
            <FeatureForm toyId={toyId} toyFeature={feature} />
          </Disclosure.Panel>
        </>
      )}
    </Disclosure>
  );
}
