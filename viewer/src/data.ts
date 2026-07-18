import type { ProjectReport } from "./types";

/**
 * Reads the project manifest embedded by Rust in `#report-data`. The Rust
 * `inspect` command replaces the placeholder token with the escaped manifest
 * JSON, so a real payload always begins with `{`. During `vite dev` (and if the
 * token is left unreplaced) the payload is not JSON, and we fall back to the
 * bundled mock manifest. NOTE: this deliberately avoids embedding the literal
 * placeholder token in the JS bundle so Rust's replace targets exactly one site.
 */
export async function loadReport(): Promise<ProjectReport> {
  const el = document.getElementById("report-data");
  const raw = el?.textContent?.trim() ?? "";
  if (raw.startsWith("{")) {
    return JSON.parse(raw) as ProjectReport;
  }
  if (import.meta.env.DEV) {
    const mock = await import("./mock.json");
    return mock.default as unknown as ProjectReport;
  }
  throw new Error("No project manifest found in #report-data.");
}

/** Reads a metadata field, preferring the recognized first-class `description`. */
export function metaValue(
  meta: Record<string, unknown> | undefined,
  key: string,
): string | undefined {
  if (!meta) return undefined;
  const v = meta[key];
  if (v == null) return undefined;
  return typeof v === "object" ? JSON.stringify(v) : String(v);
}
