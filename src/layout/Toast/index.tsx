import * as RadixToast from "@radix-ui/react-toast";
import { useState } from "react";

export default function Toast({
  buttonText,
  title,
  description,
  onClick,
}: {
  buttonText: string;
  title: string;
  description: string;
  onClick: () => void;
}) {
  const [open, setOpen] = useState(false);

  function handleClick() {
    onClick();
    setOpen(true);
  }

  return (
    <RadixToast.Provider swipeDirection="right" duration={1500}>
      <button
        className="rounded-sm border-zinc-400 border-solid border-2 py-1 px-2 hover:border-zinc-300 whitespace-pre-wrap text-xs"
        onClick={handleClick}
      >
        {buttonText}
      </button>
      <RadixToast.Root
        className="bg-zinc-100 text-zinc-900 rounded-md px-2 fixed right-2 bottom-2 z-50"
        open={open}
        onOpenChange={setOpen}
      >
        <RadixToast.Title>{title}</RadixToast.Title>
        <RadixToast.Description>{description}</RadixToast.Description>
      </RadixToast.Root>
      <RadixToast.Viewport className="absolute" />
    </RadixToast.Provider>
  );
}
