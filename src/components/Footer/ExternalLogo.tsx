export default function ({ src, link }: { src: string; link: string }) {
  return (
    <img
      className="max-h-8 cursor-pointer"
      src={src}
      onClick={() => open(link)}
    />
  );
}
