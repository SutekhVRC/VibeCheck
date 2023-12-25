export default function Loading() {
  return (
    <>
      <div
        className="-translate-y-1/4 transform animate-bounce"
        style={{
          animationDelay: "250ms",
        }}
      >
        .
      </div>
      <div
        className="-translate-y-1/4 transform animate-bounce"
        style={{
          animationDelay: "500ms",
        }}
      >
        .
      </div>
      <div
        className="-translate-y-1/4 transform animate-bounce"
        style={{
          animationDelay: "750ms",
        }}
      >
        .
      </div>
    </>
  );
}
