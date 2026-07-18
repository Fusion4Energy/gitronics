<script lang="ts">
  import type { ProjectReport } from "./types";
  import { loadReport } from "./data";
  import Overview from "./components/Overview.svelte";
  import Fillers from "./components/Fillers.svelte";
  import Envelopes from "./components/Envelopes.svelte";
  import ConfigDiff from "./components/ConfigDiff.svelte";

  let report = $state<ProjectReport | null>(null);
  let error = $state<string | null>(null);
  let tab = $state<"overview" | "fillers" | "envelopes" | "diff">("overview");
  let configName = $state("");

  $effect(() => {
    loadReport()
      .then((r) => {
        report = r;
        configName = r.configurations[0]?.name ?? "";
      })
      .catch((e) => (error = String(e)));
  });

  const config = $derived(
    report?.configurations.find((c) => c.name === configName) ?? report?.configurations[0],
  );

  let theme = $state(document.documentElement.getAttribute("data-theme") ?? "light");
  function toggleTheme() {
    theme = theme === "dark" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", theme);
    try {
      localStorage.setItem("gitronics-theme", theme);
    } catch (e) {}
  }
</script>

<div class="wrap">
  {#if error}
    <div class="panel"><h2>Could not load report</h2><p class="muted">{error}</p></div>
  {:else if !report || !config}
    <p class="muted">Loading…</p>
  {:else}
    <header class="app">
      <h1>Gitronics — Project Report</h1>
      <span class="sub">{report.project_dir} · {report.gitronics_version} · {report.commit_hash} · {report.date_time}</span>
      <span class="spacer"></span>
      {#if report.configurations.length > 1}
        <label class="field">
          Configuration
          <select bind:value={configName}>
            {#each report.configurations as c}<option value={c.name}>{c.name}</option>{/each}
          </select>
        </label>
      {/if}
      <button class="ghost" onclick={toggleTheme}>{theme === "dark" ? "☀ Light" : "☾ Dark"}</button>
    </header>

    <nav class="tabs">
      <button class:active={tab === "overview"} onclick={() => (tab = "overview")}>Overview</button>
      <button class:active={tab === "fillers"} onclick={() => (tab = "fillers")}>Fillers</button>
      <button class:active={tab === "envelopes"} onclick={() => (tab = "envelopes")}>Envelopes</button>
      {#if report.configurations.length > 1}
        <button class:active={tab === "diff"} onclick={() => (tab = "diff")}>Config diff</button>
      {/if}
    </nav>

    {#if tab === "overview"}
      <Overview {report} {config} />
    {:else if tab === "fillers"}
      <Fillers {report} {config} />
    {:else if tab === "envelopes"}
      <Envelopes {report} {config} />
    {:else}
      <ConfigDiff {report} />
    {/if}
  {/if}
</div>
