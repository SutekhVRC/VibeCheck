import * as RadixToast from "@radix-ui/react-toast";
import { useState } from "react";
import "./Toast.css";

export default function Toast({
  buttonText,
  toastText,
  onClick,
}: {
  buttonText: string;
  toastText: string;
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
        className="bg-zinc-100 text-zinc-900 rounded-md px-2 fixed right-2 bottom-2"
        open={open}
        onOpenChange={setOpen}
      >
        <RadixToast.Title>{toastText}</RadixToast.Title>
      </RadixToast.Root>
      <RadixToast.Viewport className="absolute" />
    </RadixToast.Provider>
  );
}
