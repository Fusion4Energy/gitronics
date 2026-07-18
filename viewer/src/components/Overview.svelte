<script lang="ts">
  import type { ProjectReport, ConfigEntry } from "../types";
  import { colorFor } from "../lib/colors";
  import { num } from "../lib/format";
  import Treemap from "./Treemap.svelte";

  let { report, config }: { report: ProjectReport; config: ConfigEntry } = $props();

  const fillerByName = $derived(
    new Map(report.filler_library.map((f) => [f.name, f])),
  );
  const envByName = $derived(
    new Map(report.envelope_inventory.map((e) => [e.envelope_name, e])),
  );

  // Group-by options: any envelope or filler metadata key present in the project.
  const groupOptions = $derived([
    { id: "envelope:__system__", label: "— filler —", disabled: true },
    ...report.metadata_keys.filler.map((k) => ({ id: `filler:${k}`, label: `filler · ${k}`, disabled: false })),
    { id: "envelope:__system2__", label: "— envelope —", disabled: true },
    ...report.metadata_keys.envelope.map((k) => ({ id: `envelope:${k}`, label: `envelope · ${k}`, disabled: false })),
  ]);

  let groupBy = $state("");
  $effect(() => {
    if (groupBy) return;
    const fk = report.metadata_keys.filler[0];
    const ek = report.metadata_keys.envelope[0];
    groupBy = fk ? `filler:${fk}` : ek ? `envelope:${ek}` : "";
  });
  let metric = $state<"cells" | "surfaces" | "envelopes">("cells");

  function groupValue(envName: string, fillerName: string): string {
    if (!groupBy) return "all";
    const [scope, key] = groupBy.split(":");
    const meta =
      scope === "filler"
        ? fillerByName.get(fillerName)?.metadata
        : envByName.get(envName)?.metadata;
    const v = meta?.[key];
    return v == null ? "(unset)" : String(v);
  }

  const leaves = $derived.by(() => {
    return config.envelopes
      .filter((e) => e.filler_name)
      .map((e) => {
        const f = fillerByName.get(e.filler_name!);
        const value =
          metric === "cells"
            ? (f?.cell_count ?? 0)
            : metric === "surfaces"
              ? (f?.surface_count ?? 0)
              : 1;
        const group = groupValue(e.envelope_name, e.filler_name!);
        return {
          name: e.envelope_name,
          value: Math.max(value, 0),
          group,
          color: colorFor(group),
          title: `${e.envelope_name} ← ${e.filler_name} (${group})`,
        };
      })
      .filter((l) => l.value > 0);
  });

  const groupsLegend = $derived.by(() => {
    const totals = new Map<string, number>();
    for (const l of leaves) totals.set(l.group, (totals.get(l.group) ?? 0) + l.value);
    return [...totals.entries()].sort((a, b) => b[1] - a[1]);
  });

  const totalCells = $derived(
    report.filler_library.reduce((a, f) => a + f.cell_count, 0),
  );
</script>

<div class="cards">
  <div class="stat"><div class="n">{num(config.stats.filled)}</div><div class="l">Filled envelopes</div></div>
  <div class="stat"><div class="n">{num(config.stats.unfilled)}</div><div class="l">Unfilled</div></div>
  <div class="stat"><div class="n">{num(config.stats.distinct_fillers)}</div><div class="l">Fillers used</div></div>
  <div class="stat"><div class="n">{num(report.filler_library.length)}</div><div class="l">Fillers in library</div></div>
  <div class="stat"><div class="n">{num(config.stats.unused_fillers)}</div><div class="l">Unused fillers</div></div>
</div>

<div class="panel">
  <h2>Composition — {config.name}</h2>
  <div class="controls">
    <label class="field">
      Group by
      <select bind:value={groupBy}>
        {#each groupOptions as o}
          <option value={o.id} disabled={o.disabled}>{o.label}</option>
        {/each}
      </select>
    </label>
    <label class="field">
      Size by
      <select bind:value={metric}>
        <option value="cells">cell count</option>
        <option value="surfaces">surface count</option>
        <option value="envelopes">envelope count</option>
      </select>
    </label>
    <span class="muted">{num(leaves.length)} placements · {num(totalCells)} library cells</span>
  </div>

  {#if leaves.length}
    <Treemap {leaves} />
    <div class="legend">
      {#each groupsLegend as [g, v]}
        <span class="item"><span class="swatch" style="background:{colorFor(g)}"></span>{g} · {num(v)}</span>
      {/each}
    </div>
  {:else}
    <p class="muted">No filled envelopes in this configuration.</p>
  {/if}
</div>
