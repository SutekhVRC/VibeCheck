import type { ChangeEventHandler } from "react";

// TODO convert into forward ref
export function Select({
  value,
  onChange,
  options,
  name,
}: {
  value: string;
  onChange: ChangeEventHandler<HTMLSelectElement>;
  options: readonly string[];
  name?: string;
}) {
  return (
    <select
      className="w-full rounded-sm px-2 text-zinc-800 outline-none"
      value={value}
      onChange={onChange}
      name={name}
    >
      {options.map((a) => (
        <option value={a} key={a}>
          {a}
        </option>
      ))}
    </select>
  );
}
