import PanelEars from "@/theme/PanelEars";
import { FeVCToy } from "@bindings/FeVCToy";
import { useState } from "react";
import "./App.css";
import { useCoreEventContext } from "./context/CoreEvents";
import Config from "./features/Config";
import EmptyState from "./features/EmptyState";
import Sidebar from "./features/Sidebar";
import Toy from "./features/Toy";
import { toyKey, useToys } from "./hooks/useToys";
import { useUpdate } from "./hooks/useUpdate";
import { useVersion } from "./hooks/useVersion";
import { cn } from "./lib/utils";

type Selection = ToySelection | ConfigSelection | null;

type ToySelection = {
  type: "Toy";
  toyKey: string;
};

type ConfigSelection = {
  type: "Config";
};

const SIDEBAR_COLLAPSED_KEY = "vibecheck-sidebar-collapsed";

export default function App() {
  const [selection, setSelection] = useState<Selection>(null);
  // Sidebar collapsed to an icon bar; remembered between launches
  const [sidebarCollapsed, setSidebarCollapsed] = useState(
    () => localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === "true",
  );

  function toggleSidebar() {
    setSidebarCollapsed((collapsed) => {
      localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(!collapsed));
      return !collapsed;
    });
  }
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
      <EmptyState />
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
      <div
        className={cn(
          "grid min-h-0 flex-1 grid-rows-[minmax(0,1fr)] gap-3 [transition:grid-template-columns_250ms_ease]",
          sidebarCollapsed
            ? "grid-cols-[4.5rem,_minmax(0,1fr)]"
            : "grid-cols-[15rem,_minmax(0,1fr)]",
        )}
      >
        <Sidebar
          toys={toysList}
          selectedToyKey={selection?.type == "Toy" ? selection.toyKey : null}
          configSelected={selection?.type == "Config"}
          onSelectToy={setToy}
          onToggleConfig={setConfig}
          canUpdate={canUpdate}
          version={version}
          collapsed={sidebarCollapsed}
          onToggleCollapsed={toggleSidebar}
        />
        <div className="theme-panel theme-ears mt-3 min-h-0 min-w-0">
          <PanelEars />
          <div className="flex h-full p-4">{mainPanel}</div>
        </div>
      </div>
    </div>
  );
}
