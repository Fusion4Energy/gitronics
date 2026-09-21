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

test("overview notes partial envelope metadata only", () => {
  const manifest = sampleManifest();
  const entries = manifest.envelope_entries;
  const withMetadata = entries.filter((entry) => entry.metadata && Object.keys(entry.metadata).length).length;
  assert.ok(withMetadata > 0 && withMetadata < entries.length, "fixture must have partial metadata");
  assert.ok(
    openTab(boot(manifest).document, "Overview").textContent.includes(`${withMetadata} with metadata`),
    "partial metadata not noted",
  );

  entries.forEach((entry) => { delete entry.metadata; });
  assert.ok(
    !openTab(boot(manifest).document, "Overview").textContent.includes("with metadata"),
    "no-metadata report should not mention metadata",
  );

  entries.forEach((entry) => { entry.metadata = { zone: "A" }; });
  assert.ok(
    !openTab(boot(manifest).document, "Overview").textContent.includes("with metadata"),
    "complete metadata needs no note",
  );
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

test("explorer shows the description whatever the case of its metadata key", () => {
  const manifest = sampleManifest();
  const entry = manifest.envelope_entries.find((e) => e.metadata?.description);
  assert.ok(entry, "fixture has no envelope with a description");
  entry.metadata = { Description: entry.metadata.description, ...entry.metadata };
  delete entry.metadata.description;

  const { document } = boot(manifest);
  const panel = openTab(document, "Explorer");
  const descs = [...panel.querySelectorAll(".desc")].map((n) => n.textContent);
  assert.ok(descs.includes("Blanket A"), `capitalised Description not shown: ${descs}`);
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

  await waitForDiff(panel);

  const counts = textOf(document, ".panel.active .kpi .v").map(Number);
  assert.deepEqual(
    counts.slice(0, 3),
    [1, 1, 1],
    "expected one added, one removed and one changed envelope",
  );
});

function waitForDiff(panel) {
  return new Promise((resolve, reject) => {
    const observer = new panel.ownerDocument.defaultView.MutationObserver(() => {
      if (!panel.querySelector(".diff-summary, .section-gap > .empty")) return;
      clearTimeout(timeout);
      observer.disconnect();
      resolve();
    });
    const timeout = setTimeout(() => {
      observer.disconnect();
      reject(new Error("Diff did not finish rendering"));
    }, 2000);
    observer.observe(panel, { childList: true, subtree: true });
  });
}

async function compareReports(manifest, previous) {
  const result = boot(manifest);
  const panel = openTab(result.document, "Diff");
  const event = new result.window.Event("drop", { bubbles: true });
  event.dataTransfer = {
    files: [new result.window.File([JSON.stringify(previous)], "baseline.json")],
  };
  panel.querySelector(".dropzone").dispatchEvent(event);
  await waitForDiff(panel);
  return { ...result, panel };
}

test("diff groups repeated substitutions and bounds expanded placements", async () => {
  const manifest = sampleManifest();
  manifest.envelope_entries = Array.from({ length: 60 }, (_, index) => ({
    ...manifest.envelope_entries[0], envelope_name: `placement_${index}`,
  }));
  const previous = structuredClone(manifest);
  previous.envelope_entries.forEach((entry) => { entry.filler_name = "old_filler"; });
  const { panel, window, errors } = await compareReports(manifest, previous);
  assert.equal(panel.querySelectorAll(".diff-group").length, 1);
  const group = panel.querySelector(".diff-group");
  assert.match(group.querySelector("summary").textContent, /60 placements/);
  group.open = true;
  group.dispatchEvent(new window.Event("toggle"));
  assert.equal(group.querySelectorAll("tbody tr").length, 25);
  group.querySelector('button[aria-label="Next page"]').click();
  assert.equal(group.querySelectorAll("tbody tr").length, 25);
  assert.match(group.textContent, /26-50 of 60/);
  assert.deepEqual(errors, []);
});

test("diff filters assignments by field, metadata and either filler name", async () => {
  const manifest = sampleManifest();
  const previous = structuredClone(manifest);
  previous.envelope_entries[0].filler_name = "old_filler";
  previous.envelope_entries[0].transform = "old_transform";
  previous.envelope_entries[1].filler_name = "another_filler";
  previous.envelope_entries[1].metadata.sector = "2";
  const { panel, window, errors } = await compareReports(manifest, previous);
  const select = (label, value) => {
    const control = panel.querySelector(`select[aria-label="${label}"]`);
    control.value = value;
    control.dispatchEvent(new window.Event("change"));
  };
  select("Assignment change type", "transform");
  assert.equal(panel.querySelectorAll(".diff-group").length, 1);
  assert.match(panel.querySelector(".diff-group").textContent, /old_filler/);
  select("Assignment change type", "all");
  select("Filter by metadata", "sector");
  select("Metadata value", "2");
  assert.match(panel.querySelector(".diff-group").textContent, /another_filler/);
  select("Metadata value", "");
  const search = panel.querySelector('input[aria-label="Search envelopes, fillers, or metadata"]');
  search.value = "old_filler";
  search.dispatchEvent(new window.Event("input"));
  assert.equal(panel.querySelectorAll(".diff-group").length, 1);
  assert.deepEqual(errors, []);
});

test("diff collapses path-only inputs and preserves full provenance", async () => {
  const manifest = sampleManifest();
  manifest.evidence.inputs = [
    { role: "filler", name: "universe_101", path: "new/location/filler.mcnp", sha256: "same" },
    { role: "source", name: "source", path: "source.mcnp", sha256: "new" },
  ];
  const previous = structuredClone(manifest);
  previous.evidence.inputs[0].path = "old/location/filler.mcnp";
  previous.evidence.inputs[1].sha256 = "old";
  const { panel, window, document, errors } = await compareReports(manifest, previous);
  const paths = panel.querySelector(".diff-path-changes");
  assert.equal(paths.open, false);
  assert.equal(paths.querySelectorAll("tbody tr").length, 0);
  assert.match(paths.textContent, /Path-only changes \(1\)/);
  assert.match(panel.querySelector("#diff-inputs").textContent, /Content changed/);
  panel.querySelector('button[aria-controls="diff-inputs"]').click();
  assert.equal(panel.querySelector("#diff-inputs").hidden, false);
  paths.open = true;
  paths.dispatchEvent(new window.Event("toggle"));
  paths.querySelector(".textbtn").click();
  const drawer = document.querySelector(".drawer");
  assert.ok(drawer.textContent.includes("old/location/filler.mcnp"));
  assert.ok(drawer.textContent.includes("new/location/filler.mcnp"));
  assert.deepEqual(errors, []);
});

test("diff compares warning frequencies, resolved messages and check results", async () => {
  const manifest = sampleManifest();
  manifest.warnings = ["new warning", "new warning", "recurring", "recurring"];
  manifest.checks = [{ name: "Validation", status: "failed", details: ["missing reference"] }];
  const previous = structuredClone(manifest);
  previous.warnings = ["resolved warning", "recurring"];
  previous.checks[0] = { name: "Validation", status: "passed", details: [] };
  const { panel, errors } = await compareReports(manifest, previous);
  const diagnostics = panel.querySelector("#diff-diagnostics");
  assert.match(diagnostics.textContent, /2 baseline \/ 4 current occurrences; 2 distinct current messages/);
  assert.match(diagnostics.textContent, /new warningNew02\+2/);
  assert.match(diagnostics.textContent, /resolved warningResolved10-1/);
  assert.match(diagnostics.textContent, /recurringCount changed12\+1/);
  assert.match(diagnostics.textContent, /Validationpassedfailed/);
  assert.deepEqual(errors, []);
});

test("diff does not treat missing hashes as identical content", async () => {
  const manifest = sampleManifest();
  manifest.evidence.inputs = [{ role: "filler", name: "universe_101", path: "new.mcnp" }];
  const previous = structuredClone(manifest);
  previous.evidence.inputs[0].path = "old.mcnp";
  const { panel, errors } = await compareReports(manifest, previous);
  assert.match(panel.textContent, /Content comparison is incomplete/);
  assert.match(panel.querySelector("#diff-inputs").textContent, /Not comparable/);
  assert.equal(panel.querySelector(".diff-path-changes"), null);
  assert.deepEqual(errors, []);
});

test("diff placement drawers compare both builds including removed envelopes", async () => {
  const manifest = sampleManifest();
  const previous = structuredClone(manifest);
  previous.envelope_entries[0].transform = "BASE_TRANSFORM";
  previous.envelope_entries[0].metadata.sector = "9";
  previous.envelope_entries.push({ envelope_name: "removed_placement", filler_name: "universe_101", universe_id: 101 });
  const { panel, document, window, errors } = await compareReports(manifest, previous);
  [...panel.querySelectorAll(".diff-modes button")].find((button) => button.textContent === "By envelope").click();
  [...panel.querySelectorAll("#diff-assignments .textbtn")].find((button) => button.textContent === "env_a").click();
  const drawer = document.querySelector(".drawer");
  assert.match(drawer.textContent, /BASE_TRANSFORM/);
  assert.match(drawer.textContent, /Metadata: sector91/);
  assert.ok(drawer.querySelectorAll(".diff-changed").length >= 2);
  assert.ok(drawer.querySelectorAll(".diff-unchanged").length > 0);
  drawer.querySelector('[aria-label="Close details"]').click();
  const removed = [...panel.querySelectorAll("#diff-assignments .textbtn")].find((button) => button.textContent === "removed_placement");
  removed.focus();
  removed.click();
  assert.match(drawer.textContent, /removed_placement/);
  assert.match(drawer.textContent, /universe_101Absent/);
  document.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape" }));
  assert.equal(drawer.hidden, true);
  assert.equal(document.activeElement, removed);
  assert.deepEqual(errors, []);
});

test("diff compares replacement components through their shared assignments", async () => {
  const manifest = sampleManifest();
  const previous = structuredClone(manifest);
  previous.filler_entries[0].name = "old_component";
  previous.filler_entries[0].cell_count = 140;
  previous.filler_entries[0].cell_id_runs = [[7, 12], [20, 40]];
  previous.envelope_entries.filter((entry) => entry.filler_name).forEach((entry) => { entry.filler_name = "old_component"; });
  const { panel, document, window, errors } = await compareReports(manifest, previous);
  const components = panel.querySelector("#diff-components");
  assert.match(components.textContent, /Assignment replacement/);
  assert.match(components.textContent, /2 reassigned placements/);
  const row = [...components.querySelectorAll("tbody tr")].find((row) => row.textContent.includes("Assignment replacement"));
  row.querySelector("button").click();
  const drawer = document.querySelector(".drawer");
  assert.match(drawer.textContent, /cell_count140120/);
  const runs = [...drawer.querySelectorAll("details")].find((detail) => detail.textContent.includes("cell_id_runs"));
  runs.open = true;
  runs.dispatchEvent(new window.Event("toggle"));
  assert.match(runs.textContent, /\[\[7,12\],\[20,40\]\]/);
  assert.match(runs.textContent, /250000/);
  assert.deepEqual(errors, []);
});

test("diff build summary itemizes lists and distinguishes empty selections", async () => {
  const manifest = sampleManifest();
  manifest.tallies = ["flux", "<dose>"];
  manifest.source = "current_source";
  const previous = structuredClone(manifest);
  previous.tallies = [];
  previous.source = "baseline_source";
  const { panel, errors } = await compareReports(manifest, previous);
  const summary = [...panel.querySelectorAll("details")].find((detail) => detail.querySelector("summary").textContent === "Build summary changes");
  const tallyRow = [...summary.querySelectorAll("tbody tr")].find((row) => row.firstChild.textContent === "tallies");
  assert.equal(tallyRow.children[1].textContent, "None");
  assert.deepEqual([...tallyRow.querySelectorAll("li")].map((item) => item.textContent), ["flux", "<dose>"]);
  assert.equal(tallyRow.querySelector("dose"), null);
  assert.match(summary.textContent, /sourcebaseline_sourcecurrent_source/);
  assert.deepEqual(errors, []);
});

test("diff card list prioritizes modifications and opens safe highlighted line changes", async () => {
  const manifest = sampleManifest();
  const context = Array.from({ length: 12 }, (_, index) => `c unchanged ${index}\n`).join("");
  manifest.evidence.data_cards = [
    { name: "m1", category: "materials", input: "materials.mcnp", text: `${context}m1 1001 2\n<img src=x onerror=alert(1)>\n` },
    { name: "f4", category: "tallies", input: "tallies.mcnp", text: "f4:n 1\n" },
  ];
  const previous = structuredClone(manifest);
  previous.evidence.data_cards = [
    { ...manifest.evidence.data_cards[0], text: `${context}m1 1001 1\n` },
    { name: "f8", category: "tallies", input: "old.mcnp", text: "f8:n 1\n" },
  ];
  const { panel, document, window, errors } = await compareReports(manifest, previous);
  const cards = panel.querySelector("#diff-cards");
  assert.match(cards.textContent, /1 modified \/ 1 added \/ 1 removed/);
  assert.equal(cards.querySelector("pre"), null);
  assert.equal(cards.querySelector(".textbtn").textContent, "m1");
  cards.querySelector(".textbtn").click();
  const drawer = document.querySelector(".drawer");
  assert.match(drawer.querySelector(".diff-line-removed").textContent, /m1 1001 1/);
  assert.match(drawer.querySelector(".diff-line-added").textContent, /m1 1001 2/);
  assert.match(drawer.querySelector(".diff-line-folded").textContent, /9 unchanged lines/);
  assert.equal(drawer.querySelector("img"), null);
  const checkbox = drawer.querySelector('input[type="checkbox"]');
  checkbox.checked = true;
  checkbox.dispatchEvent(new window.Event("change"));
  assert.equal(drawer.querySelector(".diff-line-folded"), null);
  assert.match(drawer.textContent, /c unchanged 0/);
  drawer.querySelector('[aria-label="Close details"]').click();
  const filter = cards.querySelector('select[aria-label="Data card change type"]');
  filter.value = "Added";
  filter.dispatchEvent(new window.Event("change"));
  assert.equal(cards.querySelectorAll("tbody tr").length, 1);
  cards.querySelector(".textbtn").click();
  assert.equal(drawer.querySelector(".diff-line-removed"), null);
  assert.match(drawer.querySelector(".diff-line-added").textContent, /f4:n 1/);
  assert.deepEqual(errors, []);
});

test("diff omits missing-newline annotations without changing card content", async () => {
  for (const suffix of ["", "\nc unchanged final line"]) {
    const manifest = sampleManifest();
    manifest.evidence.data_cards = [{ name: "M1", category: "Materials", input: "materials.mcnp", text: `M1 1001 2${suffix}` }];
    const previous = structuredClone(manifest);
    previous.evidence.data_cards[0].text = `M1 1001 1${suffix}`;
    const { panel, document, window, errors } = await compareReports(manifest, previous);
    panel.querySelector("#diff-cards .textbtn").click();
    const drawer = document.querySelector(".drawer");
    assert.equal(drawer.querySelector(".diff-line-removed pre").textContent, "M1 1001 1");
    assert.equal(drawer.querySelector(".diff-line-added pre").textContent, "M1 1001 2");
    if (suffix) assert.equal(drawer.querySelector(".diff-line-context pre").textContent, "c unchanged final line");
    assert.doesNotMatch(drawer.textContent, /No newline at end of file/);
    const full = [...drawer.querySelectorAll("details")].find((detail) => detail.querySelector("summary").textContent === "Full definitions");
    full.open = true; full.dispatchEvent(new window.Event("toggle"));
    assert.deepEqual([...full.querySelectorAll("pre")].map((node) => node.textContent), [previous.evidence.data_cards[0].text, manifest.evidence.data_cards[0].text]);
    assert.deepEqual(errors, []);
  }
});

test("diff groups tally changes by ID with unchanged members and per-card comparison", async () => {
  const manifest = sampleManifest();
  manifest.evidence.data_cards = [
    { name: "FC1014", category: "Tallies", input: "flux.tally", text: "FC1014 flux\n" },
    { name: "FM1014", category: "Tallies", input: "multipliers.tally", text: "FM1014 2\n" },
    { name: "FMESH1014:N", category: "Tallies", input: "flux.tally", text: "FMESH1014:N GEOM=XYZ\n" },
    { name: "E1014", category: "Tallies", input: "flux.tally", text: "E1014 0 20\n" },
    { name: "M1014", category: "Materials", input: "materials", text: "M1014 1001 1\n" },
  ];
  const previous = structuredClone(manifest);
  previous.evidence.data_cards[1].text = "FM1014 1\n";
  previous.evidence.data_cards = previous.evidence.data_cards.filter((card) => !["E1014", "M1014"].includes(card.name));
  previous.evidence.data_cards.push({ name: "FT1014", category: "Tallies", input: "flux.tally", text: "FT1014 SCX\n" });
  const { panel, document, window, errors } = await compareReports(manifest, previous);
  const cards = panel.querySelector("#diff-cards");
  assert.equal(cards.querySelectorAll("tbody tr").length, 2);
  assert.match(cards.textContent, /2 items \(1 tally group\); 4 individual card changes/);
  const search = cards.querySelector('input[type="search"]');
  search.value = "FC1014"; search.dispatchEvent(new window.Event("input"));
  assert.equal(cards.querySelectorAll("tbody tr").length, 1);
  assert.equal(cards.querySelector(".textbtn").textContent, "Tally 1014");
  cards.querySelector(".textbtn").click();
  const drawer = document.querySelector(".drawer");
  assert.match(drawer.textContent, /3 changed \/ 5 member cards/);
  assert.equal(drawer.querySelectorAll(".diff-tally-members tbody tr").length, 5);
  assert.match(drawer.querySelector(".diff-line-removed").textContent, /FM1014 1/);
  assert.match(drawer.querySelector(".diff-line-added").textContent, /FM1014 2/);
  const selectMember = (name) => [...drawer.querySelectorAll(".diff-tally-members button")].find((button) => button.textContent === name).click();
  selectMember("FMESH1014:N");
  assert.match(drawer.querySelector(".diff-tally-definition").textContent, /Unchanged/);
  assert.match(drawer.querySelector(".diff-tally-definition").textContent, /GEOM=XYZ/);
  selectMember("E1014");
  assert.equal(drawer.querySelector(".diff-line-removed"), null);
  assert.match(drawer.querySelector(".diff-line-added").textContent, /E1014 0 20/);
  selectMember("FT1014");
  assert.equal(drawer.querySelector(".diff-line-added"), null);
  assert.match(drawer.querySelector(".diff-line-removed").textContent, /FT1014 SCX/);
  assert.deepEqual(errors, []);
});

test("diff tally grouping ignores member order and preserves removed tallies and global defaults", async () => {
  const manifest = sampleManifest();
  manifest.evidence.data_cards = [
    { name: "*F24:N", category: "Tallies", input: "flux.tally", text: "*F24:N 1\n" },
    { name: "FM24", category: "Tallies", input: "flux.tally", text: "FM24 2\n" },
  ];
  const previous = structuredClone(manifest);
  previous.evidence.data_cards.reverse();
  const unchanged = await compareReports(manifest, previous);
  assert.equal(unchanged.panel.querySelector("#diff-cards"), null);
  assert.match(unchanged.panel.textContent, /No differences in recorded build content/);
  previous.evidence.data_cards.push(
    { name: "F34:N", category: "Tallies", input: "flux.tally", text: "F34:N 1\n" },
    { name: "FC34", category: "Tallies", input: "flux.tally", text: "FC34 removed\n" },
    { name: "E0", category: "Tallies", input: "flux.tally", text: "E0 0 20\n" },
  );
  const { panel, document, errors } = await compareReports(manifest, previous);
  const cards = panel.querySelector("#diff-cards");
  assert.deepEqual([...cards.querySelectorAll(".textbtn")].map((button) => button.textContent), ["E0", "Tally 34"]);
  assert.match(cards.textContent, /0 modified \/ 0 added \/ 2 removed/);
  [...cards.querySelectorAll(".textbtn")].find((button) => button.textContent === "Tally 34").click();
  assert.match(document.querySelector(".drawer").textContent, /2 changed \/ 2 member cards/);
  assert.equal(document.querySelector(".drawer .diff-line-added"), null);
  assert.deepEqual([...unchanged.errors, ...errors], []);
});

test("diff supports identical legacy reports and rejects unsupported schemas", async () => {
  const manifest = sampleManifest();
  manifest.schema_version = 1;
  delete manifest.evidence;
  delete manifest.warnings;
  delete manifest.checks;
  const { panel, window, errors } = await compareReports(manifest, structuredClone(manifest));
  assert.match(panel.textContent, /No differences in recorded build content/);
  assert.match(panel.textContent, /Content comparison is incomplete/);
  assert.match(panel.textContent, /diagnostics were not recorded/);
  const event = new window.Event("drop", { bubbles: true });
  event.dataTransfer = { files: [new window.File([JSON.stringify({ schema_version: 99 })], "unsupported.json")] };
  panel.querySelector(".dropzone").dispatchEvent(event);
  await waitForDiff(panel);
  assert.match(panel.textContent, /Unsupported report/);
  assert.equal(panel.querySelector(".diff-summary"), null);
  assert.deepEqual(errors, []);
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
  await waitForDiff(panel);
  assert.deepEqual(errors, []);
  const group = panel.querySelector(".diff-group");
  group.open = true;
  group.dispatchEvent(new window.Event("toggle"));
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

test("ID maps resolve data definitions, sparse gaps, and shared universe owners", async () => {
  const manifest = sampleManifest();
  manifest.envelope_structure.universe_id_runs = [[0, 0], [101, 101]];
  manifest.filler_entries[0].universe_id_runs = [[101, 102], [105, 105]];
  manifest.evidence = {
    inputs: [{ name: "mixed", path: "mixed.mat", role: "materials" }, { name: "flux", path: "flux.tally", role: "tallies" }],
    data_cards: [
      { name: "M1", input: "mixed.mat", text: "M1 1001 1" },
      { name: "M3", input: "mixed.mat", text: "M3 8016 1" },
      { name: "MT2", input: "mixed.mat", text: "MT2 lwtr" },
      { name: "*TR40", input: "mixed.mat", text: "*TR40 0 0 0" },
      { name: "TR42", input: "mixed.mat", text: "TR42 0 0 0" },
      { name: "F4:N", input: "flux.tally", text: "F4:N 1" },
      { name: "F4:P", input: "flux.tally", text: "F4:P 1" },
      { name: "FMESH14:N", input: "flux.tally", text: "FMESH14:N GEOM=XYZ" },
      { name: "FC5", input: "flux.tally", text: "FC5 comment" }
    ]
  };
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "ID Map");
  assert.equal(panel.querySelectorAll("canvas").length, 6);
  const lookup = (label, id) => {
    const input = panel.querySelector(`[aria-label="Find ${label} ids"]`);
    input.value = String(id);
    input.closest("form").dispatchEvent(new window.Event("submit", { cancelable: true }));
    return input.closest(".idmap-lane").querySelector(".id-lookup-result");
  };
  assert.match(lookup("material", 1).textContent, /mixed/);
  assert.match(lookup("material", 2).textContent, /unused/);
  const result = lookup("transformation", 40);
  assert.match(result.textContent, /mixed/);
  result.querySelector("button").click();
  assert.match(document.querySelector(".drawer").textContent, /\*TR40 0 0 0/);
  assert.match(document.querySelector(".drawer").textContent, /mixed.mat/);
  document.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape" }));
  assert.match(lookup("transformation", 41).textContent, /unused/);
  lookup("tally", 4).querySelector("button").click();
  assert.match(document.querySelector(".drawer").textContent, /F4:N 1/);
  assert.match(document.querySelector(".drawer").textContent, /F4:P 1/);
  document.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape" }));
  assert.match(lookup("tally", 5).textContent, /unused/);
  assert.match(lookup("tally", 14).textContent, /flux/);
  assert.match(lookup("universe", 0).textContent, /Envelope structure/);
  assert.match(lookup("universe", 101).textContent, /universe_101, Envelope structure/);
  assert.match(lookup("universe", 103).textContent, /unused/);
  lookup("universe", 105).querySelector("button").click();
  assert.match(document.querySelector(".drawer").textContent, /universe_101/);
  await new Promise((resolve) => window.requestAnimationFrame(resolve));
  assert.match(panel.querySelector('[data-id-kind="Tally ids"] .mono').textContent, /2 used/);
  assert.match(panel.querySelector('[data-id-kind="Universe ids"] .mono').textContent, /4 used/);
  assert.deepEqual(errors, []);
});

test("ID maps retain overlapping and duplicate owners without inflating counts", async () => {
  const manifest = sampleManifest();
  manifest.envelope_structure.universe_id_runs = [[0, 4]];
  manifest.filler_entries[0].universe_id_runs = [[1, 2]];
  manifest.evidence = {
    data_cards: [
      { name: "M1", input: "first.mat", text: "M1 1001 1" },
      { name: "M1", input: "second.mat", text: "M1 1002 1" }
    ]
  };
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "ID Map");
  const material = panel.querySelector('[aria-label="Find material ids"]');
  material.value = "1"; material.closest("form").dispatchEvent(new window.Event("submit", { cancelable: true }));
  const lane = material.closest(".idmap-lane");
  assert.match(lane.querySelector(".id-lookup-result").textContent, /first.mat, second.mat/);
  assert.equal(lane.querySelectorAll(".id-lookup-result button").length, 2);
  lane.querySelectorAll(".id-lookup-result button")[1].click();
  assert.match(document.querySelector(".drawer").textContent, /M1 1002 1/);
  document.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape" }));
  const universe = panel.querySelector('[aria-label="Find universe ids"]');
  universe.value = "3"; universe.closest("form").dispatchEvent(new window.Event("submit", { cancelable: true }));
  assert.match(universe.closest(".idmap-lane").querySelector(".id-lookup-result").textContent, /Envelope structure/);
  await new Promise((resolve) => window.requestAnimationFrame(resolve));
  assert.match(lane.querySelector(".mono").textContent, /1 used/);
  assert.match(universe.closest(".idmap-lane").querySelector(".mono").textContent, /5 used/);
  assert.deepEqual(errors, []);
});

test("ID maps distinguish absent definitions from inventories not recorded", () => {
  const manifest = sampleManifest();
  delete manifest.evidence;
  delete manifest.envelope_structure.universe_id_runs;
  const { document, errors } = boot(manifest);
  const panel = openTab(document, "ID Map");
  assert.match(panel.querySelector('[data-id-kind="Material ids"]').textContent, /not recorded/);
  assert.match(panel.querySelector('[data-id-kind="Universe ids"]').textContent, /not recorded/);
  manifest.evidence = { data_cards: [] };
  const empty = boot(manifest);
  assert.match(openTab(empty.document, "ID Map").querySelector('[data-id-kind="Tally ids"]').textContent, /No IDs defined/);
  assert.deepEqual([...errors, ...empty.errors], []);
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

test("data cards group tally members by ID without hiding matching context or file ownership", () => {
  const manifest = sampleManifest();
  manifest.materials = []; manifest.tallies = ["flux", "other"]; delete manifest.source;
  manifest.evidence.inputs = [{ name: "flux", role: "tallies", path: "flux.tally" }, { name: "other", role: "tallies", path: "other.tally" }];
  manifest.evidence.data_cards = [
    { name: "FC1014", category: "Tallies", input: "flux.tally", text: "FC1014 flux" },
    { name: "FM1014", category: "Tallies", input: "flux.tally", text: "FM1014 1" },
    { name: "FMESH1014:N", category: "Tallies", input: "flux.tally", text: "FMESH1014:N GEOM=XYZ" },
    { name: "E0", category: "Tallies", input: "flux.tally", text: "E0 0 20" },
    { name: "DF1014", category: "Tallies", input: "other.tally", text: "DF1014 1 2" },
    { name: "M1014", category: "Materials", input: "flux.tally", text: "M1014 1001 1" },
  ];
  const { document, window, errors } = boot(manifest);
  const panel = openTab(document, "Data Cards");
  assert.match(panel.textContent, /1 tally, 4 cards/);
  assert.equal(panel.querySelectorAll(".data-tally-entry").length, 0);
  const search = panel.querySelector('input[type="search"]');
  search.value = "FM1014"; search.dispatchEvent(new window.Event("input"));
  assert.equal(panel.querySelectorAll(".data-file").length, 1);
  const tally = panel.querySelector(".data-tally-entry");
  assert.equal(tally.querySelector("summary").textContent, "Tally 1014 (3 cards)");
  assert.equal(tally.querySelectorAll(".card-details").length, 0);
  tally.open = true; tally.dispatchEvent(new window.Event("toggle"));
  assert.deepEqual([...tally.querySelectorAll(".data-card-entry > summary")].map((item) => item.textContent), ["FMESH1014:N", "FC1014", "FM1014"]);
  assert.equal(tally.querySelector("mark").textContent, "FM1014");
  const member = tally.querySelector(".data-card-entry");
  member.open = true; member.dispatchEvent(new window.Event("toggle"));
  assert.match(member.textContent, /GEOM=XYZ/);
  search.value = "1014"; search.dispatchEvent(new window.Event("input"));
  assert.equal(panel.querySelectorAll(".data-tally-entry").length, 2);
  assert.equal(panel.querySelectorAll(".data-card-entry").length, 1);
  assert.equal(panel.querySelector(".data-card-entry > summary").textContent, "M1014");
  search.value = "E0"; search.dispatchEvent(new window.Event("input"));
  assert.equal(panel.querySelectorAll(".data-tally-entry").length, 0);
  assert.equal(panel.querySelector(".data-card-entry > summary").textContent, "E0");
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
