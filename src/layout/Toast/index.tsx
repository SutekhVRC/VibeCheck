import * as Toast from "@radix-ui/react-toast";
import { useState } from "react";
import "./Toast.css";

export default function ({
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
    <Toast.Provider swipeDirection="right" duration={1500}>
      <button
        className="rounded-sm border-zinc-400 border-solid border-2 py-1 px-2 hover:border-zinc-300 whitespace-pre-wrap text-xs"
        onClick={handleClick}
      >
        {buttonText}
      </button>
      <Toast.Root
        className="ToastRoot bg-zinc-100 text-zinc-900 rounded-md px-2 fixed right-2 bottom-2"
        open={open}
        onOpenChange={setOpen}
      >
        <Toast.Title>{toastText}</Toast.Title>
      </Toast.Root>
      <Toast.Viewport className="absolute" />
    </Toast.Provider>
  );
}
