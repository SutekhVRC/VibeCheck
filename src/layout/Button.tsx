import { ButtonHTMLAttributes } from "react";

export default function Button(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      className="m-2 rounded-full bg-gradient-to-r from-cute-pink to-cute-lilac px-4 py-1 font-semibold text-cute-night shadow-[0_4px_14px_-4px_rgba(247,178,217,0.6)] transition-all duration-200 hover:-translate-y-0.5 hover:brightness-110 active:translate-y-0 disabled:cursor-not-allowed disabled:from-cute-raised disabled:to-cute-raised disabled:text-cute-muted disabled:shadow-none disabled:hover:translate-y-0"
    >
      {props.children}
    </button>
  );
}
