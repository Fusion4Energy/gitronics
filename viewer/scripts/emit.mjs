// Copies the built single-file dashboard into the Rust crate's src/ as the
// committed artifact that `include_str!` embeds. Keeps the crate build Node-free.
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = resolve(here, "..", "dist", "index.html");
const dest = resolve(here, "..", "..", "src", "project_report.html");

mkdirSync(dirname(dest), { recursive: true });
copyFileSync(src, dest);
console.log(`emitted ${dest}`);
