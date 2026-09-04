// Lint rules for the build-report viewer.
//
// report.js is inlined into a plain <script> tag, not loaded as a module, so it
// is script-sourced and keeps its outer IIFE: top-level `const` would leak into
// global scope and "use strict" would be lost.
export default [
  {
    files: ["report.js"],
    languageOptions: {
      ecmaVersion: 2020,
      sourceType: "script",
      globals: {
        document: "readonly",
        window: "readonly",
        localStorage: "readonly",
        location: "readonly",
        requestAnimationFrame: "readonly",
        cancelAnimationFrame: "readonly",
        setTimeout: "readonly",
        matchMedia: "readonly",
        getComputedStyle: "readonly",
        Node: "readonly",
        Blob: "readonly",
        URL: "readonly",
        FileReader: "readonly",
        ResizeObserver: "readonly",
        history: "readonly",
      },
    },
    rules: {
      "no-var": "error",
      "prefer-const": "error",
      "prefer-template": "error",
      "prefer-arrow-callback": "error",
      "no-undef": "error",
      "no-unused-vars": "error",
      // `x == null` is deliberate throughout: it means "null or undefined".
      eqeqeq: ["error", "always", { null: "ignore" }],
    },
  },
  {
    files: ["test/**/*.mjs"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: { console: "readonly", setTimeout: "readonly", structuredClone: "readonly" },
    },
    rules: { "no-var": "error", "prefer-const": "error" },
  },
];
