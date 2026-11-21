import type { ValueDecoration } from "@/components/ui/FreeTextOptions";
import type { FeFeatureCheck } from "@bindings/FeFeatureCheck";
import type { FeVCToy } from "@bindings/FeVCToy";
import type { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import type { PenetrationSystemType } from "@bindings/PenetrationSystemType";
import type { ComponentType } from "react";
export type { ValueDecoration };

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
  /** The note's colour; muted if unset */
  noteTone?: NoteTone;
};

/** A row note's colour */
export type NoteTone = "success" | "warning" | "danger";

/** A device-list page's own row's state, e.g. "connected" */
export type ExtensionPageStatus = {
  note?: string;
  noteTone?: NoteTone;
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
   * Where its button goes: "nav" (the default), a tile in the tools row
   * above Settings, for tools; or "devices", a row in the device list below
   * the toys, for pages about devices.
   */
  placement?: "nav" | "devices";
  /** Button label and tooltip */
  label: string;
  /**
   * A shorter label where space is tight or the rest is said nearby: its
   * tile in the tools row ("nav" pages), e.g. "Stats" for "Session
   * stats"; or its row under its group title ("devices" pages with a
   * group), e.g. "Bridge" under "Smart lights"
   */
  shortLabel?: string;
  Icon: ComponentType<{ className?: string }>;
  Page: ComponentType<ExtensionPageProps>;
  /**
   * For "devices" pages: entries listed under the page's row, each opening
   * the page with that item. A hook: the sidebar calls it on every render.
   */
  useItems?: () => ExtensionItem[];
  /** For "devices" pages: a hook giving the state shown on the page's row */
  useStatus?: () => ExtensionPageStatus;
  /**
   * For "devices" pages: a title above its card, like the toys' groups
   * (e.g. "Smart lights"), with the number of items beside it
   */
  group?: string;
  /** For "devices" pages: each item's icon (default: the page's Icon) */
  ItemIcon?: ComponentType<{ className?: string }>;
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

/**
 * What a slot in a toy feature's settings is given. The current avatar is
 * in useAvatarCatalog (src/context/AvatarCatalog.tsx).
 */
export type FeatureSlotProps = {
  feature: FeVCToyFeature;
  /** What the feature's settings miss on the current avatar, and which
   * zones each input filter tag matches; null while the avatar isn't known */
  check: FeFeatureCheck | null;
  /** Sets the Processor's system, and saves */
  setSystem: (system: PenetrationSystemType) => void;
  /** Sets the input filter's tags, and saves */
  setInputFilter: (tags: string[]) => void;
};

/** Extra content under rows of a toy feature's settings */
export type FeatureFormSlots = {
  /** Under the Processor row */
  Processor?: ComponentType<FeatureSlotProps>;
  /** Under the Input Filter row */
  InputFilter?: ComponentType<FeatureSlotProps>;
  /** Extra content after an input filter tag's text, and its tooltip */
  inputFilterTag?: (
    tag: string,
    props: FeatureSlotProps,
  ) => ValueDecoration | null | undefined;
};

export type AppExtensions = {
  pages?: ExtensionPage[];
  settingsSections?: ExtensionSettingsSection[];
  /** More lines in Settings > About, under the name and version */
  About?: ComponentType;
  featureForm?: FeatureFormSlots;
};
