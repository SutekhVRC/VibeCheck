import type { FeVCToy } from "@bindings/FeVCToy";
import type { ComponentType } from "react";

/*
 * What an app built on VibeCheck can add to its frontend. The VibeCheck app
 * adds nothing (none.ts). A build can point the `@extensions` import at its
 * own module with the VIBECHECK_EXTENSIONS environment variable (see
 * vite.config.ts); that module's default export is an AppExtensions.
 */

/** One entry listed under a device-list page's own row, e.g. one device */
export type ExtensionItem = {
  /** Unique within its page */
  id: string;
  label: string;
  /** Short state at the end of the row, e.g. "armed" */
  note?: string;
};

/** What an extension's page is given */
export type ExtensionPageProps = {
  /** The item open (an ExtensionItem's id), or null for the page itself */
  item: string | null;
  /** Opens the page itself (null) or one of its items, e.g. one just added */
  open: (item: string | null) => void;
  /**
   * Every toy the app lists: connected ones (with their real sub_id) and
   * saved ones that aren't connected, as in the sidebar
   */
  toys: FeVCToy[];
};

/** A page of its own, opened from the sidebar */
export type ExtensionPage = {
  /** Unique among pages */
  id: string;
  /**
   * Where its button goes: "nav" (the default), a square next to
   * Diagnostics, for tools; or "devices", a row in the device list below
   * the toys, for pages about devices.
   */
  placement?: "nav" | "devices";
  /** Button label and tooltip */
  label: string;
  Icon: ComponentType<{ className?: string }>;
  Page: ComponentType<ExtensionPageProps>;
  /**
   * For "devices" pages: entries listed under the page's row, each opening
   * the page with that item. A hook: the sidebar calls it on every render.
   */
  useItems?: () => ExtensionItem[];
};

/** A section in Settings, before Troubleshooting */
export type ExtensionSettingsSection = {
  /** Unique among sections */
  id: string;
  title: string;
  /** Shown beside the title, e.g. "Saves instantly" */
  hint?: string;
  Body: ComponentType;
};

export type AppExtensions = {
  pages?: ExtensionPage[];
  settingsSections?: ExtensionSettingsSection[];
};
