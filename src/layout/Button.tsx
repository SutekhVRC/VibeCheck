import type { ButtonHTMLAttributes } from "react";

export default function Button(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      className="m-2 rounded-md bg-zinc-400 px-4 py-1 text-zinc-900 hover:bg-zinc-100 disabled:cursor-not-allowed disabled:bg-zinc-600 disabled:text-zinc-400"
    >
      {props.children}
    </button>
  );
}
