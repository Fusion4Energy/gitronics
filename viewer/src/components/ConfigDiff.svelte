<script lang="ts">
  import type { ProjectReport } from "../types";
  import { num } from "../lib/format";

  let { report }: { report: ProjectReport } = $props();

  const names = $derived(report.configurations.map((c) => c.name));
  let aName = $state("");
  let bName = $state("");
  $effect(() => {
    if (!aName) aName = report.configurations[0]?.name ?? "";
    if (!bName)
      bName = report.configurations[1]?.name ?? report.configurations[0]?.name ?? "";
  });

  function mapOf(name: string): Map<string, string> {
    const c = report.configurations.find((c) => c.name === name);
    const m = new Map<string, string>();
    for (const e of c?.envelopes ?? []) {
      const val = `${e.filler_name ?? "∅"}${e.transform ? " " + e.transform : ""}`;
      m.set(e.envelope_name, val);
    }
    return m;
  }

  const diff = $derived.by(() => {
    const a = mapOf(aName);
    const b = mapOf(bName);
    const keys = new Set([...a.keys(), ...b.keys()]);
    const rows: { env: string; a: string; b: string }[] = [];
    for (const k of keys) {
      const av = a.get(k) ?? "—";
      const bv = b.get(k) ?? "—";
      if (av !== bv) rows.push({ env: k, a: av, b: bv });
    }
    rows.sort((x, y) => x.env.localeCompare(y.env));
    return rows;
  });
</script>

<div class="panel">
  <h2>Configuration diff</h2>
  <div class="controls">
    <label class="field">
      A
      <select bind:value={aName}>{#each names as n}<option value={n}>{n}</option>{/each}</select>
    </label>
    <label class="field">
      B
      <select bind:value={bName}>{#each names as n}<option value={n}>{n}</option>{/each}</select>
    </label>
    <span class="muted">{num(diff.length)} envelopes differ</span>
  </div>

  {#if aName === bName}
    <p class="muted">Pick two different configurations to compare.</p>
  {:else if diff.length === 0}
    <p class="muted">Identical envelope assignments.</p>
  {:else}
    <div class="tablewrap">
      <table>
        <thead>
          <tr><th>Envelope</th><th>{aName}</th><th>{bName}</th></tr>
        </thead>
        <tbody>
          {#each diff as d (d.env)}
            <tr>
              <td>{d.env}</td>
              <td><span class="tag danger">{d.a}</span></td>
              <td><span class="tag ok">{d.b}</span></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
