import { ButtonHTMLAttributes } from "react";

export default function Button(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      className="rounded-md bg-zinc-400 px-4 py-1 text-zinc-900 hover:bg-zinc-100 disabled:text-zinc-400 disabled:bg-zinc-600 m-2"
    >
      {props.children}
    </button>
  );
}
