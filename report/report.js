/* ============================================================================
   gitronics build report — interactive viewer
   Vanilla JS, zero dependencies, fully offline. Hydrates the UI from the
   JSON manifest embedded in <script id="report-data">.
   All user-derived strings are inserted via textContent → no HTML injection.
   ========================================================================== */
(function () {
    "use strict";

    // ── Load data ──────────────────────────────────────────────────────────
    let DATA;
    try {
        DATA = JSON.parse(document.getElementById("report-data").textContent);
    } catch (e) {
        document.body.textContent = `Failed to parse report data: ${e}`;
        return;
    }

    const envelopes = DATA.envelope_entries || [];
    const fillers = DATA.filler_entries || [];
    const evidence = DATA.evidence || {};
    const inputs = evidence.inputs || [];
    const dataCards = evidence.data_cards || [];
    const dataFilesByPath = new Map();
    for (const [role, names] of [["materials", DATA.materials || []], ["transforms", DATA.transforms || []],
    ["tallies", DATA.tallies || []], ["source", DATA.source ? [DATA.source] : []]]) {
        for (const name of names) {
            const input = inputs.find((entry) => entry.role === role && entry.name === name);
            const key = input ? input.path : `${role}:${name}`;
            if (!dataFilesByPath.has(key)) dataFilesByPath.set(key, { name, path: input ? input.path : null, roles: [], cards: [] });
            const file = dataFilesByPath.get(key);
            if (!file.roles.includes(role)) file.roles.push(role);
        }
    }
    for (const card of dataCards) {
        const file = dataFilesByPath.get(card.input);
        if (file) file.cards.push(card);
    }
    const dataFiles = [...dataFilesByPath.values()];
    const statusOf = (entry) => entry.status || (entry.filler_name ? "filled" : "empty");
    const statusLabel = (status) => ({ filled: "Filled", empty: "Explicitly empty", unconfigured: "Not configured" }[status] || status);
    const fillerByName = Object.create(null);
    fillers.forEach((f) => { fillerByName[f.name] = f; });
    // Synthetic pseudo-filler carrying the envelope structure file's own exact
    // id runs, so the ID map can plot them alongside the fillers' ids.
    const envelopeStructure = DATA.envelope_structure
        ? Object.assign({ name: "Envelope structure" }, DATA.envelope_structure)
        : null;

    // ── Free-form metadata helpers ─────────────────────────────────────────
    // Metadata keys are project-defined and arbitrary; discover them from data.
    function discoverKeys(items) {
        const order = [], seen = Object.create(null);
        items.forEach((it) => {
            const m = it.metadata; if (!m) return;
            Object.keys(m).forEach((k) => { if (m[k] != null && metaVal(m[k]) !== "" && !seen[k]) { seen[k] = 1; order.push(k); } });
        });
        return order;
    }
    const envMetaKeys = discoverKeys(envelopes);
    const fillerMetaKeys = discoverKeys(fillers);

    // Format an arbitrary metadata value (string/number/bool/array/object).
    function metaVal(v) {
        if (v == null) return "";
        if (typeof v === "object") { try { return JSON.stringify(v); } catch { return String(v); } }
        return String(v);
    }
    function metaGet(entry, key) {
        return entry && entry.metadata ? entry.metadata[key] : undefined;
    }
    // Extent [min, max] of a run-length-encoded id list, or null if empty.
    function idExtent(runs) {
        if (!runs || !runs.length) return null;
        return [runs[0][0], runs[runs.length - 1][1]];
    }
    // A "description"-like key, if the project uses one, for opportunistic display.
    function descOf(entry) {
        const m = entry && entry.metadata; if (!m) return "";
        const k = ["description", "desc", "title", "name", "label"].find((x) => { return m[x] != null; });
        return k ? metaVal(m[k]) : "";
    }
    // Pick sensible default grouping fields (prefer conventional names if present).
    function defaultFields(keys) {
        return keys.map((key) => {
            const values = envelopes.map((entry) => metaGet(entry, key)).filter((value) => value != null && metaVal(value) !== "");
            const count = new Set(values.map(metaVal)).size;
            return { key, count, present: values.length };
        }).filter((field) => field.count > 1 && field.count < field.present && field.count <= 32)
            .sort((left, right) => right.present - left.present || left.count - right.count || left.key.localeCompare(right.key))
            .slice(0, 2).map((field) => field.key);
    }

    // ── DOM helpers ────────────────────────────────────────────────────────
    function el(tag, props, kids) {
        const n = document.createElement(tag);
        if (props) {
            for (const k in props) {
                if (k === "class") n.className = props[k];
                else if (k === "text") n.textContent = props[k];
                else if (k === "html") n.innerHTML = props[k];
                else if (k.slice(0, 2) === "on") n.addEventListener(k.slice(2), props[k]);
                else if (k === "style") n.setAttribute("style", props[k]);
                else if (props[k] != null) n.setAttribute(k, props[k]);
            }
        }
        if (kids != null) append(n, kids);
        return n;
    }
    function append(n, kids) {
        if (Array.isArray(kids)) kids.forEach((c) => { append(n, c); });
        else if (kids instanceof Node) n.appendChild(kids);
        else if (kids != null) n.appendChild(document.createTextNode(String(kids)));
    }
    function svgEl(tag, props) {
        const n = document.createElementNS("http://www.w3.org/2000/svg", tag);
        if (props) for (const k in props) if (props[k] != null) n.setAttribute(k, props[k]);
        return n;
    }
    function clear(n) { while (n.firstChild) n.removeChild(n.firstChild); }
    function actionable(node, handler) {
        node.setAttribute("tabindex", "0");
        node.setAttribute("role", "button");
        node.addEventListener("click", handler);
        node.addEventListener("keydown", (event) => {
            if (event.key === "Enter" || event.key === " ") { event.preventDefault(); handler(); }
        });
        return node;
    }
    function tableOf(headers, rows) {
        return el("div", { class: "tablewrap" }, el("table", { class: "data" }, [
            el("thead", {}, el("tr", {}, headers.map((heading) => el("th", { scope: "col", text: heading })))),
            el("tbody", {}, rows.map((row) => el("tr", {}, row.map((value) => el("td", {}, value)))))
        ]));
    }
    function detailsBlock(title, content) {
        const details = el("details", { class: "detail-block" }, el("summary", { text: title }));
        if (typeof content === "function") {
            let rendered = false;
            details.addEventListener("toggle", () => {
                if (details.open && !rendered) { rendered = true; details.appendChild(content()); }
            });
        } else append(details, content);
        return details;
    }
    function inputFor(name, role = "filler") { return inputs.find((input) => input.name === name && input.role === role); }
    function fmt(x) { return (x == null ? "—" : Number(x).toLocaleString("en-US")); }
    const $ = (s, r) => { return (r || document).querySelector(s); };

    // Deterministic color from a universe id.
    function uHue(id) {
        let h = 0;
        const s = String(id);
        for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) >>> 0;
        return h % 360;
    }
    function uColor(id) {
        if (id == null) return "var(--null)";
        const dark = document.documentElement.getAttribute("data-theme") === "dark";
        return `hsl(${uHue(id)} ${dark ? 55 : 62}% ${dark ? 55 : 52}%)`;
    }

    // ── Icons (inline SVG paths) ───────────────────────────────────────────
    function icon(name) {
        const p = {
            search: "M11 4a7 7 0 1 0 4.9 12l4 4 1.4-1.4-4-4A7 7 0 0 0 11 4Zm0 2a5 5 0 1 1 0 10 5 5 0 0 1 0-10Z",
            sun: "M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10Zm0-5v3m0 14v3M4 12H1m22 0h-3M5 5 3 3m18 2 2-2M5 19l-2 2m18-2 2 2",
            moon: "M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8Z",
            download: "M12 3v12m0 0 4-4m-4 4-4-4M4 17v3a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-3",
            grid: "M4 4h7v7H4V4Zm9 0h7v7h-7V4ZM4 13h7v7H4v-7Zm9 0h7v7h-7v-7Z",
            tree: "M6 4h4v3H6V4Zm8 0h4v3h-4V4ZM6 17h4v3H6v-3Zm8 0h4v3h-4v-3ZM8 7v4m8-4v4M8 11h8v3",
            map: "M9 3 3 5v16l6-2 6 2 6-2V3l-6 2-6-2Zm0 0v16m6-14v16",
            layers: "M12 3 2 8l10 5 10-5-10-5Zm-10 9 10 5 10-5M2 16l10 5 10-5",
            box: "M12 3 3 7.5V16l9 5 9-5V7.5L12 3Zm0 0v18M3 7.5l9 4.5 9-4.5",
            diff: "M12 3v18M5 8l-3 4 3 4m14-8 3 4-3 4"
        }[name];
        const s = svgEl("svg", { viewBox: "0 0 24 24", width: 16, height: 16, fill: "none", stroke: "currentColor", "stroke-width": 2, "stroke-linecap": "round", "stroke-linejoin": "round" });
        s.appendChild(svgEl("path", { d: p }));
        return s;
    }

    // ── Tooltip ────────────────────────────────────────────────────────────
    const tip = el("div", { id: "tooltip" });
    document.body.appendChild(tip);
    function showTip(html, x, y) {
        tip.innerHTML = html;
        tip.classList.add("show");
        const w = tip.offsetWidth, h = tip.offsetHeight;
        tip.style.left = `${Math.min(x + 14, window.innerWidth - w - 8)}px`;
        tip.style.top = `${Math.max(8, y - h - 12)}px`;
    }
    function hideTip() { tip.classList.remove("show"); }

    // ── Derived stats ──────────────────────────────────────────────────────
    const nFilled = envelopes.filter((e) => { return e.filler_name; }).length;
    const nNull = envelopes.filter((entry) => statusOf(entry) === "empty").length;
    const nUnconfigured = envelopes.filter((entry) => statusOf(entry) === "unconfigured").length;
    const completeness = envelopes.length ? Math.round((envelopes.length - nUnconfigured) / envelopes.length * 100) : 100;
    const coverage = envelopes.length ? Math.round((nFilled / envelopes.length) * 100) : 0;
    const nData = dataFiles.length;

    // ── App shell ──────────────────────────────────────────────────────────
    const app = el("div");
    document.body.appendChild(app);

    // ── Tabs (defined before the header, which mounts them) ────────────────
    const TABS = [
        { id: "overview", label: "Overview", icon: "layers" },
        { id: "checks", label: "Checks", icon: "layers", pill: (DATA.warnings || []).length },
        { id: "explorer", label: "Explorer", icon: "tree", pill: envelopes.length },
        { id: "coverage", label: "Coverage Map", icon: "grid" },
        { id: "idmap", label: "ID Map", icon: "map" },
        { id: "fillers", label: "Fillers", icon: "box", pill: fillers.length },
        { id: "data", label: "Data Cards", icon: "layers", pill: nData },
        { id: "inputs", label: "Inputs", icon: "layers", pill: inputs.length },
        { id: "diff", label: "Diff", icon: "diff" }
    ];
    const tabBtns = {};
    function buildTabs() {
        const nav = el("nav", { class: "tabs", "aria-label": "Report views" });
        TABS.forEach((t) => {
            const b = el("button", { class: "tab", onclick: () => { go(t.id); } }, [
                icon(t.icon), document.createTextNode(t.label)
            ]);
            if (t.pill != null) b.appendChild(el("span", { class: "pill", text: String(t.pill) }));
            tabBtns[t.id] = b;
            b.setAttribute("aria-controls", `panel-${t.id}`);
            nav.appendChild(b);
        });
        return nav;
    }

    // Header
    const themeBtn = el("button", { class: "iconbtn", title: "Toggle theme", "aria-label": "Toggle theme", onclick: toggleTheme });
    const exportBtn = el("button", { class: "iconbtn", title: "Download report data (JSON)", "aria-label": "Download JSON", onclick: downloadJson });
    exportBtn.appendChild(icon("download"));
    function paintThemeIcon() {
        clear(themeBtn);
        themeBtn.appendChild(icon(document.documentElement.getAttribute("data-theme") === "dark" ? "sun" : "moon"));
    }

    const header = el("header", { class: "topbar" }, el("div", { class: "wrap" }, [
        el("div", { class: "topbar-inner" }, [
            el("div", { class: "brand" }, [
                el("span", { class: "logo", text: "◆" }),
                el("span", { text: "gitronics" }),
                el("span", { class: "ver", text: `v${DATA.gitronics_version}` })
            ]),
            el("span", { class: "badge-report", text: "Build Report" }),
            el("div", { class: "topbar-spacer" }),
            el("div", { class: "provenance" }, [
                el("span", {}, [el("b", { text: "config " }), document.createTextNode(DATA.config_path)]),
                el("span", {}, [el("b", { text: "commit " }), document.createTextNode(DATA.commit_hash || "—")]),
                el("span", {}, [el("b", { text: "built " }), document.createTextNode(DATA.date_time)])
            ]),
            exportBtn, themeBtn
        ]),
        buildTabs()
    ]));
    app.appendChild(header);
    paintThemeIcon();

    const main = el("main", {}, el("div", { class: "wrap", id: "panels" }));
    app.appendChild(main);
    const panels = $("#panels", main);

    app.appendChild(el("footer", { class: "foot" }, [
        document.createTextNode("Generated by "),
        el("a", { href: "https://fusion4energy.github.io/gitronics/latest", text: "gitronics" }),
        document.createTextNode(` · schema v${DATA.schema_version || 1}`)
    ]));

    // ── Tabs (mounting handled by buildTabs, defined above) ────────────────
    const built = {};
    const panelEls = {};
    function go(id) {
        if (!tabBtns[id]) id = "overview";
        for (const k in tabBtns) {
            tabBtns[k].classList.toggle("active", k === id);
            tabBtns[k].setAttribute("aria-current", k === id ? "page" : "false");
        }
        for (const p in panelEls) panelEls[p].classList.toggle("active", p === id);
        if (!panelEls[id]) {
            const pane = el("section", { class: "panel", id: `panel-${id}` });
            panels.appendChild(pane);
            panelEls[id] = pane;
        }
        if (!built[id]) { RENDER[id](panelEls[id]); built[id] = true; }
        panelEls[id].classList.add("active");
        setHash({ tab: id });
        window.scrollTo({ top: 0, behavior: "instant" in window ? "instant" : "auto" });
    }

    // ── Hash / deep-linking ─────────────────────────────────────────────────
    function getHash() {
        const o = {};
        location.hash.replace(/^#/, "").split("&").forEach((kv) => {
            const i = kv.indexOf("="); if (i > 0) o[kv.slice(0, i)] = decodeURIComponent(kv.slice(i + 1));
        });
        return o;
    }
    function setHash(patch) {
        const o = getHash();
        for (const k in patch) { if (patch[k] == null) delete o[k]; else o[k] = patch[k]; }
        const s = Object.keys(o).map((k) => { return `${k}=${encodeURIComponent(o[k])}`; }).join("&");
        history.replaceState(null, "", `#${s}`);
    }

    // ── Theme ────────────────────────────────────────────────────────────────
    function toggleTheme() {
        const d = document.documentElement.getAttribute("data-theme") === "dark";
        const next = d ? "light" : "dark";
        document.documentElement.setAttribute("data-theme", next);
        try { localStorage.setItem("gitronics-theme", next); } catch { }
        paintThemeIcon();
        // re-render open dynamic panels that depend on theme colors
        ["coverage", "idmap", "overview"].forEach((id) => {
            if (built[id]) { clear(panelEls[id]); RENDER[id](panelEls[id]); }
        });
    }

    function downloadJson() {
        const blob = new Blob([JSON.stringify(DATA, null, 2)], { type: "application/json" });
        const a = el("a", { href: URL.createObjectURL(blob), download: "build_report.json" });
        document.body.appendChild(a); a.click(); a.remove();
        URL.revokeObjectURL(a.href);
    }

    // ── Search box widget ────────────────────────────────────────────────────
    function searchBox(placeholder, oninput) {
        const box = el("div", { class: "searchbox" });
        box.appendChild(icon("search"));
        const inp = el("input", { class: "input", type: "search", "aria-label": placeholder, placeholder: placeholder, oninput: (e) => { oninput(e.currentTarget.value.toLowerCase().trim()); } });
        box.appendChild(inp);
        return box;
    }

    // ── KPI cards ────────────────────────────────────────────────────────────
    function kpi(v, label, sub, spark) {
        const k = el("div", { class: "kpi" }, [
            el("div", { class: "v", text: fmt(v) }),
            el("div", { class: "l", text: label })
        ]);
        if (sub) k.appendChild(el("div", { class: "s", text: sub }));
        if (spark) k.appendChild(spark);
        return k;
    }

    // ══════════════════════════════════════════════════════════════════════
    //  RENDERERS
    // ══════════════════════════════════════════════════════════════════════
    const RENDER = {};

    // ── Overview ─────────────────────────────────────────────────────────────
    RENDER.overview = (root) => {
        clear(root);
        const wrap = el("div", { class: "section-gap" });
        root.appendChild(wrap);

        wrap.appendChild(el("div", { class: "report-heading" }, [
            el("h1", { text: (DATA.config_path || "Build").split(/[\\/]/).pop() }),
            el("span", { class: `chip ${DATA.build_status === "failed" ? "danger" : "accent"}`, text: DATA.build_status === "success" ? "Assembly completed" : DATA.build_status === "failed" ? "Build failed" : "Build status not recorded" })
        ]));
        const attention = el("div", { class: "attention" }, [
            el("button", { class: "btn", text: `${(DATA.warnings || []).length} warnings`, onclick: () => go("checks") }),
            el("button", { class: "btn", text: `${nUnconfigured} unconfigured envelopes`, onclick: () => { go("explorer"); const select = $("#panel-explorer .status-filter"); select.value = "unconfigured"; select.dispatchEvent(new window.Event("change")); } }),
            el("span", { class: "muted", text: "Geometry and transport physics: not validated by this build" })
        ]);
        wrap.appendChild(attention);

        // KPIs
        const kpis = el("div", { class: "grid kpis" }, [
            kpi(DATA.total_cells, "Cells"),
            kpi(DATA.total_surfaces, "Surfaces"),
            kpi(envelopes.length, "Envelopes", `${nFilled} filled · ${nNull} explicitly empty`),
            kpi(fillers.length, "Filler Models"),
            kpi(completeness, "Configuration completeness", `${nUnconfigured} not configured`),
            kpi(nData, "Data files")
        ]);
        // coverage % value shows literal string; fix formatting
        kpis.children[4].querySelector(".v").textContent = `${completeness}%`;
        wrap.appendChild(kpis);

        const cols = el("div", { class: "grid", style: "grid-template-columns:repeat(auto-fit,minmax(320px,1fr))" });
        wrap.appendChild(cols);

        // Top fillers by cells
        const topCells = fillers.slice().sort((a, b) => { return b.cell_count - a.cell_count; }).slice(0, 10);
        cols.appendChild(card("Top fillers by cell count", barChart(topCells.map((f) => {
            return { label: f.name, value: f.cell_count, id: f.universe_id, onclick: () => { openFiller(f.name); } };
        }))));

        // Most reused fillers
        const reused = fillers.slice().filter((f) => { return f.envelope_count > 0; })
            .sort((a, b) => { return b.envelope_count - a.envelope_count; }).slice(0, 10);
        cols.appendChild(card("Most reused fillers (envelopes filled)", barChart(reused.map((f) => {
            return { label: f.name, value: f.envelope_count, id: f.universe_id, onclick: () => { openFiller(f.name); } };
        }))));

        // Materials usage
        const matCount = {};
        fillers.forEach((f) => { (f.materials || []).forEach((m) => { matCount[m] = (matCount[m] || 0) + 1; }); });
        const mats = Object.keys(matCount).map((m) => { return { label: `mat ${m}`, value: matCount[m] }; })
            .sort((a, b) => { return b.value - a.value; }).slice(0, 12);
        if (mats.length) cols.appendChild(card("Material usage (fillers per material)", barChart(mats)));

        // Breakdown by a chosen metadata field (fully generic).
        if (defaultFields(envMetaKeys).length) {
            const field = defaultFields(envMetaKeys)[0];
            const breakdownCard = card(`Envelopes by ${field}`, el("div"));
            const head = breakdownCard.querySelector(".card-head");
            const sel = el("select", { class: "input", style: "width:auto;margin-left:auto" });
            envMetaKeys.forEach((k) => { sel.appendChild(el("option", { value: k, text: k })); });
            sel.value = field;
            head.appendChild(sel);
            const bd = breakdownCard.querySelector(".card-body");
            function paintBreakdown() {
                clear(bd);
                bd.appendChild(barChart(rollupBy(sel.value).map((g) => {
                    return { label: g.key, value: g.total, sub: `${g.filled}/${g.total} filled` };
                })));
            }
            sel.addEventListener("change", paintBreakdown);
            paintBreakdown();
            cols.appendChild(breakdownCard);
        }
    };

    function card(title, body) {
        return el("div", { class: "card" }, [
            el("div", { class: "card-head" }, el("h3", { text: title })),
            el("div", { class: "card-body" }, body)
        ]);
    }

    function legRow(color, label, val) {
        return el("div", { class: "legend", style: "margin:4px 0" }, el("div", { class: "item" }, [
            el("span", { class: "dot-swatch", style: `background:${color}` }),
            el("span", { text: `${label} · ${fmt(val)}` })
        ]));
    }

    function barChart(rows) {
        const max = rows.reduce((m, r) => { return Math.max(m, r.value); }, 0) || 1;
        const box = el("div", {});
        if (!rows.length) { box.appendChild(el("div", { class: "empty", text: "No data" })); return box; }
        rows.forEach((r) => {
            const fill = el("div", { class: "fill", style: `width:${r.value / max * 100}%${r.id != null ? `;background:${uColor(r.id)}` : ""}` });
            const row = el("div", { class: "chart-bar-row" }, [
                el("div", { class: "lab", title: r.label, text: r.label }),
                el("div", { class: "track" }, fill),
                el("div", { class: "val", text: fmt(r.value) + (r.sub ? "" : "") })
            ]);
            if (r.onclick) { row.style.cursor = "pointer"; actionable(row, r.onclick); }
            if (r.sub) row.querySelector(".lab").title = r.sub;
            box.appendChild(row);
        });
        return box;
    }

    // Group envelopes by an arbitrary field key (metadata key, or the special
    // "__filler__"/"__universe__"). Returns [{key, total, filled}] sorted by size.
    function groupKeyOf(e, field) {
        if (field === "__filler__") return e.filler_name || "(null)";
        if (field === "__universe__") return e.universe_id != null ? `u${e.universe_id}` : "(null)";
        const v = metaGet(e, field);
        return v == null || v === "" ? "—" : metaVal(v);
    }
    function rollupBy(field) {
        const m = Object.create(null);
        envelopes.forEach((e) => {
            const k = groupKeyOf(e, field);
            if (!m[k]) m[k] = { key: k, total: 0, filled: 0 };
            m[k].total++; if (e.filler_name) m[k].filled++;
        });
        return Object.keys(m).map((k) => { return m[k]; }).sort((a, b) => { return b.total - a.total; });
    }

    // Options for a "group by" <select>: metadata keys + filler/universe.
    function groupFieldOptions() {
        const opts = [{ v: "__none__", t: "(no grouping)" }];
        envMetaKeys.forEach((k) => { opts.push({ v: k, t: k }); });
        opts.push({ v: "__filler__", t: "filler" });
        opts.push({ v: "__universe__", t: "universe" });
        return opts;
    }
    function groupSelect(value, onchange) {
        const sel = el("select", { class: "input", style: "width:auto", "aria-label": "Group by metadata" });
        groupFieldOptions().forEach((o) => { sel.appendChild(el("option", { value: o.v, text: o.t })); });
        sel.value = value;
        sel.addEventListener("change", (e) => { onchange(e.currentTarget.value); });
        return sel;
    }

    // ── Explorer (tree) ──────────────────────────────────────────────────────
    RENDER.explorer = (root) => {
        clear(root);
        const treeHost = el("div", { class: "tree" });
        const note = el("div", { class: "count-note" });
        const defaults = defaultFields(envMetaKeys);
        let g1 = getHash().group || defaults[0] || "__none__";
        let g2 = defaults[1] || "__none__";
        let curQ = getHash().q || "";

        const sel1 = groupSelect(g1, (v) => { g1 = v; setHash({ group: v }); rebuild(curQ); });
        const sel2 = groupSelect(g2, (v) => { g2 = v; rebuild(curQ); });
        const statusFilter = el("select", { class: "input status-filter", "aria-label": "Assignment status", onchange: () => rebuild(curQ) },
            [el("option", { value: "all", text: "All statuses" }), ...["filled", "empty", "unconfigured"].map((status) => el("option", { value: status, text: statusLabel(status) }))]);
        const body = el("div", { class: "card-body section-gap" }, [
            el("div", { class: "toolbar" }, [
                searchBox("Filter by envelope, filler, metadata, universe…", (q) => { curQ = q; setHash({ q: q || null }); rebuild(q); }), statusFilter,
                el("span", { class: "muted", style: "font-size:.8rem", text: "Group by" }), sel1,
                el("span", { class: "muted", style: "font-size:.8rem", text: "then" }), sel2,
                note
            ]),
            treeHost
        ]);
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Envelope assignments" }),
            el("span", { class: "spring" }),
            el("span", { class: "chip ok", text: `${nFilled} filled` }),
            el("span", { class: "chip null", text: `${nNull} explicitly empty` }),
            el("span", { class: "chip", text: `${nUnconfigured} not configured` })]),
            body
        ]));

        function buildGroups(list, field) {
            const groups = Object.create(null), order = [];
            list.forEach((e) => {
                const k = groupKeyOf(e, field);
                if (!groups[k]) { groups[k] = []; order.push(k); }
                groups[k].push(e);
            });
            order.sort();
            return { groups: groups, order: order };
        }
        function groupDetails(name, count, open) {
            const det = el("details", open ? { open: "" } : {});
            det.appendChild(el("summary", {}, [
                el("span", { class: "tw", text: "▶" }),
                el("span", { class: "grp-name", text: name }),
                el("span", { class: "spring" }),
                el("span", { class: "chip", text: count })
            ]));
            return det;
        }

        function rebuild(q) {
            clear(treeHost);
            q = (q || "").toLowerCase();
            const shown = envelopes.filter((e) => (statusFilter.value === "all" || statusOf(e) === statusFilter.value) && (!q || matchEnv(e).indexOf(q) >= 0));
            note.textContent = `${shown.length} / ${envelopes.length} envelopes`;
            if (!shown.length) { treeHost.appendChild(el("div", { class: "empty", text: "No matches" })); return; }

            if (g1 === "__none__") { shown.forEach((e) => { treeHost.appendChild(leafRow(e)); }); return; }

            const lvl1 = buildGroups(shown, g1);
            const autoOpen = shown.length <= 50 || !!q;
            lvl1.order.forEach((k1) => {
                const list1 = lvl1.groups[k1];
                const d1 = groupDetails(k1, list1.length, autoOpen);
                if (g2 === "__none__" || g2 === g1) {
                    list1.forEach((e) => { d1.appendChild(leafRow(e)); });
                } else {
                    const lvl2 = buildGroups(list1, g2);
                    lvl2.order.forEach((k2) => {
                        const list2 = lvl2.groups[k2];
                        const d2 = groupDetails(k2, list2.length, autoOpen || lvl2.order.length === 1);
                        d2.querySelector(".grp-name").classList.add("muted");
                        list2.forEach((e) => { d2.appendChild(leafRow(e)); });
                        d1.appendChild(d2);
                    });
                }
                treeHost.appendChild(d1);
            });
        }
        rebuild(curQ);
        $("input", root).value = curQ;
    };

    function matchEnv(e) {
        const parts = [e.envelope_name, e.filler_name, e.universe_id, e.transform, ...(e.cell_ids || [])];
        if (e.metadata) Object.keys(e.metadata).forEach((k) => { parts.push(metaVal(e.metadata[k])); });
        return parts.filter((x) => { return x != null && x !== ""; }).join(" ").toLowerCase();
    }

    function leafRow(e) {
        const filled = !!e.filler_name;
        const leaf = el("div", { class: "leaf" });
        leaf.appendChild(el("span", { class: "dot-swatch", style: `background:${filled ? uColor(e.universe_id) : "var(--null)"}` }));
        leaf.appendChild(el("span", { class: "en", text: e.envelope_name }));
        if (filled) {
            leaf.appendChild(el("span", { class: "fl", text: `→ ${e.filler_name}` }));
            leaf.appendChild(el("span", { class: "chip accent", text: `u${e.universe_id}` }));
            if (e.transform) leaf.appendChild(el("span", { class: "chip", text: e.transform }));
        } else {
            leaf.appendChild(el("span", { class: "chip null", text: statusLabel(statusOf(e)) }));
        }
        leaf.appendChild(el("span", { class: "spring" }));
        const d = descOf(e);
        if (d) leaf.appendChild(el("span", { class: "desc", text: d }));
        actionable(leaf, () => openEnvelope(e.envelope_name));
        return leaf;
    }

    // ── Coverage map ─────────────────────────────────────────────────────────
    RENDER.coverage = (root) => {
        clear(root);
        const host = el("div", {});
        let query = "";
        let g = defaultFields(envMetaKeys)[0] || "__none__";
        const sel = groupSelect(g, (v) => { g = v; rebuild(); });
        const statusSelect = el("select", { class: "input", "aria-label": "Map assignment status", onchange: rebuild },
            [el("option", { value: "all", text: "All statuses" }), ...["filled", "empty", "unconfigured"].map((status) => el("option", { value: status, text: statusLabel(status) }))]);
        const colorSelect = el("select", { class: "input", "aria-label": "Color by", onchange: rebuild }, [
            el("option", { value: "status", text: "Color by status" }), el("option", { value: "universe", text: "Color by universe" })]);
        const body = el("div", { class: "card-body section-gap" }, [
            el("div", { class: "toolbar" }, [
                el("span", { class: "muted", style: "font-size:.8rem", text: "Group by" }), sel,
                statusSelect, colorSelect, searchBox("Filter map envelopes", (value) => { query = value; rebuild(); })
            ]),
            el("div", { class: "legend" }, [legRow("var(--ok)", "Filled", nFilled), legRow("var(--null)", "Explicitly empty", nNull), legRow("var(--warn)", "Not configured", nUnconfigured)]),
            host
        ]);
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Coverage map" }),
            el("span", { class: "spring" }),
            el("span", { class: "chip accent", text: `${coverage}% filled` })]),
            body
        ]));

        function rebuild() {
            clear(host);
            const groups = Object.create(null), order = [];
            envelopes.forEach((e) => {
                if ((statusSelect.value !== "all" && statusOf(e) !== statusSelect.value) || (query && !matchEnv(e).includes(query))) return;
                const k = g === "__none__" ? "All envelopes" : groupKeyOf(e, g);
                if (!groups[k]) { groups[k] = []; order.push(k); }
                groups[k].push(e);
            });
            order.sort();
            order.forEach((k) => {
                const list = groups[k].slice().sort((a, b) => {
                    return a.envelope_name < b.envelope_name ? -1 : a.envelope_name > b.envelope_name ? 1 : 0;
                });
                const filled = list.filter((e) => { return e.filler_name; }).length;
                const zwrap = el("div", { class: "cov-zone" });
                zwrap.appendChild(el("h4", {}, [
                    el("span", { text: k }),
                    el("span", { class: "chip", text: `${filled}/${list.length}` })
                ]));
                const grid = el("div", { class: "cov-grid" });
                list.forEach((e) => {
                    const filledCell = !!e.filler_name;
                    const stateColor = { filled: "var(--ok)", empty: "var(--null)", unconfigured: "var(--warn)" }[statusOf(e)];
                    const cell = el("button", {
                        class: "cov-cell",
                        "aria-label": `${e.envelope_name}: ${statusLabel(statusOf(e))}`,
                        title: `${e.envelope_name}: ${statusLabel(statusOf(e))}`,
                        style: `background:${colorSelect.value === "universe" && filledCell ? uColor(e.universe_id) : stateColor}`,
                        onclick: () => { hideTip(); openEnvelope(e.envelope_name); }
                    });
                    cell.addEventListener("mousemove", (ev) => { showTip(envTip(e), ev.clientX, ev.clientY); });
                    cell.addEventListener("mouseleave", hideTip);
                    grid.appendChild(cell);
                });
                zwrap.appendChild(grid);
                host.appendChild(zwrap);
            });
            if (!order.length) host.appendChild(el("p", { class: "empty", text: "No matching envelopes." }));
        }
        rebuild();
    };

    // Tooltip HTML for an envelope: name, description, up to 4 metadata fields, filler.
    function envTip(e) {
        let s = `<b>${esc(e.envelope_name)}</b>`;
        const d = descOf(e); if (d) s += `<br>${esc(d)}`;
        if (e.metadata) {
            Object.keys(e.metadata).slice(0, 5).forEach((k) => {
                if (["description", "desc", "title"].indexOf(k) >= 0) return;
                s += `<br><span style='opacity:.7'>${esc(k)}:</span> ${esc(metaVal(e.metadata[k]))}`;
            });
        }
        s += `<br>${e.filler_name ? `→ ${esc(e.filler_name)} (u${e.universe_id})` : statusLabel(statusOf(e))}`;
        return s;
    }

    function esc(s) { return String(s).replace(/[&<>"]/g, (c) => { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]; }); }

    // ── ID memory map (zoomable, exact ids) ──────────────────────────────────
    RENDER.idmap = (root) => {
        clear(root);
        const body = el("div", { class: "card-body section-gap" });
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Card-ID memory map" }),
            el("span", { class: "spring" })]),
            body
        ]));
        body.appendChild(zoomLane("Cell ids", geometrySegments("cell_id_runs")));
        body.appendChild(zoomLane("Surface ids", geometrySegments("surface_id_runs")));
        const inventoryMissing = !Array.isArray(evidence.data_cards);
        for (const [title, pattern] of [["Material ids", /^M(\d+)(?::.*)?$/i], ["Tally ids", /^\*?(?:F|FMESH)(\d+)(?::.*)?$/i], ["Transformation ids", /^\*?TR(\d+)(?::.*)?$/i]]) {
            body.appendChild(zoomLane(title, dataSegments(pattern), inventoryMissing ? "Card inventory was not recorded in this report." : "No IDs defined."));
        }
        const universeMissing = [...fillers, ...(envelopeStructure ? [envelopeStructure] : [])].some((entry) => !Array.isArray(entry.universe_id_runs));
        body.appendChild(zoomLane("Universe ids", universeMissing ? [] : geometrySegments("universe_id_runs"), universeMissing ? "Universe membership was not recorded in this report." : "No IDs defined."));
    };

    function geometrySegments(key) {
        return [...fillers, ...(envelopeStructure ? [envelopeStructure] : [])].flatMap((entry) => {
            const isEnv = entry === envelopeStructure;
            const owner = {
                name: entry.name, color: isEnv ? null : uColor(entry.universe_id),
                inspect: isEnv ? null : () => openFiller(entry.name)
            };
            return (entry[key] || []).map(([start, end]) => ({ s: start, e: end, owner }));
        });
    }

    function dataSegments(pattern) {
        const owners = new Map();
        const segments = [];
        for (const card of dataCards) {
            const match = pattern.exec(card.name);
            if (!match) continue;
            const id = Number(match[1]);
            if (!Number.isSafeInteger(id) || id <= 0) continue;
            if (!owners.has(card.input)) {
                const input = inputs.find((entry) => entry.path === card.input);
                const cards = new Map();
                owners.set(card.input, {
                    name: input ? input.name : card.input, path: card.input,
                    color: uColor(card.input), cards,
                    inspect: (selected) => openMappedCards(card.input, cards.get(selected) || [])
                });
            }
            const owner = owners.get(card.input);
            if (!owner.cards.has(id)) owner.cards.set(id, []);
            owner.cards.get(id).push(card);
            segments.push({ s: id, e: id, owner });
        }
        return segments;
    }

    function openMappedCards(path, cards) {
        clear(drawer);
        drawer.appendChild(el("div", { class: "drawer-head" }, [el("h3", { class: "spring", text: cards.map((card) => card.name).join(", ") }),
        el("button", { class: "iconbtn", "aria-label": "Close details", text: "\u00d7", onclick: closeDrawer })]));
        drawer.appendChild(el("div", { class: "drawer-body card-details" }, [el("div", { class: "muted mono", text: path }),
        ...cards.map((card) => el("pre", { text: card.text }))]));
        revealDrawer({ envelope: null, filler: null });
    }

    function ownedSegments(ranges) {
        const events = new Map();
        for (const range of ranges) {
            for (const [position, delta] of [[range.s, 1], [range.e + 1, -1]]) {
                if (!events.has(position)) events.set(position, []);
                events.get(position).push({ owner: range.owner, delta });
            }
        }
        const active = new Map(), segments = [];
        let previous = null;
        for (const [position, changes] of [...events].sort(([left], [right]) => left - right)) {
            if (previous !== null && previous < position && active.size) segments.push({ s: previous, e: position - 1, owners: [...active.keys()] });
            for (const { owner, delta } of changes) {
                const count = (active.get(owner) || 0) + delta;
                if (count) active.set(owner, count); else active.delete(owner);
            }
            previous = position;
        }
        return segments;
    }

    function zoomLane(title, ranges, emptyText = "No id data available.") {
        const segs = ownedSegments(ranges);
        const wrap = el("div", { class: "idmap-lane", "data-id-kind": title });
        if (!segs.length) {
            wrap.appendChild(el("div", { class: "lane-title", text: title }));
            wrap.appendChild(el("div", { class: "empty", text: emptyText }));
            return wrap;
        }

        const gmin = segs[0].s;
        const gmax = segs.reduce((m, x) => { return Math.max(m, x.e); }, segs[0].e);
        const used = segs.reduce((a, x) => { return a + (x.e - x.s + 1); }, 0);
        const domSpan = gmax - gmin + 1;
        const minSpan = Math.min(16, domSpan);

        let vs = gmin, ve = gmax + 1; // [vs, ve): current view (float, exclusive end)
        let hoverId = null, dragging = false, moved = false, lastX = 0;

        const canvas = el("canvas", { class: "idmap-canvas" });
        canvas.setAttribute("aria-label", `${title} occupancy map`);
        const readout = el("span", { class: "count-note mono" });
        const lookup = el("input", { class: "input", type: "number", min: gmin, max: gmax, step: 1, "aria-label": `Find ${title.toLowerCase()}`, placeholder: "Card ID" });
        const lookupResult = el("div", { class: "id-lookup-result", "aria-live": "polite" });
        function showLookup(id) {
            clear(lookupResult);
            const segment = segAt(id);
            if (!segment) { lookupResult.textContent = `${id}: unused`; return; }
            lookupResult.appendChild(el("span", { text: `${id}: ${segment.owners.map((owner) => owner.name).join(", ")}` }));
            for (const owner of segment.owners) {
                if (owner.inspect) lookupResult.appendChild(el("button", { class: "btn", type: "button", text: segment.owners.length > 1 ? `Inspect ${owner.name}` : "Inspect owner", onclick: () => owner.inspect(id) }));
            }
        }
        const lookupForm = el("form", {
            class: "toolbar", onsubmit: (event) => {
                event.preventDefault(); clear(lookupResult);
                const id = Number(lookup.value);
                if (!lookup.value || !Number.isSafeInteger(id) || id < gmin || id > gmax) {
                    lookupResult.textContent = `Enter an integer from ${gmin} to ${gmax}.`; return;
                }
                vs = id - 8; ve = id + 9; clampView(); schedule();
                showLookup(id);
            }
        }, [lookup, el("button", { class: "iconbtn", type: "submit", title: "Find card ID", "aria-label": "Find card ID" }, icon("search"))]);
        const owners = new Set(ranges.map((range) => range.owner));
        const laneTitle = `${title} · ${owners.size} owner${owners.size === 1 ? "" : "s"}`;
        const controls = el("div", { class: "idmap-controls" }, [
            el("div", { class: "lane-title", text: laneTitle }),
            el("span", { class: "spring" }),
            el("button", { class: "btn ghost", title: "Zoom out", text: "−", onclick: () => { zoomAt(0.5, 1.6); } }),
            el("button", { class: "btn ghost", title: "Zoom in", text: "+", onclick: () => { zoomAt(0.5, 0.625); } }),
            el("button", { class: "btn ghost", text: "Reset", onclick: () => { vs = gmin; ve = gmax + 1; schedule(); } }),
            readout
        ]);
        wrap.appendChild(controls);
        wrap.appendChild(lookupForm);
        wrap.appendChild(lookupResult);
        wrap.appendChild(canvas);

        function clampView() {
            const span = Math.min(Math.max(ve - vs, minSpan), domSpan);
            if (vs < gmin) vs = gmin;
            ve = vs + span;
            if (ve > gmax + 1) { ve = gmax + 1; vs = ve - span; if (vs < gmin) vs = gmin; }
        }
        function zoomAt(frac, factor) {
            const idc = vs + frac * (ve - vs);
            const span = Math.min(Math.max((ve - vs) * factor, minSpan), domSpan);
            vs = idc - frac * span; ve = vs + span; clampView(); schedule();
        }
        function idAtPx(px, W) { return vs + (px / W) * (ve - vs); }
        function segAt(id) {
            let lo = 0, hi = segs.length - 1, res = -1;
            while (lo <= hi) { const m = (lo + hi) >> 1; if (segs[m].s <= id) { res = m; lo = m + 1; } else hi = m - 1; }
            return (res >= 0 && id <= segs[res].e) ? segs[res] : null;
        }

        let raf = 0;
        function schedule() { if (!raf) raf = requestAnimationFrame(() => { raf = 0; draw(); }); }

        function draw() {
            const dpr = window.devicePixelRatio || 1;
            const W = canvas.clientWidth || 600, Hpx = canvas.clientHeight || 90;
            canvas.width = Math.round(W * dpr); canvas.height = Math.round(Hpx * dpr);
            const ctx = canvas.getContext("2d");
            ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
            ctx.clearRect(0, 0, W, Hpx);
            const cs = getComputedStyle(document.documentElement);
            const cBorder = cs.getPropertyValue("--border").trim() || "#ddd";
            const cText3 = cs.getPropertyValue("--text-3").trim() || "#888";
            const cSurface = cs.getPropertyValue("--surface").trim() || "#fff";
            const cTrack = cs.getPropertyValue("--surface-3").trim() || "#eee";

            const laneTop = 8, laneH = Hpx - 34;
            function X(id) { return (id - vs) / (ve - vs) * W; }

            ctx.fillStyle = cTrack;
            ctx.fillRect(0, laneTop, W, laneH);

            // Segments (binary-search first visible for efficiency).
            for (let i = 0; i < segs.length; i++) {
                const sg = segs[i];
                if (sg.e < vs) continue;
                if (sg.s > ve) break;
                const x0 = Math.max(0, X(sg.s));
                const x1 = Math.min(W, X(sg.e + 1));
                if (x1 <= x0) continue;
                sg.owners.forEach((owner, index) => {
                    ctx.fillStyle = owner.color || cText3;
                    ctx.fillRect(x0, laneTop + index * laneH / sg.owners.length, Math.max(1, x1 - x0), laneH / sg.owners.length);
                });
            }

            const ppid = W / (ve - vs); // pixels per id
            // When zoomed in, delineate every individual id and label it.
            if (ppid >= 6) {
                ctx.strokeStyle = cSurface; ctx.lineWidth = 1; ctx.globalAlpha = 0.6;
                ctx.beginPath();
                const startId = Math.ceil(vs), endId = Math.floor(ve - 1e-9);
                for (let k = startId; k <= endId; k++) { const xx = X(k); ctx.moveTo(xx, laneTop); ctx.lineTo(xx, laneTop + laneH); }
                ctx.stroke(); ctx.globalAlpha = 1;
                if (ppid >= 34) {
                    ctx.fillStyle = cSurface; ctx.font = `10px ${cs.getPropertyValue("--font-mono") || "monospace"}`;
                    ctx.textAlign = "center"; ctx.textBaseline = "middle";
                    for (let k2 = startId; k2 <= endId; k2++) {
                        if (segAt(k2)) ctx.fillText(String(k2), X(k2) + ppid / 2, laneTop + laneH / 2);
                    }
                }
            }

            // Bottom axis with id value labels.
            ctx.fillStyle = cText3; ctx.strokeStyle = cBorder; ctx.lineWidth = 1;
            ctx.font = `11px ${cs.getPropertyValue("--font-mono") || "monospace"}`;
            ctx.textBaseline = "top";
            for (let t = 0; t <= 5; t++) {
                const v = Math.round(vs + (t / 5) * (ve - vs));
                const x = (t / 5) * W;
                ctx.beginPath(); ctx.moveTo(x, laneTop + laneH); ctx.lineTo(x, laneTop + laneH + 4); ctx.stroke();
                ctx.textAlign = t === 0 ? "left" : t === 5 ? "right" : "center";
                ctx.fillText(fmt(v), Math.min(Math.max(x, 1), W - 1), laneTop + laneH + 6);
            }

            // Hover cursor line.
            if (hoverId != null && hoverId >= vs && hoverId < ve) {
                const hx = X(hoverId + 0.5);
                ctx.strokeStyle = cText3; ctx.globalAlpha = 0.7; ctx.beginPath();
                ctx.moveTo(hx, laneTop); ctx.lineTo(hx, laneTop + laneH); ctx.stroke(); ctx.globalAlpha = 1;
            }

            const util = Math.round((used / domSpan) * 100);
            readout.textContent = `ids ${fmt(Math.floor(vs))}–${fmt(Math.ceil(ve - 1))
                } · ${fmt(used)} used across ${fmt(domSpan)} (${util}% dense)`;
        }

        canvas.addEventListener("wheel", (e) => {
            e.preventDefault();
            const rect = canvas.getBoundingClientRect();
            const W = canvas.clientWidth;
            const frac = (e.clientX - rect.left) / W;
            zoomAt(frac, e.deltaY < 0 ? 0.82 : 1.22);
        }, { passive: false });

        canvas.addEventListener("pointerdown", (e) => {
            dragging = true; moved = false; lastX = e.clientX;
            try { canvas.setPointerCapture(e.pointerId); } catch { }
        });
        canvas.addEventListener("pointermove", (e) => {
            const rect = canvas.getBoundingClientRect();
            const W = canvas.clientWidth;
            const mx = e.clientX - rect.left;
            if (dragging) {
                const dx = e.clientX - lastX; lastX = e.clientX;
                if (Math.abs(dx) > 2) moved = true;
                const shift = -dx / W * (ve - vs);
                vs += shift; ve += shift; clampView(); hideTip(); schedule();
                return;
            }
            const id = Math.floor(idAtPx(mx, W));
            hoverId = id;
            const seg = segAt(id);
            if (seg) {
                showTip(`<b>${title}: ${fmt(id)}</b><br>${seg.owners.map((owner) => `${esc(owner.name)}${owner.path ? `<br>${esc(owner.path)}` : ""}`).join("<br>")}<br>run ${fmt(seg.s)}–${fmt(seg.e)}`, e.clientX, e.clientY);
                canvas.style.cursor = "pointer";
            } else { hideTip(); canvas.style.cursor = "grab"; }
            schedule();
        });
        canvas.addEventListener("pointerup", (e) => {
            dragging = false;
            if (!moved) {
                const rect = canvas.getBoundingClientRect();
                const id = Math.floor(idAtPx(e.clientX - rect.left, canvas.clientWidth));
                const seg = segAt(id);
                hideTip(); showLookup(id);
                if (seg && seg.owners.length === 1 && seg.owners[0].inspect) seg.owners[0].inspect(id);
            }
        });
        canvas.addEventListener("pointerleave", () => { hoverId = null; dragging = false; hideTip(); schedule(); });

        if (window.ResizeObserver) { new ResizeObserver(schedule).observe(canvas); }
        requestAnimationFrame(draw);
        return wrap;
    }

    // ── Fillers table ────────────────────────────────────────────────────────
    const fillerSort = { key: "cell_count", dir: -1 };
    RENDER.fillers = (root) => {
        clear(root);
        const tbody = el("tbody");
        const note = el("div", { class: "count-note" });
        // Fixed derived columns + one dynamic column per discovered metadata key.
        const cols = [
            { k: "name", t: "Filler", num: false },
            { k: "universe_id", t: "Universe", num: true },
            { k: "envelope_count", t: "Envelopes", num: true },
            { k: "cell_count", t: "Cells", num: true },
            { k: "surface_count", t: "Surfaces", num: true }
        ];
        fillerMetaKeys.forEach((k) => { cols.push({ k: k, t: k, num: false, meta: true }); });
        const visibleMetadata = new Set(fillerMetaKeys.slice(0, 2));

        function cellVal(f, c) { return c.meta ? metaGet(f, c.k) : f[c.k]; }

        const thead = el("thead"), htr = el("tr");
        cols.forEach((c) => {
            const th = el("th", { class: c.num ? "num" : "" }, [
                document.createTextNode(c.t), el("span", { class: "arrow", text: " " })
            ]);
            actionable(th, () => {
                if (fillerSort.key === c.k) fillerSort.dir *= -1; else { fillerSort.key = c.k; fillerSort.dir = 1; }
                paint(curQ);
            });
            th.hidden = !!c.meta && !visibleMetadata.has(c.k);
            htr.appendChild(th);
        });
        thead.appendChild(htr);
        const table = el("table", { class: "data" }, [thead, tbody]);

        const colByKey = Object.create(null); cols.forEach((c) => { colByKey[c.k] = c; });

        let curQ = "";
        function paint(q) {
            curQ = q || "";
            clear(tbody);
            const rows = fillers.filter((f) => {
                if (!curQ) return true;
                const hay = [f.name, f.universe_id];
                fillerMetaKeys.forEach((k) => { hay.push(metaVal(metaGet(f, k))); });
                return hay.filter(Boolean).join(" ").toLowerCase().indexOf(curQ) >= 0;
            });
            const sc = colByKey[fillerSort.key] || cols[0], dir = fillerSort.dir;
            rows.sort((a, b) => {
                let av = cellVal(a, sc), bv = cellVal(b, sc);
                const an = typeof av === "number", bn = typeof bv === "number";
                if (an && bn) return (av - bv) * dir;
                if (av == null || av === "") av = "";
                if (bv == null || bv === "") bv = "";
                return String(metaVal(av)).localeCompare(String(metaVal(bv)), undefined, { numeric: true }) * dir;
            });
            note.textContent = `${rows.length} / ${fillers.length} fillers`;
            rows.forEach((f) => {
                const tr = el("tr", { "data-name": f.name });
                tr.appendChild(el("td", { class: "name" }, [
                    el("span", { class: "dot-swatch", style: `background:${uColor(f.universe_id)};margin-right:7px` }),
                    document.createTextNode(f.name)
                ]));
                tr.appendChild(el("td", { class: "num", text: fmt(f.universe_id) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.envelope_count) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.cell_count) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.surface_count) }));
                fillerMetaKeys.forEach((k) => {
                    const v = metaGet(f, k);
                    const cell = el("td", { class: "muted", text: v == null ? "—" : metaVal(v) });
                    cell.hidden = !visibleMetadata.has(k);
                    tr.appendChild(cell);
                });
                actionable(tr, () => openFiller(f.name));
                tbody.appendChild(tr);
            });
            paintSortArrows(htr, cols);
        }
        const columnPicker = detailsBlock("Metadata columns", el("div", { class: "toolbar" }, fillerMetaKeys.map((key) => {
            const checkbox = el("input", {
                type: "checkbox", onchange: (event) => {
                    if (event.currentTarget.checked) visibleMetadata.add(key); else visibleMetadata.delete(key);
                    cols.forEach((column, index) => { htr.children[index].hidden = !!column.meta && !visibleMetadata.has(column.k); });
                    paint(curQ);
                }
            });
            checkbox.checked = visibleMetadata.has(key);
            return el("label", {}, [checkbox, document.createTextNode(key)]);
        })));
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Filler models" }), el("span", { class: "spring" }), note]),
            el("div", { class: "card-body" }, [searchBox("Filter fillers…", paint), columnPicker]),
            el("div", { class: "tablewrap" }, table)
        ]));
        paint("");
    };
    function paintSortArrows(htr, cols) {
        cols.forEach((c, i) => {
            const a = htr.children[i].querySelector(".arrow");
            a.textContent = fillerSort.key === c.k ? (fillerSort.dir > 0 ? " ▲" : " ▼") : "";
        });
    }

    // ── Data cards ───────────────────────────────────────────────────────────
    RENDER.checks = (root) => {
        root.appendChild(el("h2", { text: "Build checks" }));
        const checks = DATA.checks || [];
        root.appendChild(checks.length ? tableOf(["Check", "Result", "Details"], checks.map((check) => [
            check.name, check.status.replace(/_/g, " "), el("ul", {}, (check.details || []).map((detail) => el("li", { text: detail })))
        ])) : el("p", { text: "Check results were not recorded in this report." }));
        root.appendChild(el("h3", { text: "Warnings" }));
        root.appendChild((DATA.warnings || []).length ? el("ul", {}, DATA.warnings.map((warning) => el("li", { text: warning }))) : el("p", { text: DATA.warnings ? "No build warnings." : "Warnings were not recorded." }));
    };

    RENDER.inputs = (root) => {
        root.appendChild(el("h2", { text: "Build identity and inputs" }));
        root.appendChild(tableOf(["Identity", "Value"], [
            ["Configuration", DATA.config_path], ["Commit", evidence.full_commit || DATA.commit_hash || "Not recorded"],
            ["Working tree", evidence.dirty == null ? "Not recorded" : evidence.dirty ? "Uncommitted changes" : "Clean"],
            ["Built", DATA.date_time], ["Content fingerprint (SHA-256)", evidence.content_fingerprint || "Not recorded"],
            ["Inputs stable during build", evidence.inputs_unchanged == null ? "Not verified" : evidence.inputs_unchanged ? "Yes" : "No"],
            ["Output", evidence.output ? `${evidence.output.path} (${fmt(evidence.output.bytes)} bytes)` : "Not recorded for this build"],
            ["Output SHA-256", evidence.output ? evidence.output.sha256 : "Not recorded"]
        ]));
        const host = el("div");
        const paint = (query) => {
            clear(host);
            host.appendChild(tableOf(["Role", "Name", "Path (relative to configuration)", "Bytes", "SHA-256"],
                inputs.filter((input) => `${input.name} ${input.role} ${input.path}`.toLowerCase().includes(query))
                    .map((input) => [input.role, input.name, input.path, fmt(input.bytes), input.sha256])));
        };
        if (inputs.length) { root.appendChild(searchBox("Filter input files", paint)); root.appendChild(host); paint(""); }
        else root.appendChild(el("p", { text: "Input identities were not recorded." }));
        for (const layer of evidence.configuration_chain || []) {
            root.appendChild(detailsBlock(`Configuration: ${layer.path}`, el("pre", { text: JSON.stringify(layer.values, null, 2) })));
        }
        if (evidence.resolved_configuration) root.appendChild(detailsBlock("Resolved configuration", el("pre", { text: JSON.stringify(evidence.resolved_configuration, null, 2) })));
    };

    RENDER.data = (root) => {
        clear(root);
        root.appendChild(el("h2", { text: "Selected data files" }));
        const category = el("select", { class: "input", "aria-label": "Card category", onchange: () => paint(query) },
            [el("option", { value: "all", text: "All card types" }), ...[...new Set(dataFiles.flatMap((file) => file.cards.map((entry) => entry.category)))].sort().map((value) => el("option", { value, text: value }))]);
        let query = "";
        const list = el("div", { class: "data-file-list" });
        const transformPlacements = new Map();
        for (const entry of envelopes) {
            const reference = /^\*?\(\s*(\d+)\s*\)$/.exec(entry.transform || "");
            if (reference) {
                const id = Number(reference[1]);
                if (!transformPlacements.has(id)) transformPlacements.set(id, []);
                transformPlacements.get(id).push(entry);
            }
        }
        const note = el("div", { class: "count-note", "aria-live": "polite" });
        function cardDetails(entry) {
            const body = el("div", { class: "card-details" }, el("pre", { text: entry.text }));
            if (entry.referenced_cell_runs && entry.referenced_cell_runs.length) {
                body.appendChild(detailsBlock("Directly referencing cell IDs", () => el("pre", { text: entry.referenced_cell_runs.map(([start, end]) => start === end ? String(start) : `${start}-${end}`).join(", ") })));
            }
            const material = /^M(\d+)$/.exec(entry.name);
            if (material) {
                const using = fillers.filter((filler) => (filler.materials || []).includes(Number(material[1])));
                body.appendChild(el("div", { class: "chips" }, using.map((filler) => el("button", { class: "textbtn", text: `${filler.name} (${filler.envelope_count} placements)`, onclick: () => openFiller(filler.name) }))));
            }
            const transform = /^\*?TR(\d+)$/.exec(entry.name);
            if (transform) {
                const using = transformPlacements.get(Number(transform[1])) || [];
                body.appendChild(detailsBlock(`Envelope placements (${using.length})`, () => el("div", {}, using.map((placement) => el("button", { class: "textbtn", text: placement.envelope_name, onclick: () => openEnvelope(placement.envelope_name) })))));
            }
            return body;
        }
        function groupedCards(cards) {
            const groups = new Map();
            for (const card of cards) {
                if (!groups.has(card.category)) groups.set(card.category, []);
                groups.get(card.category).push(card);
            }
            return groups;
        }
        function paint(value) {
            query = value; clear(list);
            let shown = 0;
            for (const file of dataFiles) {
                const fileMatches = !query || `${file.name} ${file.path || ""} ${file.roles.join(" ")}`.toLowerCase().includes(query);
                const matching = file.cards.filter((entry) => (category.value === "all" || entry.category === category.value)
                    && (fileMatches || `${entry.name} ${entry.text}`.toLowerCase().includes(query)));
                if (!matching.length && (!fileMatches || category.value !== "all")) continue;
                shown++;
                const renderContents = () => {
                    const content = el("div", { class: "data-file-content" });
                    for (const [type, cards] of groupedCards(matching)) {
                        const group = el("section", { class: "data-card-group" }, el("h3", { text: `${type} (${fmt(cards.length)})` }));
                        const identifiers = el("div", { class: "card-identifiers" });
                        for (const entry of cards) {
                            const detail = detailsBlock(entry.name, () => cardDetails(entry));
                            detail.classList.add("data-card-entry");
                            if (query && entry.name.toLowerCase().includes(query)) {
                                const summary = detail.querySelector("summary");
                                clear(summary); summary.appendChild(el("mark", { text: entry.name }));
                            }
                            identifiers.appendChild(detail);
                        }
                        group.appendChild(identifiers); content.appendChild(group);
                    }
                    if (!matching.length) content.appendChild(el("p", { class: "muted", text: evidence.data_cards && file.path ? "No parsed cards recorded in this file." : "Card inventory was not recorded for this file." }));
                    return content;
                };
                const expanded = !!query || category.value !== "all";
                const row = detailsBlock(file.name, expanded ? renderContents() : renderContents);
                row.classList.add("data-file");
                row.open = expanded;
                const summary = row.querySelector("summary");
                clear(summary);
                const counts = [...groupedCards(file.cards)].map(([type, cards]) => `${fmt(cards.length)} ${type.toLowerCase()}`).join(" · ");
                append(summary, [el("span", { class: "data-file-name", text: file.name }),
                el("span", { class: "data-file-role", text: `Selected as: ${file.roles.map((role) => role === "transforms" ? "transformations" : role).join(", ")}` }),
                el("span", { class: "data-file-path mono", text: file.path || "Path not recorded" }),
                el("span", { class: "data-file-counts", text: counts || (evidence.data_cards && file.path ? "0 parsed cards" : "Card inventory not recorded") })]);
                list.appendChild(row);
            }
            note.textContent = `${shown} / ${nData} files`;
            if (!shown) list.appendChild(el("p", { class: "empty", text: nData ? "No matching data files." : "No data files selected." }));
        }
        root.appendChild(el("div", { class: "toolbar" }, [category, searchBox("Filter files, card IDs, or definitions", paint), note]));
        root.appendChild(list); paint("");
    };

    // ── Diff ─────────────────────────────────────────────────────────────────
    RENDER.diff = (root) => {
        clear(root);
        const out = el("div", { class: "section-gap" });
        const dz = el("div", { class: "dropzone" }, [
            el("div", { style: "font-weight:700;margin-bottom:6px", text: "Compare against another build" }),
            el("div", { text: "Drop a build_report.json here, or click to choose a file." })
        ]);
        const fileInp = el("input", { type: "file", accept: "application/json,.json", class: "hidden" });
        actionable(dz, () => fileInp.click());
        dz.addEventListener("dragover", (e) => { e.preventDefault(); dz.classList.add("hot"); });
        dz.addEventListener("dragleave", () => { dz.classList.remove("hot"); });
        dz.addEventListener("drop", (e) => {
            e.preventDefault(); dz.classList.remove("hot");
            if (e.dataTransfer.files[0]) readFile(e.dataTransfer.files[0]);
        });
        fileInp.addEventListener("change", (e) => {
            const [file] = e.currentTarget.files;
            if (file) readFile(file);
        });
        function readFile(file) {
            const fr = new FileReader();
            fr.onload = () => {
                try { renderDiff(JSON.parse(fr.result)); }
                catch (err) { clear(out); out.appendChild(el("div", { class: "empty", text: `Could not parse JSON: ${err}` })); }
            };
            fr.readAsText(file);
        }
        function renderDiff(other) {
            clear(out);
            if (!other || !Array.isArray(other.envelope_entries) || !Array.isArray(other.filler_entries) || ![1, 2].includes(other.schema_version)) {
                out.appendChild(el("p", { class: "empty", text: "Unsupported report: expected schema 1 or 2 with envelope and filler entries." }));
                return;
            }
            const oEnv = Object.create(null); other.envelope_entries.forEach((e) => { oEnv[e.envelope_name] = e; });
            const cEnv = Object.create(null); envelopes.forEach((e) => { cEnv[e.envelope_name] = e; });
            out.appendChild(tableOf(["Build", "Configuration", "Commit", "Built"], [
                ["Baseline", other.config_path || "Unknown", other.commit_hash || "Unknown", other.date_time || "Unknown"],
                ["Current", DATA.config_path, DATA.commit_hash, DATA.date_time]
            ]));
            const added = [], removed = [], changed = [];
            envelopes.forEach((e) => {
                const o = oEnv[e.envelope_name];
                if (!o) added.push(e);
                else if ((o.filler_name || null) !== (e.filler_name || null) || (o.transform || null) !== (e.transform || null) || o.universe_id !== e.universe_id || statusOf(o) !== statusOf(e))
                    changed.push({ name: e.envelope_name, before: o, after: e });
            });
            (other.envelope_entries || []).forEach((e) => { if (!cEnv[e.envelope_name]) removed.push(e); });

            out.appendChild(el("div", { class: "grid kpis" }, [
                kpi(added.length, "Added envelopes"),
                kpi(removed.length, "Removed envelopes"),
                kpi(changed.length, "Changed assignments")
            ]));
            out.querySelectorAll(".kpi .v").forEach((value, index) => {
                value.style.color = ["var(--ok)", "var(--danger)", "var(--warn)"][index];
            });

            if (changed.length) out.appendChild(card("Changed assignments", tableOf(
                ["Envelope", "Baseline filler / status", "Current filler / status", "Universe (before / after)", "Transform (before / after)"],
                changed.map((change) => [
                    el("button", { class: "textbtn", text: change.name, onclick: () => openEnvelope(change.name) }),
                    change.before.filler_name || statusLabel(statusOf(change.before)), change.after.filler_name || statusLabel(statusOf(change.after)),
                    `${change.before.universe_id ?? "None"} / ${change.after.universe_id ?? "None"}`,
                    `${change.before.transform || "None"} / ${change.after.transform || "None"}`
                ]))));

            const oldEvidence = other.evidence || {};
            const oldInputs = oldEvidence.inputs || [];
            const keyOf = (input) => `${input.role}:${input.name}`;
            const previousInputs = new Map(oldInputs.map((input) => [keyOf(input), input]));
            const currentInputs = new Map(inputs.map((input) => [keyOf(input), input]));
            const inputChanges = [];
            for (const key of new Set([...previousInputs.keys(), ...currentInputs.keys()])) {
                const before = previousInputs.get(key), after = currentInputs.get(key);
                if (before && after && before.sha256 === after.sha256 && before.path === after.path) continue;
                const input = after || before;
                const impacts = new Set([...envelopes, ...other.envelope_entries]
                    .filter((entry) => (input.role === "filler" || input.role === "filler_metadata") ? entry.filler_name === input.name : true)
                    .map((entry) => entry.envelope_name));
                inputChanges.push([input.role, input.name, !before ? "Added" : !after ? "Removed" : before.sha256 !== after.sha256 ? "Content changed" : "Path changed",
                before ? before.path : "None", after ? after.path : "None",
                input.role === "filler" || input.role === "filler_metadata" ? `${fmt(impacts.size)} placements (baseline/current union)` : "Build-wide input"]);
            }
            if (inputChanges.length) out.appendChild(card("Input changes and impact", tableOf(["Role", "Input", "Change", "Baseline path", "Current path", "Impact"], inputChanges)));
            if (!oldInputs.length || !inputs.length) out.appendChild(el("p", { class: "notice", text: "Content comparison is incomplete: input hashes were not recorded for one or both builds." }));

            const previousFillers = new Map(other.filler_entries.map((filler) => [filler.name, filler]));
            const componentChanges = [];
            for (const filler of fillers) {
                const previous = previousFillers.get(filler.name);
                if (!previous) continue;
                for (const field of ["universe_id", "cell_count", "surface_count", "materials", "cell_id_runs", "surface_id_runs"]) {
                    if (JSON.stringify(previous[field]) !== JSON.stringify(filler[field])) {
                        const summary = (value) => field.endsWith("_runs") ? `${(value || []).length} ID runs` : metaVal(value);
                        componentChanges.push([filler.name, field, summary(previous[field]), summary(filler[field]), `${fmt(filler.envelope_count)} current placements`]);
                    }
                }
            }
            if (componentChanges.length) out.appendChild(card("Component changes", tableOf(["Filler", "Field", "Baseline", "Current", "Impact"], componentChanges)));
            const summaryChanges = [];
            for (const field of ["total_cells", "total_surfaces", "materials", "transforms", "source", "tallies"]) {
                if (JSON.stringify(other[field]) !== JSON.stringify(DATA[field])) summaryChanges.push([field, metaVal(other[field]), metaVal(DATA[field])]);
            }
            if (summaryChanges.length) out.appendChild(card("Build summary changes", tableOf(["Field", "Baseline", "Current"], summaryChanges)));
            const cardKey = (entry) => `${entry.category}:${entry.name}`;
            const previousCards = new Map((oldEvidence.data_cards || []).map((entry) => [cardKey(entry), entry]));
            const currentCards = new Map(dataCards.map((entry) => [cardKey(entry), entry]));
            const cardChanges = [];
            for (const key of new Set([...previousCards.keys(), ...currentCards.keys()])) {
                const before = previousCards.get(key), after = currentCards.get(key);
                if (before && after && before.text === after.text) continue;
                cardChanges.push([(after || before).name, before ? el("pre", { text: before.text }) : "Absent", after ? el("pre", { text: after.text }) : "Absent"]);
            }
            if (cardChanges.length) out.appendChild(detailsBlock(`Data card changes (${cardChanges.length})`, tableOf(["Card", "Baseline", "Current"], cardChanges)));
            [["Added in this build", added, "tag-added"], ["Removed (only in other)", removed, "tag-removed"]].forEach((grp) => {
                if (!grp[1].length) return;
                const chips = el("div", { class: "chips" });
                grp[1].forEach((e) => { chips.appendChild(el("span", { class: `chip ${grp[2]}`, text: e.envelope_name })); });
                out.appendChild(el("div", { class: "card" }, [el("div", { class: "card-head" }, el("h3", { text: grp[0] })), el("div", { class: "card-body" }, chips)]));
            });
            if (!added.length && !removed.length && !changed.length && !inputChanges.length && !componentChanges.length && !summaryChanges.length && !cardChanges.length)
                out.appendChild(el("div", { class: "empty", text: "No differences in recorded build content." }));
        }
        root.appendChild(el("div", { class: "card" }, el("div", { class: "card-body" }, dz)));
        root.appendChild(fileInp);
        root.appendChild(out);
    };

    // ── Filler detail drawer ─────────────────────────────────────────────────
    const scrim = el("div", { class: "scrim", onclick: closeDrawer });
    const drawer = el("div", { class: "drawer" });
    drawer.hidden = true;
    document.body.appendChild(scrim);
    document.body.appendChild(drawer);
    let previousFocus = null;
    drawer.setAttribute("role", "dialog");
    drawer.setAttribute("aria-modal", "true");
    drawer.setAttribute("aria-label", "Model details");
    document.addEventListener("keydown", (event) => {
        if (!drawer.classList.contains("open")) return;
        if (event.key === "Escape") closeDrawer();
        if (event.key === "Tab") {
            const focusable = [...drawer.querySelectorAll("button, input, select, summary, [tabindex='0']")];
            const first = focusable[0], last = focusable[focusable.length - 1];
            if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
            if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
        }
    });

    function revealDrawer(hash) {
        if (!drawer.classList.contains("open")) previousFocus = document.activeElement;
        drawer.hidden = false;
        scrim.classList.add("open");
        drawer.classList.add("open");
        app.setAttribute("inert", "");
        document.body.classList.add("drawer-open");
        setHash(hash);
        const first = drawer.querySelector("button");
        if (first) first.focus();
    }

    function openEnvelope(name) {
        const entry = envelopes.find((item) => item.envelope_name === name);
        if (!entry) return;
        clear(drawer);
        drawer.appendChild(el("div", { class: "drawer-head" }, [el("h3", { class: "spring", text: name }),
        el("button", { class: "iconbtn", "aria-label": "Close details", text: "\u00d7", onclick: closeDrawer })]));
        const structure = inputs.find((input) => input.role === "envelope_structure");
        const body = el("div", { class: "drawer-body" }, [tableOf(["Placement", "Value"], [
            ["Status", statusLabel(statusOf(entry))], ["Cell IDs", (entry.cell_ids || []).join(", ") || "Not recorded"],
            ["Envelope structure", structure ? structure.path : "Not recorded"],
            ["Assigned by", (evidence.assignment_origins || {})[name] || "Not recorded"],
            ["Universe", entry.universe_id == null ? "None" : String(entry.universe_id)],
            ["Transform", entry.transform || "None"]
        ])]);
        if (entry.filler_name) body.appendChild(el("button", { class: "btn", text: `Filler: ${entry.filler_name}`, onclick: () => openFiller(entry.filler_name) }));
        const metadata = entry.metadata || {};
        if (Object.keys(metadata).length) body.appendChild(tableOf(["Metadata", "Value"], Object.entries(metadata).map(([key, value]) => [key, metaVal(value)])));
        drawer.appendChild(body);
        revealDrawer({ envelope: name, filler: null });
    }

    function openFiller(name) {
        const f = fillerByName[name];
        if (!f) return;
        clear(drawer);
        const desc = descOf(f);
        drawer.appendChild(el("div", { class: "drawer-head" }, [
            el("span", { class: "dot-swatch", style: `background:${uColor(f.universe_id)};width:16px;height:16px;margin-top:4px` }),
            el("div", { class: "spring" }, [
                el("h3", { text: f.name }),
                desc ? el("div", { class: "muted", style: "font-size:.84rem;margin-top:2px", text: desc }) : null
            ]),
            el("button", { class: "iconbtn", title: "Close", onclick: closeDrawer, text: "✕" })
        ]));
        const body = el("div", { class: "drawer-body" });
        const kv = el("dl", { class: "kv" });
        function row(k, v) { kv.appendChild(el("dt", { text: k })); kv.appendChild(el("dd", { text: v })); }
        row("Universe id", String(f.universe_id));
        row("Envelopes filled", String(f.envelope_count));
        row("Cells", fmt(f.cell_count));
        row("Surfaces", fmt(f.surface_count));
        const source = inputFor(name);
        if (source) { row("Input file", source.path); row("SHA-256", source.sha256); }
        const cr = idExtent(f.cell_id_runs), sr = idExtent(f.surface_id_runs);
        if (cr) row("Cell ids", `${fmt(cr[0])} – ${fmt(cr[1])} · ${f.cell_id_runs.length} run${f.cell_id_runs.length > 1 ? "s" : ""}`);
        if (sr) row("Surface ids", `${fmt(sr[0])} – ${fmt(sr[1])} · ${f.surface_id_runs.length} run${f.surface_id_runs.length > 1 ? "s" : ""}`);
        body.appendChild(kv);

        // Arbitrary, project-defined metadata (rendered generically).
        const mkeys = f.metadata ? Object.keys(f.metadata).filter((k) => { return ["description", "desc", "title"].indexOf(k) < 0; }) : [];
        if (mkeys.length) {
            body.appendChild(el("div", { class: "subhead", text: "Metadata" }));
            const mkv = el("dl", { class: "kv" });
            mkeys.forEach((k) => {
                mkv.appendChild(el("dt", { text: k }));
                mkv.appendChild(el("dd", { text: metaVal(f.metadata[k]) }));
            });
            body.appendChild(mkv);
        }

        if (f.materials && f.materials.length) {
            body.appendChild(el("div", { class: "subhead", text: `Materials (${f.materials.length})` }));
            const mc = el("div", { class: "chips" });
            f.materials.forEach((material) => {
                mc.appendChild(el("button", {
                    class: "chip", text: `mat ${material}`, onclick: () => {
                        closeDrawer(); go("data"); const search = $("#panel-data input"); if (search) { search.value = `m${material} `; search.dispatchEvent(new window.Event("input")); }
                    }
                }));
            });
            body.appendChild(mc);
        }

        const filledEnvs = envelopes.filter((e) => { return e.filler_name === name; });
        if (filledEnvs.length) {
            body.appendChild(el("div", { class: "subhead", text: `Fills ${filledEnvs.length} envelope${filledEnvs.length > 1 ? "s" : ""}` }));
            const list = el("div", {});
            const note = el("div", { class: "count-note" });
            let limit = 100, query = "";
            const more = el("button", { class: "btn", text: "Show more", onclick: () => { limit += 100; paintPlacements(); } });
            function paintPlacements() {
                clear(list);
                const matching = filledEnvs.filter((entry) => !query || matchEnv(entry).includes(query));
                note.textContent = `${Math.min(limit, matching.length)} / ${matching.length} placements`;
                more.hidden = limit >= matching.length;
                matching.slice(0, limit).forEach((e) => {
                    const r = el("div", { class: "leaf", style: "padding-left:8px" }, [
                        el("span", { class: "en", text: e.envelope_name })
                    ]);
                    if (e.transform) r.appendChild(el("span", { class: "chip", text: e.transform }));
                    const d = descOf(e);
                    if (d) r.appendChild(el("span", { class: "desc", text: d }));
                    actionable(r, () => openEnvelope(e.envelope_name));
                    list.appendChild(r);
                });
            }
            body.appendChild(searchBox("Filter placements", (value) => { query = value; limit = 100; paintPlacements(); }));
            body.appendChild(note); body.appendChild(list); body.appendChild(more); paintPlacements();
        }
        drawer.appendChild(body);
        revealDrawer({ filler: name, envelope: null });
    }
    function closeDrawer() {
        scrim.classList.remove("open");
        drawer.classList.remove("open");
        drawer.hidden = true;
        app.removeAttribute("inert");
        document.body.classList.remove("drawer-open");
        setHash({ filler: null, envelope: null });
        if (previousFocus && previousFocus.isConnected) previousFocus.focus();
    }

    // ── Boot ─────────────────────────────────────────────────────────────────
    (function init() {
        try {
            const saved = localStorage.getItem("gitronics-theme");
            if (saved) document.documentElement.setAttribute("data-theme", saved);
            else if (window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches)
                document.documentElement.setAttribute("data-theme", "dark");
        } catch { }
        paintThemeIcon();
        const h = getHash();
        go(h.tab || "overview");
        if (h.filler) setTimeout(() => { openFiller(h.filler); }, 60);
        else if (h.envelope) setTimeout(() => { openEnvelope(h.envelope); }, 60);
    })();
})();
