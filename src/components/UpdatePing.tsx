import type { ReactNode } from "react";

export default function UpdatePing({
  children,
  canUpdate,
}: {
  children: ReactNode;
  canUpdate: boolean;
}) {
  return (
    <div className={`relative select-none ${canUpdate && "cursor-pointer"}`}>
      {children}
      {canUpdate && (
        <>
          <div className="absolute top-0 right-0 w-2 h-2 rounded-full bg-green-300 animate-ping" />
          <div className="absolute top-0 right-0 w-2 h-2 rounded-full bg-green-300" />
        </>
      )}
    </div>
  );
}
