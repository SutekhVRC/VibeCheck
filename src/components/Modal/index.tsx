import { Dialog, Transition } from "@headlessui/react";
import { Fragment } from "react";
import type { ReactElement } from "react";

export default function MyModal({
  title,
  children,
  isOpen,
  onClose,
}: {
  title: string;
  children: ReactElement;
  isOpen: boolean;
  onClose: () => void;
}) {
  return (
    <>
      <Transition appear show={isOpen} as={Fragment}>
        <Dialog as="div" className="relative z-10" onClose={onClose}>
          <Transition.Child
            as={Fragment}
            enter="ease-out duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="fixed inset-0 bg-black bg-opacity-25" />
          </Transition.Child>

          <div className="fixed inset-0 overflow-y-auto">
            <div className="flex min-h-full items-center justify-center p-4">
              <Transition.Child
                as={Fragment}
                enter="ease-out duration-300"
                enterFrom="opacity-0 scale-95"
                enterTo="opacity-100 scale-100"
                leave="ease-in duration-200"
                leaveFrom="opacity-100 scale-100"
                leaveTo="opacity-0 scale-95"
              >
                <Dialog.Panel className="w-full max-w-md transform overflow-hidden rounded-xl transition-all p-6 bg-zinc-800">
                  <Dialog.Title as="h3" className="text-xl font-bold">
                    {title}
                  </Dialog.Title>
                  <div className="mt-2">
                    <p className="text-zinc-300">{children}</p>
                  </div>
                </Dialog.Panel>
              </Transition.Child>
            </div>
          </div>
        </Dialog>
      </Transition>
    </>
  );
}
