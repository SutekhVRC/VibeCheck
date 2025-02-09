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
          <div className="absolute top-0 right-0 h-2 w-2 animate-ping rounded-full bg-green-300" />
          <div className="absolute top-0 right-0 h-2 w-2 rounded-full bg-green-300" />
        </>
      )}
    </div>
  );
}
