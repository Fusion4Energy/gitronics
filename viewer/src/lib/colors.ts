// A brand-neutral categorical palette (accessible in light and dark). Colors are
// assigned to category values deterministically so the same value keeps its color
// across views and reloads.
const PALETTE = [
  "#4c78a8",
  "#f58518",
  "#54a24b",
  "#e45756",
  "#72b7b2",
  "#b279a2",
  "#ff9da6",
  "#9d755d",
  "#bab0ac",
  "#6a9f58",
  "#d4a017",
  "#8c6bb1",
];

const UNSET = "#9aa4b0";

const assigned = new Map<string, string>();

/** Stable color for a category value; `null`/empty gets a neutral grey. */
export function colorFor(value: string | null | undefined): string {
  if (value == null || value === "") return UNSET;
  const hit = assigned.get(value);
  if (hit) return hit;
  const color = PALETTE[assigned.size % PALETTE.length];
  assigned.set(value, color);
  return color;
}

export const UNSET_COLOR = UNSET;
