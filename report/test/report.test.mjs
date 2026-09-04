// Behavioural tests for the build-report viewer.
//
// The failure these exist to catch is the silent one: report.js throws while
// booting, the Rust tests still pass because the HTML shell is well formed, and
// the user opens a blank page. Every test therefore asserts on a rendered DOM,
// and `boots_without_errors` asserts on the error channel directly.

import assert from "node:assert/strict";
import { test } from "node:test";
import { boot, sampleManifest, textOf } from "./harness.mjs";

const TABS = ["Overview", "Explorer", "Coverage Map", "ID Map", "Fillers", "Data Cards", "Diff"];

/** Clicks a tab by its visible label and returns its panel element. */
function openTab(document, label) {
  const button = [...document.querySelectorAll(".tab")].find((b) =>
    b.textContent.includes(label),
  );
  assert.ok(button, `no tab labelled ${label}`);
  button.dispatchEvent(new document.defaultView.MouseEvent("click", { bubbles: true }));
  return document.querySelector(".panel.active");
}

test("boots without errors", () => {
  const { document, errors } = boot(sampleManifest());

  assert.deepEqual(errors, [], "the viewer raised errors while booting");
  assert.ok(document.querySelector("header.topbar"), "no header rendered");
  assert.ok(
    !document.body.textContent.startsWith("Failed to parse report data"),
    "the embedded manifest did not parse",
  );
});

test("header shows the report's provenance", () => {
  const manifest = sampleManifest();
  const { document } = boot(manifest);
  const header = document.querySelector("header.topbar").textContent;

  assert.match(header, new RegExp(`v${manifest.gitronics_version}`));
  assert.ok(header.includes(manifest.config_path), "config path missing");
  assert.ok(header.includes(manifest.commit_hash), "commit hash missing");
});

test("renders every tab, with counts from the manifest", () => {
  const manifest = sampleManifest();
  const { document } = boot(manifest);

  const labels = textOf(document, ".tab").map((t) => t.replace(/\d+$/, "").trim());
  assert.deepEqual(labels, TABS);

  const pill = (label) =>
    Number(
      [...document.querySelectorAll(".tab")]
        .find((b) => b.textContent.includes(label))
        .querySelector(".pill").textContent,
    );
  assert.equal(pill("Explorer"), manifest.envelope_entries.length);
  assert.equal(pill("Fillers"), manifest.filler_entries.length);
});

test("every tab renders without throwing", () => {
  const { document, errors } = boot(sampleManifest());

  for (const label of TABS) {
    const panel = openTab(document, label);
    assert.ok(panel, `${label} rendered no active panel`);
    assert.ok(panel.textContent.trim().length > 0, `${label} rendered an empty panel`);
  }
  assert.deepEqual(errors, [], "rendering a tab raised errors");
});

test("overview reports coverage over the envelopes", () => {
  const manifest = sampleManifest();
  const { document } = boot(manifest);
  const text = openTab(document, "Overview").textContent;

  const filled = manifest.envelope_entries.filter((e) => e.filler_name).length;
  const expected = Math.round((filled / manifest.envelope_entries.length) * 100);
  assert.ok(text.includes(`${expected}%`), `coverage ${expected}% not shown`);
  assert.ok(text.includes(String(manifest.total_cells)), "total cells not shown");
});

test("explorer groups envelopes by a discovered metadata key", () => {
  const manifest = sampleManifest();
  const { document } = boot(manifest);
  const panel = openTab(document, "Explorer");

  const select = panel.querySelector("select");
  assert.ok(select, "no group-by control");

  // The keys offered are discovered from the data, not hard-coded.
  const options = [...select.options].map((o) => o.value);
  const metadataKeys = new Set(
    manifest.envelope_entries.flatMap((e) => Object.keys(e.metadata ?? {})),
  );
  for (const key of metadataKeys) {
    assert.ok(options.includes(key), `metadata key ${key} not offered for grouping`);
  }

  // Every envelope is reachable through the tree.
  for (const entry of manifest.envelope_entries) {
    assert.ok(
      panel.textContent.includes(entry.envelope_name),
      `envelope ${entry.envelope_name} missing from the explorer`,
    );
  }
});

test("explorer search filters the envelope list", () => {
  const { document, window } = boot(sampleManifest());
  const panel = openTab(document, "Explorer");

  const input = panel.querySelector("input");
  assert.ok(input, "no search box");
  const before = panel.querySelectorAll(".leaf").length;
  assert.ok(before > 0, "no envelopes listed to filter");

  input.value = "env_a";
  input.dispatchEvent(new window.Event("input", { bubbles: true }));

  const after = panel.querySelectorAll(".leaf").length;
  assert.ok(after < before, `search did not narrow the list (${before} -> ${after})`);
  assert.ok(after > 0, "search matched nothing");
});

test("id map draws to the canvas", () => {
  const { document, errors } = boot(sampleManifest());
  const panel = openTab(document, "ID Map");

  const canvas = panel.querySelector("canvas");
  assert.ok(canvas, "no canvas in the ID map");
  assert.deepEqual(errors, [], "drawing the id map raised errors");
});

test("diff reports added, removed and reassigned envelopes", async () => {
  const manifest = sampleManifest();
  const { document, window } = boot(manifest);
  const panel = openTab(document, "Diff");

  // A prior build: one envelope since removed, one since reassigned, and one
  // of the current envelopes absent (so it reads as added).
  const previous = structuredClone(manifest);
  previous.envelope_entries = [
    { ...previous.envelope_entries[0], filler_name: "universe_999" },
    { ...previous.envelope_entries[1] },
    { envelope_name: "env_gone", filler_name: "universe_101", universe_id: 101 },
  ];

  const dropzone = panel.querySelector(".dropzone");
  assert.ok(dropzone, "no dropzone in the diff tab");

  const event = new window.Event("drop", { bubbles: true });
  event.dataTransfer = {
    files: [new window.File([JSON.stringify(previous)], "build_report.json")],
  };
  dropzone.dispatchEvent(event);

  // FileReader resolves asynchronously.
  await new Promise((resolve) => setTimeout(resolve, 50));

  const counts = textOf(document, ".panel.active .kpi .v").map(Number);
  assert.deepEqual(
    counts.slice(0, 3),
    [1, 1, 1],
    "expected one added, one removed and one changed envelope",
  );
});

test("a deep link opens the tab it names", () => {
  const { document } = boot(sampleManifest(), { hash: "#tab=fillers" });
  const active = document.querySelector(".tab.active");
  assert.ok(active.textContent.includes("Fillers"), "hash did not select the Fillers tab");
});

test("the theme toggle flips and is remembered", () => {
  const { document, window } = boot(sampleManifest());
  const root = document.documentElement;
  const before = root.getAttribute("data-theme");

  document
    .querySelector('button[aria-label="Toggle theme"]')
    .dispatchEvent(new window.MouseEvent("click", { bubbles: true }));

  const after = root.getAttribute("data-theme");
  assert.notEqual(after, before, "theme did not change");
  assert.equal(window.localStorage.getItem("gitronics-theme"), after, "theme not persisted");
});

test("manifest values are never interpreted as markup", () => {
  const manifest = sampleManifest();
  manifest.filler_entries[0].name = '<img src=x onerror=alert(1)>';
  manifest.envelope_entries[0].envelope_name = "</script><b>pwned</b>";

  const { document, errors } = boot(manifest);
  // Filler names render on the Fillers tab, envelope names on the Explorer.
  openTab(document, "Fillers");
  openTab(document, "Explorer");

  assert.deepEqual(errors, [], "hostile manifest values raised errors");
  // The page uses <b> itself, so look for the injected markup specifically.
  assert.equal(document.querySelectorAll("img").length, 0, "a manifest value became an element");
  assert.equal(
    [...document.querySelectorAll("b")].filter((n) => n.textContent === "pwned").length,
    0,
    "a manifest value became an element",
  );
  assert.ok(
    document.body.textContent.includes("<img src=x onerror=alert(1)>"),
    "the value should render as literal text",
  );
  assert.ok(
    document.body.textContent.includes("</script><b>pwned</b>"),
    "the value should render as literal text",
  );
});
