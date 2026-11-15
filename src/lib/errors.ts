/**
 * Errors from backend commands. VibeCheck's own commands fail with
 * `{ code, message }` (see src-tauri/src/vcore/errors.rs): `message` is
 * written for the user, `code` names the problem for cases the frontend
 * handles itself. Tauri plugins and the browser fail with strings or Errors.
 */

export type CommandError = { code: string; message: string };

function isCommandError(e: unknown): e is CommandError {
  return (
    typeof e === "object" &&
    e !== null &&
    typeof (e as CommandError).code === "string" &&
    typeof (e as CommandError).message === "string"
  );
}

/** Any error as text to show the user */
export function errorText(e: unknown): string {
  if (isCommandError(e)) return e.message;
  if (e instanceof Error) return e.message;
  return typeof e === "string" ? e : JSON.stringify(e);
}

/** A command error's code, e.g. "ToyOnline", or null for other errors */
export function errorCode(e: unknown): string | null {
  return isCommandError(e) ? e.code : null;
}
