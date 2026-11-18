import type { ComponentType } from "react";

/*
 * What an app built on VibeCheck can add to its frontend. The VibeCheck app
 * adds nothing (none.ts). A build can point the `@extensions` import at its
 * own module with the VIBECHECK_EXTENSIONS environment variable (see
 * vite.config.ts); that module's default export is an AppExtensions.
 */

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
  Page: ComponentType;
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
