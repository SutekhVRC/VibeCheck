import { invoke } from "@tauri-apps/api";
import { VERSION } from "../../data/constants";
import { useEffect, useState } from "react";

export default function () {
  const [version, setVersion] = useState("");

  useEffect(() => {
    async function getVersion() {
      await invoke<string>(VERSION).then((r) => {
        setVersion(r.replaceAll("-", " "));
      });
    }
    getVersion();
  }, []);

  return <div>{version}</div>;
}
