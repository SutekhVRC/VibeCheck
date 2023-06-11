import { ChangeEventHandler } from "react";

export function Select({
  defaultValue,
  onChange,
  options,
}: {
  defaultValue: string;
  onChange: ChangeEventHandler<HTMLSelectElement>;
  options: readonly string[];
}) {
  return (
    <select
      className="outline-none text-zinc-800 px-2 rounded-sm"
      defaultValue={defaultValue}
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
