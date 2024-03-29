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

export type ObjectValues<T> = T[keyof T];
