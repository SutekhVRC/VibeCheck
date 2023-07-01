import { ChangeEventHandler } from "react";

export function Select({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: ChangeEventHandler<HTMLSelectElement>;
  options: readonly string[];
}) {
  return (
    <select
      className="outline-none text-zinc-800 px-2 rounded-sm w-full"
      value={value}
      onChange={onChange}
    >
      {options.map((a) => (
        <option value={a} key={a}>
          {a}
        </option>
      ))}
    </select>
  );
}
