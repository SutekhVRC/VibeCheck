import { ReactNode } from "react";

export default function FourPanelContainer({
  children,
}: {
  children: ReactNode;
}) {
  return (
    <div className="grid grid-cols-2 items-center gap-x-0 gap-y-1 p-4 text-justify text-sm md:grid-cols-[minmax(6rem,_1fr)_minmax(1rem,_1fr)_minmax(4rem,_6fr)_minmax(2.5rem,_1fr)] md:gap-x-8 md:gap-y-2">
      {children}
    </div>
  );
}
