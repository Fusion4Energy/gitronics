// Boots the build-report viewer in jsdom, the same way `gitronics build` boots
// it in a browser: the same template, the same escaping, the same three files.
//
// Nothing here may diverge from `src/build_report.rs` — if the substitution or
// the escaping drifts, these tests stop testing the shipped page.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM, VirtualConsole } from "jsdom";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPORT_DIR = join(HERE, "..");

const TEMPLATE = readFileSync(join(REPORT_DIR, "report.html"), "utf8");
const STYLE = readFileSync(join(REPORT_DIR, "report.css"), "utf8");
const SCRIPT = readFileSync(join(REPORT_DIR, "report.js"), "utf8");

/** Mirrors `escape_json_for_script` in src/build_report.rs. */
function escapeJsonForScript(json) {
  const map = {
    "<": "\\u003c",
    ">": "\\u003e",
    "&": "\\u0026",
    "\u2028": "\\u2028",
    "\u2029": "\\u2029",
  };
  return json.replace(/[<>&\u2028\u2029]/g, (c) => map[c]);
}

/** Mirrors `push_escaped_text` in src/build_report.rs. */
function escapeText(s) {
  return s.replace(/[&<>"']/g, (c) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  })[c]);
}

/** Renders the document exactly as `generate_html_from_json` does. */
export function renderReport(manifest) {
  const json = JSON.stringify(manifest, null, 2);
  const values = [
    ["{{TITLE}}", escapeText(manifest.config_path ?? "")],
    ["{{STYLE}}", STYLE],
    ["{{DATA}}", escapeJsonForScript(json)],
    ["{{SCRIPT}}", SCRIPT],
  ];

  let out = "";
  let rest = TEMPLATE;
  for (const [marker, value] of values) {
    const at = rest.indexOf(marker);
    if (at < 0) throw new Error(`template is missing ${marker}`);
    out += rest.slice(0, at) + value;
    rest = rest.slice(at + marker.length);
  }
  return out + rest;
}

/**
 * A no-op 2D context that records the calls made against it.
 *
 * jsdom returns null from getContext("2d") unless the native `canvas` package
 * is installed, and the ID-map lane dereferences the context immediately. A
 * recording stub keeps the dependency list to jsdom alone and lets a test
 * assert the lane actually drew something.
 */
function stubCanvas(window) {
  window.HTMLCanvasElement.prototype.getContext = function () {
    const calls = [];
    const record = (name) => (...args) => { calls.push([name, ...args]); };
    return {
      calls,
      canvas: this,
      setTransform: record("setTransform"),
      clearRect: record("clearRect"),
      fillRect: record("fillRect"),
      strokeRect: record("strokeRect"),
      beginPath: record("beginPath"),
      moveTo: record("moveTo"),
      lineTo: record("lineTo"),
      stroke: record("stroke"),
      fill: record("fill"),
      fillText: record("fillText"),
      save: record("save"),
      restore: record("restore"),
      measureText: () => ({ width: 0 }),
      set fillStyle(_) {}, get fillStyle() { return "#000"; },
      set strokeStyle(_) {}, get strokeStyle() { return "#000"; },
      set lineWidth(_) {}, get lineWidth() { return 1; },
      set font(_) {}, get font() { return "10px monospace"; },
      set globalAlpha(_) {}, get globalAlpha() { return 1; },
      set textAlign(_) {}, get textAlign() { return "center"; },
      set textBaseline(_) {}, get textBaseline() { return "middle"; },
    };
  };
}

/**
 * Boots the viewer against `manifest`.
 *
 * Returns the jsdom window plus every console error and jsdom error raised
 * during boot — a page that throws on load still produces a `<body>`, so the
 * error list is what distinguishes a working report from a blank one.
 */
export function boot(manifest, { hash = "" } = {}) {
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on("jsdomError", (e) => errors.push(e.message ?? String(e)));
  virtualConsole.on("error", (...args) => errors.push(args.join(" ")));

  const dom = new JSDOM(renderReport(manifest), {
    runScripts: "dangerously",
    pretendToBeVisual: true,
    url: `https://report.test/${hash}`,
    virtualConsole,
    beforeParse(window) {
      stubCanvas(window);
      // jsdom logs "Not implemented" for these; they are irrelevant to the
      // page's behaviour, and stubbing them keeps the error assertion strict.
      window.scrollTo = () => {};
      window.URL.createObjectURL = () => "blob:stub";
      window.URL.revokeObjectURL = () => {};
    },
  });

  return { dom, window: dom.window, document: dom.window.document, errors };
}

/** The manifest the Rust tests use, kept in step by `js_fixture_matches_sample_report`. */
export function sampleManifest() {
  return JSON.parse(
    readFileSync(join(HERE, "fixtures", "sample_report.json"), "utf8"),
  );
}

/** Text of every element matching `selector`. */
export function textOf(document, selector) {
  return [...document.querySelectorAll(selector)].map((n) => n.textContent.trim());
}
