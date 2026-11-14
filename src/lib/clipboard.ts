import { invoke } from "@tauri-apps/api/core";

/**
 * Copies text to the system clipboard through Tauri's clipboard plugin
 * (registered in main.rs, allowed in capabilities/migrated.json).
 *
 * Not navigator.clipboard: the webview can refuse it (WebKitGTK on Linux
 * does), and it fails with a DOMException that says nothing useful.
 */
export async function copyText(text: string): Promise<void> {
  await invoke("plugin:clipboard-manager|write_text", { text });
}

/** An error from a failed copy, as readable text */
export function errorText(e: unknown): string {
  if (e instanceof Error) return e.message;
  return typeof e === "string" ? e : JSON.stringify(e);
}
