import ThemeIcon from "@/theme/ThemeIcon";
import { useTheme } from "@/theme/ThemeProvider";
import { FeVCToy } from "@bindings/FeVCToy";
import { useState } from "react";
import "./App.css";
import cryingAnimeGirl from "./assets/menhera_chan.gif";
import { useCoreEventContext } from "./context/CoreEvents";
import Config from "./features/Config";
import Sidebar from "./features/Sidebar";
import Toy from "./features/Toy";
import { toyKey, useToys } from "./hooks/useToys";
import { useUpdate } from "./hooks/useUpdate";
import { useVersion } from "./hooks/useVersion";

type Selection = ToySelection | ConfigSelection | null;

type ToySelection = {
  type: "Toy";
  toyKey: string;
};

type ConfigSelection = {
  type: "Config";
};

export default function App() {
  const themeText = useTheme().theme.text;
  const [selection, setSelection] = useState<Selection>(null);
  const { toys, hasOnlineToys } = useToys();
  const toy =
    selection?.type == "Toy" && selection.toyKey in toys
      ? toys[selection.toyKey]
      : null;
  const toysList = Object.values(toys);
  const { isEnabled, toggleIsEnabled, config, refreshConfig } =
    useCoreEventContext();
  const { canUpdate } = useUpdate();
  const { version } = useVersion();

  async function disableOnPortChange() {
    if (isEnabled) {
      await toggleIsEnabled();
    }
  }

  if (
    selection?.type == "Toy" &&
    !toysList.some((t) => toyKey(t) == selection.toyKey)
  )
    setSelection(null); // selection is no longer valid

  const mainPanel =
    selection?.type == "Toy" && toy != null ? (
      <Toy toy={toy} />
    ) : selection?.type == "Config" && config != null ? (
      <Config
        config={config}
        refreshConfig={refreshConfig}
        canUpdate={canUpdate}
        version={version}
        disableOnPortChange={disableOnPortChange}
      />
    ) : !hasOnlineToys ? (
      <div className="flex w-full flex-col items-center justify-center gap-3">
        <img
          src={cryingAnimeGirl}
          className="animate-float rounded-3xl drop-shadow-theme-glow"
        />
        <div className="flex items-center gap-2 text-lg text-theme-primary">
          <ThemeIcon className="h-5 w-5" />
          {themeText.noToysTitle}
          <ThemeIcon className="h-5 w-5 -scale-x-100" />
        </div>
        <div className="text-sm text-theme-muted">{themeText.noToysHint}</div>
      </div>
    ) : null;

  function setToy(toy: FeVCToy) {
    const newKey = toyKey(toy);
    if (selection?.type == "Toy" && selection.toyKey == newKey) return;
    setSelection({
      type: "Toy",
      toyKey: newKey,
    });
  }

  function setConfig() {
    if (selection?.type == "Config") setSelection(null);
    else setSelection({ type: "Config" });
  }

  return (
    <div
      className="flex h-screen w-full flex-col p-4"
      onContextMenu={(e) => e.preventDefault()}
    >
      {/* The grid fills the window height; its single row is capped to that
          height so tall content scrolls inside the panels instead of
          stretching past the bottom of the window */}
      <div className="grid min-h-0 flex-1 grid-cols-[12rem,_minmax(0,1fr)] grid-rows-[minmax(0,1fr)] gap-3">
        <Sidebar
          toys={toysList}
          selectedToyKey={selection?.type == "Toy" ? selection.toyKey : null}
          configSelected={selection?.type == "Config"}
          onSelectToy={setToy}
          onToggleConfig={setConfig}
          canUpdate={canUpdate}
          version={version}
        />
        <div className="theme-panel theme-ears mt-3 min-h-0 min-w-0">
          <div className="flex h-full p-4">{mainPanel}</div>
        </div>
      </div>
    </div>
  );
}
