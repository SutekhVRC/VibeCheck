export default function ({ enabled }: { enabled: boolean }) {
  if (enabled) {
    return <div className="bg-green-500 rounded-md pl-2 pr-2 m-2">Enabled</div>;
  } else {
    return <div className="bg-red-500 rounded-md pl-2 pr-2 m-2">Disabled</div>;
  }
}
