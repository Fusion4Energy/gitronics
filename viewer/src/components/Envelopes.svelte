<script lang="ts">
  import type { ProjectReport, ConfigEntry } from "../types";
  import { num } from "../lib/format";

  let { report, config }: { report: ProjectReport; config: ConfigEntry } = $props();

  const envMeta = $derived(
    new Map(report.envelope_inventory.map((e) => [e.envelope_name, e])),
  );

  let only = $state<"all" | "filled" | "unfilled">("all");
  let search = $state("");

  const rows = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return config.envelopes.filter((e) => {
      if (only === "filled" && !e.filler_name) return false;
      if (only === "unfilled" && e.filler_name) return false;
      if (q && !e.envelope_name.toLowerCase().includes(q)) return false;
      return true;
    });
  });

  const usedFillers = $derived(
    new Set(config.envelopes.map((e) => e.filler_name).filter(Boolean) as string[]),
  );
  const unusedFillers = $derived(
    report.filler_library.filter((f) => !usedFillers.has(f.name)),
  );
</script>

<div class="cards">
  <div class="stat"><div class="n">{num(config.envelopes.length)}</div><div class="l">Envelopes (config)</div></div>
  <div class="stat"><div class="n">{num(config.stats.filled)}</div><div class="l">Filled</div></div>
  <div class="stat"><div class="n" style="color:{config.stats.unfilled ? 'var(--warn)' : 'inherit'}">{num(config.stats.unfilled)}</div><div class="l">Unfilled</div></div>
  <div class="stat"><div class="n" style="color:{unusedFillers.length ? 'var(--warn)' : 'inherit'}">{num(unusedFillers.length)}</div><div class="l">Unused fillers</div></div>
</div>

<div class="panel">
  <h2>Envelope coverage — {config.name}</h2>
  <div class="controls">
    <input type="search" placeholder="Search envelope…" bind:value={search} />
    <label class="field">
      Show
      <select bind:value={only}>
        <option value="all">all</option>
        <option value="filled">filled</option>
        <option value="unfilled">unfilled</option>
      </select>
    </label>
    <span class="muted">{num(rows.length)} shown</span>
  </div>
  <div class="tablewrap">
    <table>
      <thead>
        <tr>
          <th>Envelope</th>
          <th>Description</th>
          {#each report.metadata_keys.envelope as k}<th>{k}</th>{/each}
          <th>Filler</th>
          <th>U</th>
          <th>Transform</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as e (e.envelope_name)}
          {@const m = envMeta.get(e.envelope_name)}
          <tr>
            <td>{e.envelope_name}</td>
            <td class="muted">{m?.description ?? ""}</td>
            {#each report.metadata_keys.envelope as k}
              <td>{m?.metadata?.[k] ?? ""}</td>
            {/each}
            <td>{e.filler_name ?? "—"}</td>
            <td class="muted">{e.universe_id ?? ""}</td>
            <td class="muted">{e.transform ?? ""}</td>
            <td>
              {#if e.filler_name}
                <span class="tag ok">filled</span>
              {:else}
                <span class="tag warn">unfilled</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if unusedFillers.length}
    <p class="muted" style="margin-top:.8rem">
      <strong>Unused fillers</strong> (in library, not placed by {config.name}):
      {unusedFillers.map((f) => f.name).join(", ")}
    </p>
  {/if}
</div>
