// Behavioural tests for the build-report viewer.
//
// The failure these exist to catch is the silent one: report.js throws while
// booting, the Rust tests still pass because the HTML shell is well formed, and
// the user opens a blank page. Every test therefore asserts on a rendered DOM,
// and `boots_without_errors` asserts on the error channel directly.

import assert from "node:assert/strict";
import { test } from "node:test";
import { boot, sampleManifest, textOf } from "./harness.mjs";

const TABS = ["Overview", "Checks", "Explorer", "Coverage Map", "ID Map", "Fillers", "Data Cards", "Inputs", "Diff"];

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

test("overview reports configuration completeness separately from fill", () => {
  const manifest = sampleManifest();
  const { document } = boot(manifest);
  const text = openTab(document, "Overview").textContent;

  const configured = manifest.envelope_entries.filter((entry) => entry.status !== "unconfigured").length;
  const expected = Math.round((configured / manifest.envelope_entries.length) * 100);
  assert.ok(text.includes(`${expected}%`), `coverage ${expected}% not shown`);
  assert.ok(text.includes(String(manifest.total_cells)), "total cells not shown");
});

test("explorer groups envelopes by a discovered metadata key", () => {
  const manifest = sampleManifest();
  const { document } = boot(manifest);
  const panel = openTab(document, "Explorer");

  const select = panel.querySelector('select[aria-label="Group by metadata"]');
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

test("diff shows transform values and detects unchanged-name content", async () => {
  const manifest = sampleManifest();
  manifest.schema_version = 2;
  manifest.evidence = { inputs: [{ role: "filler", name: "universe_101", path: "filler.mcnp", sha256: "new" }] };
  const previous = structuredClone(manifest);
  previous.envelope_entries[0].transform = "(999)";
  previous.evidence.inputs[0].sha256 = "old";
  previous.filler_entries[0].cell_count += 1;
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "Diff");
  const event = new window.Event("drop", { bubbles: true });
  event.dataTransfer = { files: [new window.File([JSON.stringify(previous)], "previous.json")] };
  panel.querySelector(".dropzone").dispatchEvent(event);
  await new Promise((resolve) => setTimeout(resolve, 50));
  assert.deepEqual(errors, []);
  assert.ok(panel.textContent.includes("(999)"));
  assert.ok(panel.textContent.includes(manifest.envelope_entries[0].transform));
  assert.ok(panel.textContent.includes("Content changed"));
  assert.ok(panel.textContent.includes("cell_count"));
  assert.ok(panel.textContent.includes("2 placements"));
});

test("grouping defaults use values rather than project-specific names", () => {
  const manifest = sampleManifest();
  manifest.envelope_entries.forEach((entry, index) => { entry.metadata = { sector: "same", assembly_family: index === 2 ? "beta" : "alpha", serial: `item-${index}` }; });
  const { document } = boot(manifest);
  const panel = openTab(document, "Explorer");
  assert.equal(panel.querySelector('select[aria-label="Group by metadata"]').value, "assembly_family");
  assert.ok(panel.textContent.includes("alpha"));
});

test("arbitrary metadata values cannot collide with object properties", () => {
  const manifest = sampleManifest();
  manifest.envelope_entries.forEach((entry, index) => {
    entry.metadata = { component: index === 2 ? "toString" : "__proto__" };
  });
  const { document, errors } = boot(manifest);
  for (const name of ["Overview", "Explorer", "Coverage Map"]) openTab(document, name);
  assert.deepEqual(errors, []);
  assert.ok(document.querySelector("#panel-explorer").textContent.includes("__proto__"));
});

test("envelope details preserve placement identity and keyboard access", () => {
  const manifest = sampleManifest();
  manifest.envelope_entries[0].cell_ids = [42];
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "Explorer");
  const leaf = panel.querySelector(".leaf");
  leaf.focus();
  leaf.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  assert.ok(document.querySelector('.drawer[role="dialog"]').textContent.includes("42"));
  assert.ok(document.querySelector('.drawer').textContent.includes("env_a"));
  document.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape" }));
  assert.equal(document.querySelector(".drawer").hidden, true);
  assert.equal(document.activeElement, leaf);
  assert.deepEqual(errors, []);
});

test("map filters unconfigured envelopes and exposes named buttons", () => {
  const manifest = sampleManifest();
  manifest.envelope_entries[2].status = "unconfigured";
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "Coverage Map");
  const select = panel.querySelector('[aria-label="Map assignment status"]');
  select.value = "unconfigured"; select.dispatchEvent(new window.Event("change"));
  assert.equal(panel.querySelectorAll("button.cov-cell").length, 1);
  assert.match(panel.querySelector("button.cov-cell").getAttribute("aria-label"), /Not configured/);
  assert.deepEqual(errors, []);
});

test("ID lookup resolves exact owners and gaps", () => {
  const { document, window, errors } = boot(sampleManifest());
  const panel = openTab(document, "ID Map");
  const input = panel.querySelector('[aria-label="Find cell ids"]');
  input.value = "250000";
  input.closest("form").dispatchEvent(new window.Event("submit", { cancelable: true }));
  assert.match(panel.querySelector(".id-lookup-result").textContent, /universe_101/);
  input.value = "250050";
  input.closest("form").dispatchEvent(new window.Event("submit", { cancelable: true }));
  assert.match(panel.querySelector(".id-lookup-result").textContent, /unused/);
  assert.deepEqual(errors, []);
});

test("data files own grouped card IDs with definitions loaded on demand", () => {
  const manifest = sampleManifest();
  manifest.materials = ["mixed"]; manifest.source = "source"; manifest.tallies = [];
  manifest.evidence = {
    inputs: [{ name: "mixed", role: "materials", path: "mixed.mat" }, { name: "source", role: "source", path: "source" }],
    data_cards: [{ name: "M101", category: "Materials", input: "mixed.mat", text: "M101 1001 1", referenced_cell_runs: [] }, { name: "*TR40", category: "Transforms", input: "mixed.mat", text: "*TR40 0 0 0", referenced_cell_runs: [] }, { name: "NPS", category: "Run settings", input: "source", text: "NPS 100", referenced_cell_runs: [] }]
  };
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "Data Cards");
  assert.equal(document.querySelector('[aria-controls="panel-data"] .pill').textContent, "2");
  assert.equal(document.querySelector("#panel-overview .kpi:last-child .v").textContent, "2");
  assert.equal(panel.querySelectorAll(".data-file").length, 2);
  assert.equal(panel.querySelectorAll(".data-card-entry").length, 0);
  assert.equal(panel.querySelectorAll(".card-details").length, 0);
  const first = panel.querySelector(".data-file");
  assert.match(first.textContent, /Selected as: materials/);
  assert.match(first.textContent, /1 materials · 1 transforms/);
  first.open = true; first.dispatchEvent(new window.Event("toggle"));
  assert.equal(first.querySelectorAll(".data-card-group").length, 2);
  const card = [...first.querySelectorAll(".data-card-entry")].find((entry) => entry.textContent === "*TR40");
  assert.equal(panel.querySelectorAll(".card-details").length, 0);
  card.open = true; card.dispatchEvent(new window.Event("toggle"));
  assert.ok(panel.textContent.includes("*TR40 0 0 0"));
  const search = panel.querySelector('input[type="search"]');
  search.value = "TR40"; search.dispatchEvent(new window.Event("input"));
  assert.equal(panel.querySelectorAll(".data-file").length, 1);
  assert.equal(panel.querySelector(".data-file").open, true);
  assert.equal(panel.querySelector("mark").textContent, "*TR40");
  assert.equal(panel.querySelectorAll(".card-details").length, 0);
  search.value = "mixed.mat"; search.dispatchEvent(new window.Event("input"));
  assert.equal(panel.querySelectorAll(".data-card-entry").length, 2);
  search.value = ""; search.dispatchEvent(new window.Event("input"));
  const category = panel.querySelector('[aria-label="Card category"]');
  category.value = "Run settings"; category.dispatchEvent(new window.Event("change"));
  const selected = panel.querySelector(".data-card-entry");
  selected.open = true; selected.dispatchEvent(new window.Event("toggle"));
  assert.ok(panel.querySelector(".data-file-list").textContent.includes("NPS 100"));
  assert.ok(!panel.querySelector(".data-file-list").textContent.includes("*TR40"));
  assert.deepEqual(errors, []);
});

test("data files stay visible with no inventory and older schemas", () => {
  const manifest = sampleManifest();
  manifest.schema_version = 1;
  delete manifest.evidence;
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "Data Cards");
  assert.equal(panel.querySelectorAll(".data-file").length, 3);
  assert.match(panel.textContent, /Card inventory not recorded/);
  const search = panel.querySelector("input");
  search.value = "no-matching-file"; search.dispatchEvent(new window.Event("input"));
  assert.match(panel.textContent, /No matching data files/);
  assert.deepEqual(errors, []);
});

test("resolved file paths keep ownership distinct and combine selected roles", () => {
  const manifest = sampleManifest();
  manifest.materials = ["mixed", "other"]; manifest.transforms = ["mixed"]; manifest.tallies = [];
  delete manifest.source;
  manifest.evidence = {
    inputs: [{ name: "mixed", role: "materials", path: "a/mixed.mat" }, { name: "mixed", role: "transforms", path: "a/mixed.mat" }, { name: "other", role: "materials", path: "b/mixed.mat" }],
    data_cards: [{ name: "*TR40", category: "Transforms", input: "a/mixed.mat", text: "*TR40 0 0 0" }, { name: "M2", category: "Materials", input: "b/mixed.mat", text: "M2 1001 1" }]
  };
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "Data Cards");
  assert.equal(panel.querySelectorAll(".data-file").length, 2);
  assert.equal(document.querySelector('[aria-controls="panel-data"] .pill').textContent, "2");
  const file = panel.querySelector(".data-file");
  assert.match(file.textContent, /materials, transformations/);
  file.open = true; file.dispatchEvent(new window.Event("toggle"));
  assert.equal(file.querySelectorAll(".data-card-entry").length, 1);
  assert.equal(file.querySelector(".data-card-entry summary").textContent, "*TR40");
  assert.deepEqual(errors, []);
});

test("data files without parsed cards and no selection have explicit states", () => {
  const manifest = sampleManifest();
  manifest.materials = ["comments_only"]; manifest.tallies = []; delete manifest.source;
  manifest.evidence = { inputs: [{ name: "comments_only", role: "materials", path: "comments.mat" }], data_cards: [] };
  const { document, errors } = boot(manifest);
  const panel = openTab(document, "Data Cards");
  assert.equal(panel.querySelectorAll(".data-file").length, 1);
  assert.match(panel.textContent, /0 parsed cards/);
  manifest.materials = [];
  const empty = boot(manifest);
  assert.match(openTab(empty.document, "Data Cards").textContent, /No data files selected/);
  assert.deepEqual([...errors, ...empty.errors], []);
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
