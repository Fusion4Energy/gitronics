const nf = new Intl.NumberFormat("en-US");

export function num(n: number): string {
  return nf.format(n);
}

/** Total ids covered by inclusive [start, end] runs. */
export function runsTotal(runs: [number, number][]): number {
  return runs.reduce((acc, [a, b]) => acc + (b - a + 1), 0);
}

/** A compact "12–18, 40" style rendering of id runs (first few only). */
export function runsLabel(runs: [number, number][], max = 3): string {
  const parts = runs
    .slice(0, max)
    .map(([a, b]) => (a === b ? `${a}` : `${a}–${b}`));
  if (runs.length > max) parts.push(`+${runs.length - max}`);
  return parts.join(", ");
}
