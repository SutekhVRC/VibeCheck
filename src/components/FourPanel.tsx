import { ReactNode } from "react";

export default function FourPanel({
  one,
  two,
  three,
  four,
}: {
  one: ReactNode;
  two?: ReactNode | null;
  three?: ReactNode | null;
  four?: ReactNode | null;
}) {
  return (
    <>
      {one}
      {two ?? <div></div>}
      {three ?? <div></div>}
      {four ?? <div></div>}
    </>
  );
}
