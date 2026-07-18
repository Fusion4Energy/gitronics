<script lang="ts">
  import type { ProjectReport, ConfigEntry } from "../types";
  import { colorFor } from "../lib/colors";
  import { num, runsLabel } from "../lib/format";

  let { report, config }: { report: ProjectReport; config: ConfigEntry } = $props();

  // Envelope reuse of each filler *within the selected configuration*.
  const reuse = $derived.by(() => {
    const m = new Map<string, number>();
    for (const e of config.envelopes) {
      if (e.filler_name) m.set(e.filler_name, (m.get(e.filler_name) ?? 0) + 1);
    }
    return m;
  });
  const maxReuse = $derived(Math.max(1, ...[...reuse.values()]));

  let search = $state("");
  let facetKey = $state("");
  let facetVal = $state("");
  let sortKey = $state<"name" | "cells" | "surfaces" | "reuse">("reuse");
  let sortDir = $state<1 | -1>(-1);

  const facetValues = $derived.by(() => {
    if (!facetKey) return [];
    const s = new Set<string>();
    for (const f of report.filler_library) {
      const v = f.metadata?.[facetKey];
      s.add(v == null ? "(unset)" : String(v));
    }
    return [...s].sort();
  });

  function sortBy(k: typeof sortKey) {
    if (sortKey === k) sortDir = sortDir === 1 ? -1 : 1;
    else {
      sortKey = k;
      sortDir = k === "name" ? 1 : -1;
    }
  }

  const rows = $derived.by(() => {
    const q = search.trim().toLowerCase();
    let list = report.filler_library.filter((f) => {
      if (q && !(f.name.toLowerCase().includes(q) || (f.description ?? "").toLowerCase().includes(q)))
        return false;
      if (facetKey && facetVal) {
        const v = f.metadata?.[facetKey];
        if ((v == null ? "(unset)" : String(v)) !== facetVal) return false;
      }
      return true;
    });
    list = [...list].sort((a, b) => {
      let d = 0;
      if (sortKey === "name") d = a.name.localeCompare(b.name);
      else if (sortKey === "cells") d = a.cell_count - b.cell_count;
      else if (sortKey === "surfaces") d = a.surface_count - b.surface_count;
      else d = (reuse.get(a.name) ?? 0) - (reuse.get(b.name) ?? 0);
      return d * sortDir;
    });
    return list;
  });
</script>

<div class="panel">
  <h2>Filler library — {num(report.filler_library.length)} universes</h2>
  <div class="controls">
    <input type="search" placeholder="Search name / description…" bind:value={search} />
    <label class="field">
      Facet
      <select bind:value={facetKey} onchange={() => (facetVal = "")}>
        <option value="">— none —</option>
        {#each report.metadata_keys.filler as k}
          <option value={k}>{k}</option>
        {/each}
      </select>
    </label>
    {#if facetKey}
      <select bind:value={facetVal}>
        <option value="">all</option>
        {#each facetValues as v}
          <option value={v}>{v}</option>
        {/each}
      </select>
    {/if}
    <span class="muted">{num(rows.length)} shown · reuse within {config.name}</span>
  </div>

  <div class="tablewrap">
    <table>
      <thead>
        <tr>
          <th onclick={() => sortBy("name")}>Filler</th>
          <th>U</th>
          <th>Description</th>
          {#each report.metadata_keys.filler as k}<th>{k}</th>{/each}
          <th class="num" onclick={() => sortBy("cells")}>Cells</th>
          <th class="num" onclick={() => sortBy("surfaces")}>Surfaces</th>
          <th onclick={() => sortBy("reuse")}>Reuse</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as f (f.name)}
          {@const r = reuse.get(f.name) ?? 0}
          <tr>
            <td>{f.name}</td>
            <td class="muted">{f.universe_id}</td>
            <td class="muted">{f.description ?? ""}</td>
            {#each report.metadata_keys.filler as k}
              {@const v = f.metadata?.[k]}
              <td>
                {#if v != null}
                  <span class="tag" style="border-color:{colorFor(String(v))}">{v}</span>
                {/if}
              </td>
            {/each}
            <td class="num">{num(f.cell_count)}</td>
            <td class="num">{num(f.surface_count)}</td>
            <td>
              <div style="display:flex; align-items:center; gap:.4rem">
                <div class="bar" style="width:{(r / maxReuse) * 120}px; opacity:{r ? 1 : 0.15}"></div>
                <span class="muted">{r}</span>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  {#if rows.length}
    <p class="muted" style="margin-top:.6rem">
      Cell ids e.g. {rows[0].name}: {runsLabel(rows[0].cell_id_runs)}
    </p>
  {/if}
</div>
