import type { FeVCToyFeature } from "../../../src-tauri/bindings/FeVCToyFeature";
import ToyFeatureForm from "./ToyFeatureForm";
import { ChevronUpIcon } from "@heroicons/react/20/solid";
import EnabledBadge from "../Toys/EnabledBadge";
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
            <div className="flex justify-between items-center pl-1 pr-1">
              <div className="flex items-center">
                <div>{`${feature.feature_type} ${feature.feature_index}`}</div>
                <EnabledBadge enabled={feature.feature_enabled} />
              </div>
              <ChevronUpIcon
                className={`${
                  open ? "rotate-180 transform" : ""
                } h-6 w-6 text-zinc-400`}
              />
            </div>
          </Disclosure.Button>
          <Disclosure.Panel>
            <ToyFeatureForm toyId={toyId} feature={feature} />
          </Disclosure.Panel>
        </>
      )}
    </Disclosure>
  );
}
