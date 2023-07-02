import { ReactNode } from "react";

export default function FourPanelContainer({
  children,
}: {
  children: ReactNode;
}) {
  return (
    <div className="grid grid-cols-2 md:grid-cols-[minmax(5rem,_1fr)_minmax(6rem,_1fr)_minmax(4rem,_6fr)_minmax(2.5rem,_1fr)] text-sm text-justify p-4 gap-y-1 md:gap-y-2 gap-x-0 md:gap-x-8 items-center">
      {children}
    </div>
  );
}
