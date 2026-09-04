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
        document.body.textContent = `Failed to parse report data: ${  e}`;
        return;
    }

    const envelopes = DATA.envelope_entries || [];
    const fillers = DATA.filler_entries || [];
    const fillerByName = {};
    fillers.forEach((f) => { fillerByName[f.name] = f; });

    // ── Free-form metadata helpers ─────────────────────────────────────────
    // Metadata keys are project-defined and arbitrary; discover them from data.
    function discoverKeys(items) {
        const order = [], seen = {};
        items.forEach((it) => {
            const m = it.metadata; if (!m) return;
            Object.keys(m).forEach((k) => { if (!seen[k]) { seen[k] = 1; order.push(k); } });
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
        const prefs = ["zone", "region", "system", "group", "sector", "row", "column", "col"];
        const picked = prefs.filter((p) => { return keys.indexOf(p) >= 0; });
        if (picked.length) return picked.slice(0, 2);
        return keys.slice(0, 2);
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
        return `hsl(${  uHue(id)  } ${  dark ? 55 : 62  }% ${  dark ? 55 : 52  }%)`;
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
        tip.style.left = `${Math.min(x + 14, window.innerWidth - w - 8)  }px`;
        tip.style.top = `${Math.max(8, y - h - 12)  }px`;
    }
    function hideTip() { tip.classList.remove("show"); }

    // ── Derived stats ──────────────────────────────────────────────────────
    const nFilled = envelopes.filter((e) => { return e.filler_name; }).length;
    const nNull = envelopes.length - nFilled;
    const coverage = envelopes.length ? Math.round((nFilled / envelopes.length) * 100) : 0;
    const nData = (DATA.materials || []).length + (DATA.tallies || []).length +
        (DATA.transforms || []).length + (DATA.source ? 1 : 0);

    // ── App shell ──────────────────────────────────────────────────────────
    const app = el("div");
    document.body.appendChild(app);

    // ── Tabs (defined before the header, which mounts them) ────────────────
    const TABS = [
        { id: "overview", label: "Overview", icon: "layers" },
        { id: "explorer", label: "Explorer", icon: "tree", pill: envelopes.length },
        { id: "coverage", label: "Coverage Map", icon: "grid" },
        { id: "idmap", label: "ID Map", icon: "map" },
        { id: "fillers", label: "Fillers", icon: "box", pill: fillers.length },
        { id: "data", label: "Data Cards", icon: "layers", pill: nData },
        { id: "diff", label: "Diff", icon: "diff" }
    ];
    const tabBtns = {};
    function buildTabs() {
        const nav = el("nav", { class: "tabs" });
        TABS.forEach((t) => {
            const b = el("button", { class: "tab", onclick: () => { go(t.id); } }, [
                icon(t.icon), document.createTextNode(t.label)
            ]);
            if (t.pill != null) b.appendChild(el("span", { class: "pill", text: String(t.pill) }));
            tabBtns[t.id] = b;
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
                el("span", { class: "ver", text: `v${  DATA.gitronics_version}` })
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
        document.createTextNode(` · schema v${  DATA.schema_version || 1}`)
    ]));

    // ── Tabs (mounting handled by buildTabs, defined above) ────────────────
    const built = {};
    const panelEls = {};
    function go(id) {
        if (!tabBtns[id]) id = "overview";
        for (const k in tabBtns) tabBtns[k].classList.toggle("active", k === id);
        for (const p in panelEls) panelEls[p].classList.toggle("active", p === id);
        if (!panelEls[id]) {
            const pane = el("section", { class: "panel", id: `panel-${  id}` });
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
        const s = Object.keys(o).map((k) => { return `${k  }=${  encodeURIComponent(o[k])}`; }).join("&");
        history.replaceState(null, "", `#${  s}`);
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
    }

    // ── Search box widget ────────────────────────────────────────────────────
    function searchBox(placeholder, oninput) {
        const box = el("div", { class: "searchbox" });
        box.appendChild(icon("search"));
        const inp = el("input", { class: "input", type: "text", placeholder: placeholder, oninput: (e) => { oninput(e.currentTarget.value.toLowerCase().trim()); } });
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
    function donutMini(pct) {
        const s = svgEl("svg", { class: "spark", width: 34, height: 34, viewBox: "0 0 36 36" });
        const bg = svgEl("circle", { cx: 18, cy: 18, r: 15, fill: "none", stroke: "var(--surface-3)", "stroke-width": 5 });
        const c = 2 * Math.PI * 15;
        const fg = svgEl("circle", { cx: 18, cy: 18, r: 15, fill: "none", stroke: "var(--ok)", "stroke-width": 5, "stroke-linecap": "round", "stroke-dasharray": `${c * pct / 100  } ${  c}`, transform: "rotate(-90 18 18)" });
        s.appendChild(bg); s.appendChild(fg);
        return s;
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

        // KPIs
        const kpis = el("div", { class: "grid kpis" }, [
            kpi(DATA.total_cells, "Cells"),
            kpi(DATA.total_surfaces, "Surfaces"),
            kpi(envelopes.length, "Envelopes", `${nFilled  } filled · ${  nNull  } null`),
            kpi(fillers.length, "Filler Models"),
            kpi(`${coverage  }%`, "Fill Coverage", null, donutMini(coverage)),
            kpi(nData, "Data Files")
        ]);
        // coverage % value shows literal string; fix formatting
        kpis.children[4].querySelector(".v").textContent = `${coverage  }%`;
        wrap.appendChild(kpis);

        const cols = el("div", { class: "grid", style: "grid-template-columns:repeat(auto-fit,minmax(320px,1fr))" });
        wrap.appendChild(cols);

        // Coverage donut card
        cols.appendChild(card("Fill coverage", donutCard()));

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
        const mats = Object.keys(matCount).map((m) => { return { label: `mat ${  m}`, value: matCount[m] }; })
            .sort((a, b) => { return b.value - a.value; }).slice(0, 12);
        if (mats.length) cols.appendChild(card("Material usage (fillers per material)", barChart(mats)));

        // Breakdown by a chosen metadata field (fully generic).
        if (envMetaKeys.length) {
            const field = defaultFields(envMetaKeys)[0] || envMetaKeys[0];
            const breakdownCard = card(`Envelopes by ${  field}`, el("div"));
            const head = breakdownCard.querySelector(".card-head");
            const sel = el("select", { class: "input", style: "width:auto;margin-left:auto" });
            envMetaKeys.forEach((k) => { sel.appendChild(el("option", { value: k, text: k })); });
            sel.value = field;
            head.appendChild(sel);
            const bd = breakdownCard.querySelector(".card-body");
            function paintBreakdown() {
                clear(bd);
                bd.appendChild(barChart(rollupBy(sel.value).map((g) => {
                    return { label: g.key, value: g.total, sub: `${g.filled  }/${  g.total  } filled` };
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

    function donutCard() {
        const wrap = el("div", { class: "donut-wrap" });
        const size = 150, r = 60, cx = 75, cy = 75, sw = 22;
        const s = svgEl("svg", { width: size, height: size, viewBox: "0 0 150 150" });
        const segs = [
            { v: nFilled, c: "var(--ok)", label: "Filled" },
            { v: nNull, c: "var(--null)", label: "Null" }
        ].filter((x) => { return x.v > 0; });
        const total = nFilled + nNull || 1, circ = 2 * Math.PI * r;
        let off = 0;
        segs.forEach((seg) => {
            const frac = seg.v / total;
            const c = svgEl("circle", {
                cx: cx, cy: cy, r: r, fill: "none", stroke: seg.c, "stroke-width": sw,
                "stroke-dasharray": `${circ * frac  } ${  circ}`,
                "stroke-dashoffset": -off * circ,
                transform: `rotate(-90 ${  cx  } ${  cy  })`
            });
            s.appendChild(c);
            off += frac;
        });
        s.appendChild(svgEl("circle", { cx: cx, cy: cy, r: r - sw / 2 - 2, fill: "var(--surface)" }));
        const t1 = svgEl("text", { x: cx, y: cy - 2, "text-anchor": "middle", "font-size": 26, "font-weight": 800, fill: "var(--text)" });
        t1.textContent = `${coverage  }%`;
        const t2 = svgEl("text", { x: cx, y: cy + 18, "text-anchor": "middle", "font-size": 11, fill: "var(--text-3)" });
        t2.textContent = "filled";
        s.appendChild(t1); s.appendChild(t2);
        wrap.appendChild(s);
        const leg = el("div", {});
        leg.appendChild(legRow("var(--ok)", "Filled", nFilled));
        leg.appendChild(legRow("var(--null)", "Null", nNull));
        wrap.appendChild(leg);
        return wrap;
    }
    function legRow(color, label, val) {
        return el("div", { class: "legend", style: "margin:4px 0" }, el("div", { class: "item" }, [
            el("span", { class: "dot-swatch", style: `background:${  color}` }),
            el("span", { text: `${label  } · ${  fmt(val)}` })
        ]));
    }

    function barChart(rows) {
        const max = rows.reduce((m, r) => { return Math.max(m, r.value); }, 0) || 1;
        const box = el("div", {});
        if (!rows.length) { box.appendChild(el("div", { class: "empty", text: "No data" })); return box; }
        rows.forEach((r) => {
            const fill = el("div", { class: "fill", style: `width:${  r.value / max * 100  }%${  r.id != null ? `;background:${  uColor(r.id)}` : ""}` });
            const row = el("div", { class: "chart-bar-row" }, [
                el("div", { class: "lab", title: r.label, text: r.label }),
                el("div", { class: "track" }, fill),
                el("div", { class: "val", text: fmt(r.value) + (r.sub ? "" : "") })
            ]);
            if (r.onclick) { row.style.cursor = "pointer"; row.addEventListener("click", r.onclick); }
            if (r.sub) row.querySelector(".lab").title = r.sub;
            box.appendChild(row);
        });
        return box;
    }

    // Group envelopes by an arbitrary field key (metadata key, or the special
    // "__filler__"/"__universe__"). Returns [{key, total, filled}] sorted by size.
    function groupKeyOf(e, field) {
        if (field === "__filler__") return e.filler_name || "(null)";
        if (field === "__universe__") return e.universe_id != null ? `u${  e.universe_id}` : "(null)";
        const v = metaGet(e, field);
        return v == null || v === "" ? "—" : metaVal(v);
    }
    function rollupBy(field) {
        const m = {};
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
        const sel = el("select", { class: "input", style: "width:auto" });
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
        let g1 = defaults[0] || (envMetaKeys.length ? envMetaKeys[0] : "__none__");
        let g2 = defaults[1] || "__none__";
        let curQ = getHash().q || "";

        const sel1 = groupSelect(g1, (v) => { g1 = v; rebuild(curQ); });
        const sel2 = groupSelect(g2, (v) => { g2 = v; rebuild(curQ); });
        const body = el("div", { class: "card-body section-gap" }, [
            el("div", { class: "toolbar" }, [
                searchBox("Filter by envelope, filler, metadata, universe…", (q) => { curQ = q; rebuild(q); }),
                el("span", { class: "muted", style: "font-size:.8rem", text: "Group by" }), sel1,
                el("span", { class: "muted", style: "font-size:.8rem", text: "then" }), sel2,
                note
            ]),
            treeHost
        ]);
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Model hierarchy" }),
            el("span", { class: "spring" }),
            el("span", { class: "chip ok", text: `${nFilled  } filled` }),
            el("span", { class: "chip null", text: `${nNull  } null` })]),
            body
        ]));

        function buildGroups(list, field) {
            const groups = {}, order = [];
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
            const shown = envelopes.filter((e) => { return !q || matchEnv(e).indexOf(q) >= 0; });
            note.textContent = `${shown.length  } / ${  envelopes.length  } envelopes`;
            if (!shown.length) { treeHost.appendChild(el("div", { class: "empty", text: "No matches" })); return; }

            if (g1 === "__none__") { shown.forEach((e) => { treeHost.appendChild(leafRow(e)); }); return; }

            const lvl1 = buildGroups(shown, g1);
            const autoOpen = lvl1.order.length <= 3 || !!q;
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
    };

    function matchEnv(e) {
        const parts = [e.envelope_name, e.filler_name, e.universe_id, e.transform];
        if (e.metadata) Object.keys(e.metadata).forEach((k) => { parts.push(metaVal(e.metadata[k])); });
        return parts.filter((x) => { return x != null && x !== ""; }).join(" ").toLowerCase();
    }

    function leafRow(e) {
        const filled = !!e.filler_name;
        const leaf = el("div", { class: "leaf" });
        leaf.appendChild(el("span", { class: "dot-swatch", style: `background:${  filled ? uColor(e.universe_id) : "var(--null)"}` }));
        leaf.appendChild(el("span", { class: "en", text: e.envelope_name }));
        if (filled) {
            leaf.appendChild(el("span", { class: "fl", text: `→ ${  e.filler_name}` }));
            leaf.appendChild(el("span", { class: "chip accent", text: `u${  e.universe_id}` }));
            if (e.transform) leaf.appendChild(el("span", { class: "chip", text: e.transform }));
        } else {
            leaf.appendChild(el("span", { class: "chip null", text: "null" }));
        }
        leaf.appendChild(el("span", { class: "spring" }));
        const d = descOf(e);
        if (d) leaf.appendChild(el("span", { class: "desc", text: d }));
        if (filled) leaf.addEventListener("click", () => { openFiller(e.filler_name); });
        return leaf;
    }

    // ── Coverage map ─────────────────────────────────────────────────────────
    RENDER.coverage = (root) => {
        clear(root);
        const host = el("div", {});
        let g = defaultFields(envMetaKeys)[0] || "__none__";
        const sel = groupSelect(g, (v) => { g = v; rebuild(); });
        const body = el("div", { class: "card-body section-gap" }, [
            el("div", { class: "toolbar" }, [
                el("span", { class: "muted", style: "font-size:.8rem", text: "Group by" }), sel,
                el("span", { class: "item muted", style: "font-size:.8rem", text: "Each square is an envelope, colored by the universe filling it. Hover for details, click to inspect the filler." })
            ]),
            host
        ]);
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Coverage map" }),
            el("span", { class: "spring" }),
            el("span", { class: "chip accent", text: `${coverage  }% filled` })]),
            body
        ]));

        function rebuild() {
            clear(host);
            const groups = {}, order = [];
            envelopes.forEach((e) => {
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
                    el("span", { class: "chip", text: `${filled  }/${  list.length}` })
                ]));
                const grid = el("div", { class: "cov-grid" });
                list.forEach((e) => {
                    const filledCell = !!e.filler_name;
                    const cell = el("div", {
                        class: "cov-cell",
                        style: `background:${  filledCell ? uColor(e.universe_id) : "var(--null-weak)"}`
                    });
                    cell.addEventListener("mousemove", (ev) => { showTip(envTip(e), ev.clientX, ev.clientY); });
                    cell.addEventListener("mouseleave", hideTip);
                    if (filledCell) cell.addEventListener("click", () => { hideTip(); openFiller(e.filler_name); });
                    grid.appendChild(cell);
                });
                zwrap.appendChild(grid);
                host.appendChild(zwrap);
            });
        }
        rebuild();
    };

    // Tooltip HTML for an envelope: name, description, up to 4 metadata fields, filler.
    function envTip(e) {
        let s = `<b>${  esc(e.envelope_name)  }</b>`;
        const d = descOf(e); if (d) s += `<br>${  esc(d)}`;
        if (e.metadata) {
            Object.keys(e.metadata).slice(0, 5).forEach((k) => {
                if (["description", "desc", "title"].indexOf(k) >= 0) return;
                s += `<br><span style='opacity:.7'>${  esc(k)  }:</span> ${  esc(metaVal(e.metadata[k]))}`;
            });
        }
        s += `<br>${  e.filler_name ? `→ ${  esc(e.filler_name)  } (u${  e.universe_id  })` : "null"}`;
        return s;
    }

    function esc(s) { return String(s).replace(/[&<>"]/g, (c) => { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]; }); }

    // ── ID memory map (zoomable, exact ids) ──────────────────────────────────
    RENDER.idmap = (root) => {
        clear(root);
        const body = el("div", { class: "card-body section-gap" });
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Card-ID memory map" }),
            el("span", { class: "spring" }),
            el("span", { class: "muted count-note", text: "Exact id positions, colored by universe. Scroll to zoom, drag to pan; zoom in to see every individual id." })]),
            body
        ]));
        body.appendChild(zoomLane("Cell ids", "cell_id_runs"));
        body.appendChild(zoomLane("Surface ids", "surface_id_runs"));
    };

    function zoomLane(title, key) {
        // Flatten every filler's runs into disjoint segments over the id axis.
        const segs = [];
        fillers.forEach((f) => {
            (f[key] || []).forEach((r) => { segs.push({ s: r[0], e: r[1], f: f }); });
        });
        segs.sort((a, b) => { return a.s - b.s; });

        const wrap = el("div", { class: "idmap-lane" });
        if (!segs.length) {
            wrap.appendChild(el("div", { class: "lane-title", text: title }));
            wrap.appendChild(el("div", { class: "empty", text: "No id data available." }));
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
        const readout = el("span", { class: "count-note mono" });
        const controls = el("div", { class: "idmap-controls" }, [
            el("div", { class: "lane-title", text: `${title  } · ${  fillers.length  } fillers` }),
            el("span", { class: "spring" }),
            el("button", { class: "btn ghost", title: "Zoom out", text: "−", onclick: () => { zoomAt(0.5, 1.6); } }),
            el("button", { class: "btn ghost", title: "Zoom in", text: "+", onclick: () => { zoomAt(0.5, 0.625); } }),
            el("button", { class: "btn ghost", text: "Reset", onclick: () => { vs = gmin; ve = gmax + 1; schedule(); } }),
            readout
        ]);
        wrap.appendChild(controls);
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
                ctx.fillStyle = uColor(sg.f.universe_id);
                ctx.fillRect(x0, laneTop, Math.max(1, x1 - x0), laneH);
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
                    ctx.fillStyle = cSurface; ctx.font = `10px ${  cs.getPropertyValue("--font-mono") || "monospace"}`;
                    ctx.textAlign = "center"; ctx.textBaseline = "middle";
                    for (let k2 = startId; k2 <= endId; k2++) {
                        if (segAt(k2)) ctx.fillText(String(k2), X(k2) + ppid / 2, laneTop + laneH / 2);
                    }
                }
            }

            // Bottom axis with id value labels.
            ctx.fillStyle = cText3; ctx.strokeStyle = cBorder; ctx.lineWidth = 1;
            ctx.font = `11px ${  cs.getPropertyValue("--font-mono") || "monospace"}`;
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
            readout.textContent = `ids ${  fmt(Math.floor(vs))  }–${  fmt(Math.ceil(ve - 1)) 
                } · ${  fmt(used)  } used across ${  fmt(domSpan)  } (${  util  }% dense)`;
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
                showTip(`<b>${  esc(seg.f.name)  }</b><br>id ${  fmt(id)  } · u${  seg.f.universe_id 
                    }<br>run ${  fmt(seg.s)  }–${  fmt(seg.e)  } (${  fmt(seg.e - seg.s + 1)  } ids)`, e.clientX, e.clientY);
                canvas.style.cursor = "pointer";
            } else { hideTip(); canvas.style.cursor = "grab"; }
            schedule();
        });
        canvas.addEventListener("pointerup", (e) => {
            dragging = false;
            if (!moved) {
                const rect = canvas.getBoundingClientRect();
                const seg = segAt(Math.floor(idAtPx(e.clientX - rect.left, canvas.clientWidth)));
                if (seg) { hideTip(); openFiller(seg.f.name); }
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

        function cellVal(f, c) { return c.meta ? metaGet(f, c.k) : f[c.k]; }

        const thead = el("thead"), htr = el("tr");
        cols.forEach((c) => {
            const th = el("th", { class: c.num ? "num" : "" }, [
                document.createTextNode(c.t), el("span", { class: "arrow", text: " " })
            ]);
            th.addEventListener("click", () => {
                if (fillerSort.key === c.k) fillerSort.dir *= -1; else { fillerSort.key = c.k; fillerSort.dir = 1; }
                paint(curQ);
            });
            htr.appendChild(th);
        });
        thead.appendChild(htr);
        const table = el("table", { class: "data" }, [thead, tbody]);

        const colByKey = {}; cols.forEach((c) => { colByKey[c.k] = c; });

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
            note.textContent = `${rows.length  } / ${  fillers.length  } fillers`;
            rows.forEach((f) => {
                const tr = el("tr", { "data-name": f.name });
                tr.appendChild(el("td", { class: "name" }, [
                    el("span", { class: "dot-swatch", style: `background:${  uColor(f.universe_id)  };margin-right:7px` }),
                    document.createTextNode(f.name)
                ]));
                tr.appendChild(el("td", { class: "num", text: fmt(f.universe_id) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.envelope_count) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.cell_count) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.surface_count) }));
                fillerMetaKeys.forEach((k) => {
                    const v = metaGet(f, k);
                    tr.appendChild(el("td", { class: "muted", text: v == null ? "—" : metaVal(v) }));
                });
                tr.addEventListener("click", () => { openFiller(f.name); });
                tbody.appendChild(tr);
            });
            paintSortArrows(htr, cols);
        }
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Filler models" }), el("span", { class: "spring" }), note]),
            el("div", { class: "card-body" }, searchBox("Filter fillers…", paint)),
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
    RENDER.data = (root) => {
        clear(root);
        const grid = el("div", { class: "grid", style: "grid-template-columns:repeat(auto-fit,minmax(300px,1fr))" });
        root.appendChild(grid);
        grid.appendChild(listCard("Materials", DATA.materials || []));
        grid.appendChild(listCard("Tallies", DATA.tallies || []));
        grid.appendChild(listCard("Transforms", DATA.transforms || []));
        grid.appendChild(listCard("Source", DATA.source ? [DATA.source] : []));
    };
    function listCard(title, items) {
        const ul = el("div", {});
        function paint(q) {
            clear(ul);
            const f = items.filter((x) => { return !q || x.toLowerCase().indexOf(q) >= 0; });
            if (!f.length) { ul.appendChild(el("div", { class: "empty", text: "None" })); return; }
            f.forEach((x) => {
                ul.appendChild(el("div", { class: "leaf" }, [
                    el("span", { class: "dot-swatch", style: "background:var(--accent)" }),
                    el("span", { class: "en", text: x })
                ]));
            });
        }
        const body = el("div", { class: "card-body section-gap" });
        if (items.length > 8) body.appendChild(searchBox(`Filter ${  title  }…`, paint));
        body.appendChild(ul);
        paint("");
        return el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h3", { text: title }), el("span", { class: "spring" }), el("span", { class: "chip", text: String(items.length) })]),
            body
        ]);
    }

    // ── Diff ─────────────────────────────────────────────────────────────────
    RENDER.diff = (root) => {
        clear(root);
        const out = el("div", { class: "section-gap" });
        const dz = el("div", { class: "dropzone" }, [
            el("div", { style: "font-weight:700;margin-bottom:6px", text: "Compare against another build" }),
            el("div", { text: "Drop a build_report.json here, or click to choose a file." })
        ]);
        const fileInp = el("input", { type: "file", accept: "application/json,.json", class: "hidden" });
        dz.addEventListener("click", () => { fileInp.click(); });
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
                catch (err) { clear(out); out.appendChild(el("div", { class: "empty", text: `Could not parse JSON: ${  err}` })); }
            };
            fr.readAsText(file);
        }
        function renderDiff(other) {
            clear(out);
            const oEnv = {}; (other.envelope_entries || []).forEach((e) => { oEnv[e.envelope_name] = e; });
            const cEnv = {}; envelopes.forEach((e) => { cEnv[e.envelope_name] = e; });
            const added = [], removed = [], changed = [];
            envelopes.forEach((e) => {
                const o = oEnv[e.envelope_name];
                if (!o) added.push(e);
                else if ((o.filler_name || null) !== (e.filler_name || null) || (o.transform || null) !== (e.transform || null))
                    changed.push({ name: e.envelope_name, from: o.filler_name || "null", to: e.filler_name || "null", tf: `${o.transform || ""  }→${  e.transform || ""}` });
            });
            (other.envelope_entries || []).forEach((e) => { if (!cEnv[e.envelope_name]) removed.push(e); });

            out.appendChild(el("div", { class: "grid kpis" }, [
                kpi(added.length, "Added envelopes"),
                kpi(removed.length, "Removed envelopes"),
                kpi(changed.length, "Changed assignments")
            ]));
            out.children[0].children[0].querySelector(".v").style.color = "var(--ok)";
            out.children[0].children[1].querySelector(".v").style.color = "var(--danger)";
            out.children[0].children[2].querySelector(".v").style.color = "var(--warn)";

            if (changed.length) {
                const tb = el("tbody");
                changed.forEach((c) => {
                    tb.appendChild(el("tr", {}, [
                        el("td", { class: "name", text: c.name }),
                        el("td", { class: "diff-del", text: c.from }),
                        el("td", { text: "→" }),
                        el("td", { class: "diff-add", text: c.to })
                    ]));
                });
                out.appendChild(el("div", { class: "card" }, [
                    el("div", { class: "card-head" }, el("h3", { text: "Changed assignments (this build ← other)" })),
                    el("div", { class: "tablewrap" }, el("table", { class: "data" }, [
                        el("thead", {}, el("tr", {}, [el("th", { text: "Envelope" }), el("th", { text: "Other" }), el("th", {}), el("th", { text: "This build" })])),
                        tb
                    ]))
                ]));
            }
            [["Added in this build", added, "tag-added"], ["Removed (only in other)", removed, "tag-removed"]].forEach((grp) => {
                if (!grp[1].length) return;
                const chips = el("div", { class: "chips" });
                grp[1].forEach((e) => { chips.appendChild(el("span", { class: `chip ${  grp[2]}`, text: e.envelope_name })); });
                out.appendChild(el("div", { class: "card" }, [el("div", { class: "card-head" }, el("h3", { text: grp[0] })), el("div", { class: "card-body" }, chips)]));
            });
            if (!added.length && !removed.length && !changed.length)
                out.appendChild(el("div", { class: "empty", text: "No differences in envelope assignments." }));
        }
        root.appendChild(el("div", { class: "card" }, el("div", { class: "card-body" }, dz)));
        root.appendChild(fileInp);
        root.appendChild(out);
    };

    // ── Filler detail drawer ─────────────────────────────────────────────────
    const scrim = el("div", { class: "scrim", onclick: closeDrawer });
    const drawer = el("div", { class: "drawer" });
    document.body.appendChild(scrim);
    document.body.appendChild(drawer);
    document.addEventListener("keydown", (e) => { if (e.key === "Escape") closeDrawer(); });

    function openFiller(name) {
        const f = fillerByName[name];
        if (!f) return;
        clear(drawer);
        const desc = descOf(f);
        drawer.appendChild(el("div", { class: "drawer-head" }, [
            el("span", { class: "dot-swatch", style: `background:${  uColor(f.universe_id)  };width:16px;height:16px;margin-top:4px` }),
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
        const cr = idExtent(f.cell_id_runs), sr = idExtent(f.surface_id_runs);
        if (cr) row("Cell ids", `${fmt(cr[0])  } – ${  fmt(cr[1])  } · ${  f.cell_id_runs.length  } run${  f.cell_id_runs.length > 1 ? "s" : ""}`);
        if (sr) row("Surface ids", `${fmt(sr[0])  } – ${  fmt(sr[1])  } · ${  f.surface_id_runs.length  } run${  f.surface_id_runs.length > 1 ? "s" : ""}`);
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
            body.appendChild(el("div", { class: "subhead", text: `Materials (${  f.materials.length  })` }));
            const mc = el("div", { class: "chips" });
            f.materials.forEach((m) => { mc.appendChild(el("span", { class: "chip", text: `mat ${  m}` })); });
            body.appendChild(mc);
        }

        const filledEnvs = envelopes.filter((e) => { return e.filler_name === name; });
        if (filledEnvs.length) {
            body.appendChild(el("div", { class: "subhead", text: `Fills ${  filledEnvs.length  } envelope${  filledEnvs.length > 1 ? "s" : ""}` }));
            const list = el("div", {});
            filledEnvs.forEach((e) => {
                const r = el("div", { class: "leaf", style: "padding-left:8px" }, [
                    el("span", { class: "en", text: e.envelope_name })
                ]);
                if (e.transform) r.appendChild(el("span", { class: "chip", text: e.transform }));
                const d = descOf(e);
                if (d) r.appendChild(el("span", { class: "desc", text: d }));
                list.appendChild(r);
            });
            body.appendChild(list);
        }
        drawer.appendChild(body);
        scrim.classList.add("open");
        drawer.classList.add("open");
        setHash({ filler: name });
    }
    function closeDrawer() {
        scrim.classList.remove("open");
        drawer.classList.remove("open");
        setHash({ filler: null });
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
    })();
})();
