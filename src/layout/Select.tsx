import { ChangeEventHandler } from "react";

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
      className="outline-none text-zinc-800 px-2 rounded-sm w-full"
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
