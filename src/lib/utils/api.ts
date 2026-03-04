/**
 * Extracts a human-readable error message from Tauri command errors.
 * Tauri errors are NOT JavaScript Error objects — use this, not instanceof Error.
 */
export function getErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return JSON.stringify(error);
}
