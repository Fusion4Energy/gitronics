import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { viteSingleFile } from "vite-plugin-singlefile";
import { resolve } from "node:path";

// The viewer compiles to ONE self-contained HTML file with all JS + CSS inlined
// (no network requests), which the Rust crate embeds via `include_str!` and fills
// with a project manifest at run time. The build output is written directly into
// `../src/` so `cargo build` picks it up; the committed artifact keeps the crate
// build Node-free.
export default defineConfig({
  plugins: [svelte(), viteSingleFile()],
  build: {
    target: "es2020",
    outDir: resolve(__dirname, "dist"),
    emptyOutDir: true,
    // Deterministic single-file output; the plugin inlines assets regardless.
    cssCodeSplit: false,
    assetsInlineLimit: 100_000_000,
    rollupOptions: {
      input: resolve(__dirname, "index.html"),
    },
  },
});
