import { clsx, ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export const percent = new Intl.NumberFormat("en-US", {
  style: "percent",
  minimumFractionDigits: 0,
  maximumFractionDigits: 0,
});

export const round0 = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 0,
  maximumFractionDigits: 0,
});

export function assertExhaustive(e: never): never {
  throw new Error("Non-Exhaustive switch statement", e);
}

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
