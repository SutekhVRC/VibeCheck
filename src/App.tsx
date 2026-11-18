import PanelEars from "@/theme/PanelEars";
import { FeVCToy } from "@bindings/FeVCToy";
import { useState } from "react";
import "./App.css";
import { useCoreEventContext } from "./context/CoreEvents";
import { appExtensions } from "./extensions";
import Config from "./features/Config";
import Diagnostics from "./features/Diagnostics";
import EmptyState from "./features/EmptyState";
import Sidebar from "./features/Sidebar";
import Toy from "./features/Toy";
import { toyKey, useToys } from "./hooks/useToys";
import { useUpdate } from "./hooks/useUpdate";
import { useVersion } from "./hooks/useVersion";
import { cn } from "./lib/utils";

type Selection =
  | ToySelection
  | ConfigSelection
  | DiagnosticsSelection
  | ExtensionSelection
  | null;

type ToySelection = {
  type: "Toy";
  toyKey: string;
};

type ConfigSelection = {
  type: "Config";
};

type DiagnosticsSelection = {
  type: "Diagnostics";
};

/** A page an extension adds (src/extensions) */
type ExtensionSelection = {
  type: "Extension";
  id: string;
  /** One of the page's items (ExtensionItem), or null for the page itself */
  item: string | null;
};

const extensionPages = appExtensions.pages ?? [];

export default function App() {
  const [selection, setSelection] = useState<Selection>(null);
  const { toys, hasOnlineToys, removeOfflineToy } = useToys();
  const toy =
    selection?.type == "Toy" && selection.toyKey in toys
      ? toys[selection.toyKey]
      : null;
  const toysList = Object.values(toys);
  const { isEnabled, toggleIsEnabled, config, refreshConfig, saveConfig } =
    useCoreEventContext();
  // Sidebar collapsed to an icon bar; saved in Config.json
  const sidebarCollapsed = config?.sidebar_collapsed ?? false;
  function toggleSidebar() {
    saveConfig({ sidebar_collapsed: !sidebarCollapsed });
  }
  const { canUpdate, updateStatus } = useUpdate();
  const { version, appName } = useVersion();

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

  const extensionPage =
    selection?.type == "Extension"
      ? extensionPages.find((p) => p.id == selection.id)
      : undefined;

  const mainPanel =
    extensionPage != null && selection?.type == "Extension" ? (
      <extensionPage.Page
        item={selection.item}
        open={(item) =>
          setSelection({ type: "Extension", id: selection.id, item })
        }
      />
    ) : selection?.type == "Toy" && toy != null ? (
      <Toy toy={toy} onRemove={removeOfflineToy} />
    ) : selection?.type == "Config" && config != null ? (
      <Config
        config={config}
        refreshConfig={refreshConfig}
        canUpdate={canUpdate}
        updateStatus={updateStatus}
        version={version}
        appName={appName}
        disableOnPortChange={disableOnPortChange}
      />
    ) : selection?.type == "Diagnostics" ? (
      <Diagnostics toys={toysList} />
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

  function setExtensionPage(id: string, item: string | null = null) {
    if (
      selection?.type == "Extension" &&
      selection.id == id &&
      selection.item == item
    )
      setSelection(null);
    else setSelection({ type: "Extension", id, item });
  }

  function setDiagnostics() {
    if (selection?.type == "Diagnostics") setSelection(null);
    else setSelection({ type: "Diagnostics" });
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
          diagnosticsSelected={selection?.type == "Diagnostics"}
          onToggleDiagnostics={setDiagnostics}
          extensionPages={extensionPages}
          selectedExtension={selection?.type == "Extension" ? selection : null}
          onToggleExtensionPage={setExtensionPage}
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
