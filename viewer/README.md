# Gitronics viewer

The interactive dashboard rendered by `gitronics inspect`. It is a
[Svelte](https://svelte.dev) app built with [Vite](https://vitejs.dev) and
[`vite-plugin-singlefile`](https://github.com/richardtallent/vite-plugin-singlefile) into **one
self-contained HTML file** with all JS and CSS inlined and no external requests.

## Build-time only

Node is only needed to *rebuild the viewer*. It is **not** a runtime dependency and is **not** required
to build the Rust crate or the Python wheel: the built artifact is committed at
[`../src/project_report.html`](../src/project_report.html) and embedded into the binary with
`include_str!`. At run time, `gitronics inspect` injects the project manifest into the file's
`#report-data` placeholder.

## Commands

```bash
npm install        # install dev dependencies (first time)
npm run build      # build → dist/index.html, then copy to ../src/project_report.html
npm run dev        # local dev server with a bundled mock manifest (src/mock.json)
npm run check      # svelte-check type checker
```

**After changing anything under `viewer/`, run `npm run build` and commit the updated
`src/project_report.html`.** CI rebuilds the viewer and fails if the committed artifact is out of date
(see the `viewer` job in `.github/workflows/ci.yml`).

## Layout

| Path | Purpose |
|---|---|
| `src/App.svelte` | Shell: header, tabs, configuration selector, theme toggle. |
| `src/components/` | The four views: Overview (treemap), Fillers, Envelopes, ConfigDiff. |
| `src/types.ts` | TypeScript mirror of the Rust `ProjectReport` manifest (`src/project_report.rs`). |
| `src/data.ts` | Reads the injected manifest (or the dev mock). |
| `src/mock.json` | Sample manifest used by `npm run dev`. |
| `scripts/emit.mjs` | Copies the built file to `../src/project_report.html`. |
