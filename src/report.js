/* ============================================================================
   gitronics build report — interactive viewer
   Vanilla JS, zero dependencies, fully offline. Hydrates the UI from the
   JSON manifest embedded in <script id="report-data">.
   All user-derived strings are inserted via textContent → no HTML injection.
   ========================================================================== */
(function () {
    "use strict";

    // ── Load data ──────────────────────────────────────────────────────────
    var DATA;
    try {
        DATA = JSON.parse(document.getElementById("report-data").textContent);
    } catch (e) {
        document.body.textContent = "Failed to parse report data: " + e;
        return;
    }

    var envelopes = DATA.envelope_entries || [];
    var fillers = DATA.filler_entries || [];
    var fillerByName = {};
    fillers.forEach(function (f) { fillerByName[f.name] = f; });

    // ── DOM helpers ────────────────────────────────────────────────────────
    function el(tag, props, kids) {
        var n = document.createElement(tag);
        if (props) {
            for (var k in props) {
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
        if (Array.isArray(kids)) kids.forEach(function (c) { append(n, c); });
        else if (kids instanceof Node) n.appendChild(kids);
        else if (kids != null) n.appendChild(document.createTextNode(String(kids)));
    }
    function svgEl(tag, props) {
        var n = document.createElementNS("http://www.w3.org/2000/svg", tag);
        if (props) for (var k in props) if (props[k] != null) n.setAttribute(k, props[k]);
        return n;
    }
    function clear(n) { while (n.firstChild) n.removeChild(n.firstChild); }
    function fmt(x) { return (x == null ? "—" : Number(x).toLocaleString("en-US")); }
    var $ = function (s, r) { return (r || document).querySelector(s); };

    // Deterministic color from a universe id.
    function uHue(id) {
        var h = 0, s = String(id);
        for (var i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) >>> 0;
        return h % 360;
    }
    function uColor(id) {
        if (id == null) return "var(--null)";
        var dark = document.documentElement.getAttribute("data-theme") === "dark";
        return "hsl(" + uHue(id) + " " + (dark ? 55 : 62) + "% " + (dark ? 55 : 52) + "%)";
    }

    // ── Icons (inline SVG paths) ───────────────────────────────────────────
    function icon(name) {
        var p = {
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
        var s = svgEl("svg", { viewBox: "0 0 24 24", width: 16, height: 16, fill: "none", stroke: "currentColor", "stroke-width": 2, "stroke-linecap": "round", "stroke-linejoin": "round" });
        s.appendChild(svgEl("path", { d: p }));
        return s;
    }

    // ── Tooltip ────────────────────────────────────────────────────────────
    var tip = el("div", { id: "tooltip" });
    document.body.appendChild(tip);
    function showTip(html, x, y) {
        tip.innerHTML = html;
        tip.classList.add("show");
        var w = tip.offsetWidth, h = tip.offsetHeight;
        tip.style.left = Math.min(x + 14, window.innerWidth - w - 8) + "px";
        tip.style.top = Math.max(8, y - h - 12) + "px";
    }
    function hideTip() { tip.classList.remove("show"); }

    // ── Derived stats ──────────────────────────────────────────────────────
    var nFilled = envelopes.filter(function (e) { return e.filler_name; }).length;
    var nNull = envelopes.length - nFilled;
    var coverage = envelopes.length ? Math.round((nFilled / envelopes.length) * 100) : 0;
    var nData = (DATA.materials || []).length + (DATA.tallies || []).length +
        (DATA.transforms || []).length + (DATA.source ? 1 : 0);

    // ── App shell ──────────────────────────────────────────────────────────
    var app = el("div");
    document.body.appendChild(app);

    // ── Tabs (defined before the header, which mounts them) ────────────────
    var TABS = [
        { id: "overview", label: "Overview", icon: "layers" },
        { id: "explorer", label: "Explorer", icon: "tree", pill: envelopes.length },
        { id: "coverage", label: "Coverage Map", icon: "grid" },
        { id: "idmap", label: "ID Map", icon: "map" },
        { id: "fillers", label: "Fillers", icon: "box", pill: fillers.length },
        { id: "data", label: "Data Cards", icon: "layers", pill: nData },
        { id: "diff", label: "Diff", icon: "diff" }
    ];
    var tabBtns = {};
    function buildTabs() {
        var nav = el("nav", { class: "tabs" });
        TABS.forEach(function (t) {
            var b = el("button", { class: "tab", onclick: function () { go(t.id); } }, [
                icon(t.icon), document.createTextNode(t.label)
            ]);
            if (t.pill != null) b.appendChild(el("span", { class: "pill", text: String(t.pill) }));
            tabBtns[t.id] = b;
            nav.appendChild(b);
        });
        return nav;
    }

    // Header
    var themeBtn = el("button", { class: "iconbtn", title: "Toggle theme", "aria-label": "Toggle theme", onclick: toggleTheme });
    var exportBtn = el("button", { class: "iconbtn", title: "Download report data (JSON)", "aria-label": "Download JSON", onclick: downloadJson });
    exportBtn.appendChild(icon("download"));
    function paintThemeIcon() {
        clear(themeBtn);
        themeBtn.appendChild(icon(document.documentElement.getAttribute("data-theme") === "dark" ? "sun" : "moon"));
    }

    var header = el("header", { class: "topbar" }, el("div", { class: "wrap" }, [
        el("div", { class: "topbar-inner" }, [
            el("div", { class: "brand" }, [
                el("span", { class: "logo", text: "◆" }),
                el("span", { text: "gitronics" }),
                el("span", { class: "ver", text: "v" + DATA.gitronics_version })
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

    var main = el("main", {}, el("div", { class: "wrap", id: "panels" }));
    app.appendChild(main);
    var panels = $("#panels", main);

    app.appendChild(el("footer", { class: "foot" }, [
        document.createTextNode("Generated by "),
        el("a", { href: "https://fusion4energy.github.io/gitronics/latest", text: "gitronics" }),
        document.createTextNode(" · schema v" + (DATA.schema_version || 1))
    ]));

    // ── Tabs (mounting handled by buildTabs, defined above) ────────────────
    var built = {};
    var panelEls = {};
    function go(id) {
        if (!tabBtns[id]) id = "overview";
        for (var k in tabBtns) tabBtns[k].classList.toggle("active", k === id);
        for (var p in panelEls) panelEls[p].classList.toggle("active", p === id);
        if (!panelEls[id]) {
            var pane = el("section", { class: "panel", id: "panel-" + id });
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
        var o = {};
        location.hash.replace(/^#/, "").split("&").forEach(function (kv) {
            var i = kv.indexOf("="); if (i > 0) o[kv.slice(0, i)] = decodeURIComponent(kv.slice(i + 1));
        });
        return o;
    }
    function setHash(patch) {
        var o = getHash();
        for (var k in patch) { if (patch[k] == null) delete o[k]; else o[k] = patch[k]; }
        var s = Object.keys(o).map(function (k) { return k + "=" + encodeURIComponent(o[k]); }).join("&");
        history.replaceState(null, "", "#" + s);
    }

    // ── Theme ────────────────────────────────────────────────────────────────
    function toggleTheme() {
        var d = document.documentElement.getAttribute("data-theme") === "dark";
        var next = d ? "light" : "dark";
        document.documentElement.setAttribute("data-theme", next);
        try { localStorage.setItem("gitronics-theme", next); } catch (e) { }
        paintThemeIcon();
        // re-render open dynamic panels that depend on theme colors
        ["coverage", "idmap", "overview"].forEach(function (id) {
            if (built[id]) { clear(panelEls[id]); RENDER[id](panelEls[id]); }
        });
    }

    function downloadJson() {
        var blob = new Blob([JSON.stringify(DATA, null, 2)], { type: "application/json" });
        var a = el("a", { href: URL.createObjectURL(blob), download: "build_report.json" });
        document.body.appendChild(a); a.click(); a.remove();
    }

    // ── Search box widget ────────────────────────────────────────────────────
    function searchBox(placeholder, oninput) {
        var box = el("div", { class: "searchbox" });
        box.appendChild(icon("search"));
        var inp = el("input", { class: "input", type: "text", placeholder: placeholder, oninput: function () { oninput(this.value.toLowerCase().trim()); } });
        box.appendChild(inp);
        return box;
    }

    // ── KPI cards ────────────────────────────────────────────────────────────
    function kpi(v, label, sub, spark) {
        var k = el("div", { class: "kpi" }, [
            el("div", { class: "v", text: fmt(v) }),
            el("div", { class: "l", text: label })
        ]);
        if (sub) k.appendChild(el("div", { class: "s", text: sub }));
        if (spark) k.appendChild(spark);
        return k;
    }
    function donutMini(pct) {
        var s = svgEl("svg", { class: "spark", width: 34, height: 34, viewBox: "0 0 36 36" });
        var bg = svgEl("circle", { cx: 18, cy: 18, r: 15, fill: "none", stroke: "var(--surface-3)", "stroke-width": 5 });
        var c = 2 * Math.PI * 15;
        var fg = svgEl("circle", { cx: 18, cy: 18, r: 15, fill: "none", stroke: "var(--ok)", "stroke-width": 5, "stroke-linecap": "round", "stroke-dasharray": (c * pct / 100) + " " + c, transform: "rotate(-90 18 18)" });
        s.appendChild(bg); s.appendChild(fg);
        return s;
    }

    // ══════════════════════════════════════════════════════════════════════
    //  RENDERERS
    // ══════════════════════════════════════════════════════════════════════
    var RENDER = {};

    // ── Overview ─────────────────────────────────────────────────────────────
    RENDER.overview = function (root) {
        clear(root);
        var wrap = el("div", { class: "section-gap" });
        root.appendChild(wrap);

        // KPIs
        var kpis = el("div", { class: "grid kpis" }, [
            kpi(DATA.total_cells, "Cells"),
            kpi(DATA.total_surfaces, "Surfaces"),
            kpi(envelopes.length, "Envelopes", nFilled + " filled · " + nNull + " null"),
            kpi(fillers.length, "Filler Models"),
            kpi(coverage + "%", "Fill Coverage", null, donutMini(coverage)),
            kpi(nData, "Data Files")
        ]);
        // coverage % value shows literal string; fix formatting
        kpis.children[4].querySelector(".v").textContent = coverage + "%";
        wrap.appendChild(kpis);

        var cols = el("div", { class: "grid", style: "grid-template-columns:repeat(auto-fit,minmax(320px,1fr))" });
        wrap.appendChild(cols);

        // Coverage donut card
        cols.appendChild(card("Fill coverage", donutCard()));

        // Top fillers by cells
        var topCells = fillers.slice().sort(function (a, b) { return b.cell_count - a.cell_count; }).slice(0, 10);
        cols.appendChild(card("Top fillers by cell count", barChart(topCells.map(function (f) {
            return { label: f.name, value: f.cell_count, id: f.universe_id, onclick: function () { openFiller(f.name); } };
        }))));

        // Most reused fillers
        var reused = fillers.slice().filter(function (f) { return f.envelope_count > 0; })
            .sort(function (a, b) { return b.envelope_count - a.envelope_count; }).slice(0, 10);
        cols.appendChild(card("Most reused fillers (envelopes filled)", barChart(reused.map(function (f) {
            return { label: f.name, value: f.envelope_count, id: f.universe_id, onclick: function () { openFiller(f.name); } };
        }))));

        // Materials usage
        var matCount = {};
        fillers.forEach(function (f) { (f.materials || []).forEach(function (m) { matCount[m] = (matCount[m] || 0) + 1; }); });
        var mats = Object.keys(matCount).map(function (m) { return { label: "mat " + m, value: matCount[m] }; })
            .sort(function (a, b) { return b.value - a.value; }).slice(0, 12);
        if (mats.length) cols.appendChild(card("Material usage (fillers per material)", barChart(mats)));

        // Per-zone rollup
        var zones = rollupZones();
        if (zones.length > 1 || (zones.length === 1 && zones[0].zone !== "—")) {
            cols.appendChild(card("Envelopes per zone", barChart(zones.map(function (z) {
                return { label: z.zone, value: z.total, sub: z.filled + "/" + z.total + " filled" };
            }))));
        }
    };

    function card(title, body) {
        return el("div", { class: "card" }, [
            el("div", { class: "card-head" }, el("h3", { text: title })),
            el("div", { class: "card-body" }, body)
        ]);
    }

    function donutCard() {
        var wrap = el("div", { class: "donut-wrap" });
        var size = 150, r = 60, cx = 75, cy = 75, sw = 22;
        var s = svgEl("svg", { width: size, height: size, viewBox: "0 0 150 150" });
        var segs = [
            { v: nFilled, c: "var(--ok)", label: "Filled" },
            { v: nNull, c: "var(--null)", label: "Null" }
        ].filter(function (x) { return x.v > 0; });
        var total = nFilled + nNull || 1, off = 0, circ = 2 * Math.PI * r;
        segs.forEach(function (seg) {
            var frac = seg.v / total;
            var c = svgEl("circle", {
                cx: cx, cy: cy, r: r, fill: "none", stroke: seg.c, "stroke-width": sw,
                "stroke-dasharray": (circ * frac) + " " + circ,
                "stroke-dashoffset": -off * circ,
                transform: "rotate(-90 " + cx + " " + cy + ")"
            });
            s.appendChild(c);
            off += frac;
        });
        s.appendChild(svgEl("circle", { cx: cx, cy: cy, r: r - sw / 2 - 2, fill: "var(--surface)" }));
        var t1 = svgEl("text", { x: cx, y: cy - 2, "text-anchor": "middle", "font-size": 26, "font-weight": 800, fill: "var(--text)" });
        t1.textContent = coverage + "%";
        var t2 = svgEl("text", { x: cx, y: cy + 18, "text-anchor": "middle", "font-size": 11, fill: "var(--text-3)" });
        t2.textContent = "filled";
        s.appendChild(t1); s.appendChild(t2);
        wrap.appendChild(s);
        var leg = el("div", {});
        leg.appendChild(legRow("var(--ok)", "Filled", nFilled));
        leg.appendChild(legRow("var(--null)", "Null", nNull));
        wrap.appendChild(leg);
        return wrap;
    }
    function legRow(color, label, val) {
        return el("div", { class: "legend", style: "margin:4px 0" }, el("div", { class: "item" }, [
            el("span", { class: "dot-swatch", style: "background:" + color }),
            el("span", { text: label + " · " + fmt(val) })
        ]));
    }

    function barChart(rows) {
        var max = rows.reduce(function (m, r) { return Math.max(m, r.value); }, 0) || 1;
        var box = el("div", {});
        if (!rows.length) { box.appendChild(el("div", { class: "empty", text: "No data" })); return box; }
        rows.forEach(function (r) {
            var fill = el("div", { class: "fill", style: "width:" + (r.value / max * 100) + "%" + (r.id != null ? ";background:" + uColor(r.id) : "") });
            var row = el("div", { class: "chart-bar-row" }, [
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

    function rollupZones() {
        var m = {};
        envelopes.forEach(function (e) {
            var z = e.zone || "—";
            if (!m[z]) m[z] = { zone: z, total: 0, filled: 0 };
            m[z].total++; if (e.filler_name) m[z].filled++;
        });
        return Object.keys(m).map(function (k) { return m[k]; }).sort(function (a, b) { return b.total - a.total; });
    }

    // ── Explorer (tree) ──────────────────────────────────────────────────────
    RENDER.explorer = function (root) {
        clear(root);
        var treeHost = el("div", { class: "tree" });
        var note = el("div", { class: "count-note" });
        var body = el("div", { class: "card-body section-gap" }, [
            el("div", { class: "toolbar" }, [
                searchBox("Filter by envelope, filler, description, universe…", function (q) { rebuild(q); }),
                note
            ]),
            treeHost
        ]);
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Model hierarchy" }),
            el("span", { class: "spring" }),
            el("span", { class: "chip ok", text: nFilled + " filled" }),
            el("span", { class: "chip null", text: nNull + " null" })]),
            body
        ]));

        function rebuild(q) {
            clear(treeHost);
            q = q || "";
            var groups = {};
            var shown = 0;
            envelopes.forEach(function (e) {
                if (q && matchEnv(e).indexOf(q) < 0) return;
                shown++;
                var z = e.zone || "Unzoned";
                var s = e.sector != null ? ("Sector " + e.sector) : "—";
                (groups[z] = groups[z] || {})[s] = (groups[z][s] || []);
                groups[z][s].push(e);
            });
            note.textContent = shown + " / " + envelopes.length + " envelopes";
            var zoneKeys = Object.keys(groups).sort();
            if (!zoneKeys.length) { treeHost.appendChild(el("div", { class: "empty", text: "No matches" })); return; }
            var autoOpen = zoneKeys.length <= 3 || !!q;
            zoneKeys.forEach(function (z) {
                var secKeys = Object.keys(groups[z]).sort();
                var zTotal = secKeys.reduce(function (a, s) { return a + groups[z][s].length; }, 0);
                var zDet = el("details", autoOpen ? { open: "" } : {});
                zDet.appendChild(el("summary", {}, [
                    el("span", { class: "tw", text: "▶" }),
                    el("span", { class: "grp-name", text: z }),
                    el("span", { class: "spring" }),
                    el("span", { class: "chip", text: zTotal })
                ]));
                secKeys.forEach(function (s) {
                    var list = groups[z][s];
                    var sDet = el("details", (autoOpen || secKeys.length === 1) ? { open: "" } : {});
                    sDet.appendChild(el("summary", {}, [
                        el("span", { class: "tw", text: "▶" }),
                        el("span", { class: "grp-name muted", text: s }),
                        el("span", { class: "spring" }),
                        el("span", { class: "chip", text: list.length })
                    ]));
                    list.forEach(function (e) { sDet.appendChild(leafRow(e)); });
                    zDet.appendChild(sDet);
                });
                treeHost.appendChild(zDet);
            });
        }
        rebuild(getHash().q || "");
    };

    function matchEnv(e) {
        return [e.envelope_name, e.filler_name, e.description, e.zone, e.sector, e.universe_id, e.transform]
            .filter(Boolean).join(" ").toLowerCase();
    }

    function leafRow(e) {
        var filled = !!e.filler_name;
        var leaf = el("div", { class: "leaf" });
        leaf.appendChild(el("span", { class: "dot-swatch", style: "background:" + (filled ? uColor(e.universe_id) : "var(--null)") }));
        leaf.appendChild(el("span", { class: "en", text: e.envelope_name }));
        if (filled) {
            leaf.appendChild(el("span", { class: "fl", text: "→ " + e.filler_name }));
            leaf.appendChild(el("span", { class: "chip accent", text: "u" + e.universe_id }));
            if (e.transform) leaf.appendChild(el("span", { class: "chip", text: e.transform }));
        } else {
            leaf.appendChild(el("span", { class: "chip null", text: "null" }));
        }
        leaf.appendChild(el("span", { class: "spring" }));
        if (e.description) leaf.appendChild(el("span", { class: "desc", text: e.description }));
        if (filled) leaf.addEventListener("click", function () { openFiller(e.filler_name); });
        return leaf;
    }

    // ── Coverage map ─────────────────────────────────────────────────────────
    RENDER.coverage = function (root) {
        clear(root);
        var host = el("div", {});
        var body = el("div", { class: "card-body section-gap" }, [
            el("div", { class: "legend" }, [
                el("span", { class: "item" }, [el("span", { class: "dot-swatch", style: "background:var(--null)" }), document.createTextNode("Null / empty")]),
                el("span", { class: "item muted", text: "Each square is an envelope, colored by the universe filling it. Hover for details, click to inspect the filler." })
            ]),
            host
        ]);
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Coverage map" }),
            el("span", { class: "spring" }),
            el("span", { class: "chip accent", text: coverage + "% filled" })]),
            body
        ]));

        var byZone = {};
        envelopes.forEach(function (e) {
            var z = e.zone || "Unzoned";
            (byZone[z] = byZone[z] || []).push(e);
        });
        Object.keys(byZone).sort().forEach(function (z) {
            var list = byZone[z].slice().sort(function (a, b) {
                var sa = (a.sector || "") + a.envelope_name, sb = (b.sector || "") + b.envelope_name;
                return sa < sb ? -1 : sa > sb ? 1 : 0;
            });
            var filled = list.filter(function (e) { return e.filler_name; }).length;
            var zwrap = el("div", { class: "cov-zone" });
            zwrap.appendChild(el("h4", {}, [
                el("span", { text: z }),
                el("span", { class: "chip", text: filled + "/" + list.length })
            ]));
            var grid = el("div", { class: "cov-grid" });
            list.forEach(function (e) {
                var filledCell = !!e.filler_name;
                var cell = el("div", {
                    class: "cov-cell",
                    style: "background:" + (filledCell ? uColor(e.universe_id) : "var(--null-weak)")
                });
                cell.addEventListener("mousemove", function (ev) {
                    showTip("<b>" + esc(e.envelope_name) + "</b>" +
                        (e.description ? "<br>" + esc(e.description) : "") +
                        (e.sector != null ? "<br>sector " + esc(e.sector) : "") +
                        "<br>" + (filledCell ? "→ " + esc(e.filler_name) + " (u" + e.universe_id + ")" : "null"),
                        ev.clientX, ev.clientY);
                });
                cell.addEventListener("mouseleave", hideTip);
                if (filledCell) cell.addEventListener("click", function () { hideTip(); openFiller(e.filler_name); });
                grid.appendChild(cell);
            });
            zwrap.appendChild(grid);
            host.appendChild(zwrap);
        });
    };

    function esc(s) { return String(s).replace(/[&<>"]/g, function (c) { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]; }); }

    // ── ID allocation map ────────────────────────────────────────────────────
    RENDER.idmap = function (root) {
        clear(root);
        var body = el("div", { class: "card-body section-gap" });
        root.appendChild(el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h2", { text: "Card-ID allocation map" }),
            el("span", { class: "spring" }),
            el("span", { class: "muted count-note", text: "Each bar spans a filler's [min…max] id range, colored by universe. Rows pack overlapping spans; the build guarantees the actual ids never collide." })]),
            body
        ]));
        body.appendChild(idLane("Cell ids", "cell_id_range"));
        body.appendChild(idLane("Surface ids", "surface_id_range"));
    };

    function idLane(title, key) {
        var items = fillers.filter(function (f) { return f[key]; })
            .map(function (f) { return { f: f, min: f[key].min, max: f[key].max }; })
            .sort(function (a, b) { return a.min - b.min || a.max - b.max; });
        var wrap = el("div", { class: "idmap" });
        wrap.appendChild(el("div", { class: "lane-title", text: title + " · " + items.length + " fillers" }));
        if (!items.length) { wrap.appendChild(el("div", { class: "empty", text: "No id-range data available." })); return wrap; }

        var gmin = items[0].min, gmax = items.reduce(function (m, it) { return Math.max(m, it.max); }, items[0].max);
        var span = (gmax - gmin) || 1;

        // Greedy row packing so overlapping spans stack into distinct rows.
        var rows = [];
        items.forEach(function (it) {
            var placed = false;
            for (var r = 0; r < rows.length; r++) {
                if (rows[r] < it.min) { it.row = r; rows[r] = it.max; placed = true; break; }
            }
            if (!placed) { it.row = rows.length; rows.push(it.max); }
        });

        var W = 1000, rowH = 22, gap = 4, padL = 4, padR = 4, top = 6;
        var H = top + rows.length * (rowH + gap) + 26;
        var svg = svgEl("svg", { viewBox: "0 0 " + W + " " + H, preserveAspectRatio: "none", height: H });
        function X(v) { return padL + (v - gmin) / span * (W - padL - padR); }

        items.forEach(function (it, i) {
            var x = X(it.min), w = Math.max(2, X(it.max) - x), y = top + it.row * (rowH + gap);
            var g = svgEl("g", { class: "bar" });
            var rect = svgEl("rect", {
                x: x, y: y, width: w, height: rowH, rx: 4,
                fill: uColor(it.f.universe_id),
                "fill-opacity": 0.85, stroke: "var(--surface)", "stroke-width": 0.5
            });
            g.appendChild(rect);
            if (w > 46) {
                var t = svgEl("text", { x: x + 5, y: y + rowH / 2 + 4, "font-size": 11, fill: "#fff", "font-family": "var(--font-mono)" });
                t.textContent = it.f.name.replace(/^universe_/, "u");
                g.appendChild(t);
            }
            g.addEventListener("mousemove", function (ev) {
                showTip("<b>" + esc(it.f.name) + "</b><br>" + title.toLowerCase() + ": " + fmt(it.min) + " – " + fmt(it.max) +
                    "<br>" + fmt(it.max - it.min + 1) + " ids · u" + it.f.universe_id,
                    ev.clientX, ev.clientY);
            });
            g.addEventListener("mouseleave", hideTip);
            g.addEventListener("click", function () { hideTip(); openFiller(it.f.name); });
            svg.appendChild(g);
        });
        // axis
        [0, 0.25, 0.5, 0.75, 1].forEach(function (t) {
            var v = Math.round(gmin + t * span), x = X(v);
            svg.appendChild(svgEl("line", { x1: x, y1: top, x2: x, y2: H - 22, stroke: "var(--border)", "stroke-width": 0.5, "stroke-dasharray": "2 3" }));
            var lab = svgEl("text", { x: Math.min(Math.max(x, 20), W - 20), y: H - 6, "text-anchor": "middle", "font-size": 11, fill: "var(--text-3)", "font-family": "var(--font-mono)" });
            lab.textContent = fmt(v);
            svg.appendChild(lab);
        });
        wrap.appendChild(svg);
        return wrap;
    }

    // ── Fillers table ────────────────────────────────────────────────────────
    var fillerSort = { key: "cell_count", dir: -1 };
    RENDER.fillers = function (root) {
        clear(root);
        var tbody = el("tbody");
        var note = el("div", { class: "count-note" });
        var cols = [
            { k: "name", t: "Filler", cls: "name" },
            { k: "universe_id", t: "Universe", cls: "num" },
            { k: "envelope_count", t: "Envelopes", cls: "num" },
            { k: "cell_count", t: "Cells", cls: "num" },
            { k: "surface_count", t: "Surfaces", cls: "num" },
            { k: "pbs", t: "PBS", cls: "" },
            { k: "description", t: "Description", cls: "muted" }
        ];
        var thead = el("thead"), htr = el("tr");
        cols.forEach(function (c) {
            var th = el("th", { class: c.cls === "num" ? "num" : "" }, [
                document.createTextNode(c.t), el("span", { class: "arrow", text: " " })
            ]);
            th.addEventListener("click", function () {
                if (fillerSort.key === c.k) fillerSort.dir *= -1; else { fillerSort.key = c.k; fillerSort.dir = 1; }
                paint(curQ);
            });
            htr.appendChild(th);
        });
        thead.appendChild(htr);
        var table = el("table", { class: "data" }, [thead, tbody]);

        var curQ = "";
        function paint(q) {
            curQ = q || "";
            clear(tbody);
            var rows = fillers.filter(function (f) {
                if (!curQ) return true;
                return [f.name, f.universe_id, f.pbs, f.description].filter(Boolean).join(" ").toLowerCase().indexOf(curQ) >= 0;
            });
            var key = fillerSort.key, dir = fillerSort.dir;
            rows.sort(function (a, b) {
                var av = a[key], bv = b[key];
                if (av == null) av = typeof bv === "number" ? -Infinity : "";
                if (bv == null) bv = typeof av === "number" ? -Infinity : "";
                if (typeof av === "string" || typeof bv === "string") return String(av).localeCompare(String(bv)) * dir;
                return (av - bv) * dir;
            });
            note.textContent = rows.length + " / " + fillers.length + " fillers";
            rows.forEach(function (f) {
                var tr = el("tr", { "data-name": f.name });
                tr.appendChild(el("td", { class: "name" }, [
                    el("span", { class: "dot-swatch", style: "background:" + uColor(f.universe_id) + ";margin-right:7px" }),
                    document.createTextNode(f.name)
                ]));
                tr.appendChild(el("td", { class: "num", text: fmt(f.universe_id) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.envelope_count) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.cell_count) }));
                tr.appendChild(el("td", { class: "num", text: fmt(f.surface_count) }));
                tr.appendChild(el("td", { text: f.pbs || "—" }));
                tr.appendChild(el("td", { class: "muted", text: f.description || "—" }));
                tr.addEventListener("click", function () { openFiller(f.name); });
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
        cols.forEach(function (c, i) {
            var a = htr.children[i].querySelector(".arrow");
            a.textContent = fillerSort.key === c.k ? (fillerSort.dir > 0 ? " ▲" : " ▼") : "";
        });
    }

    // ── Data cards ───────────────────────────────────────────────────────────
    RENDER.data = function (root) {
        clear(root);
        var grid = el("div", { class: "grid", style: "grid-template-columns:repeat(auto-fit,minmax(300px,1fr))" });
        root.appendChild(grid);
        grid.appendChild(listCard("Materials", DATA.materials || []));
        grid.appendChild(listCard("Tallies", DATA.tallies || []));
        grid.appendChild(listCard("Transforms", DATA.transforms || []));
        grid.appendChild(listCard("Source", DATA.source ? [DATA.source] : []));
    };
    function listCard(title, items) {
        var ul = el("div", {});
        function paint(q) {
            clear(ul);
            var f = items.filter(function (x) { return !q || x.toLowerCase().indexOf(q) >= 0; });
            if (!f.length) { ul.appendChild(el("div", { class: "empty", text: "None" })); return; }
            f.forEach(function (x) {
                ul.appendChild(el("div", { class: "leaf" }, [
                    el("span", { class: "dot-swatch", style: "background:var(--accent)" }),
                    el("span", { class: "en", text: x })
                ]));
            });
        }
        var body = el("div", { class: "card-body section-gap" });
        if (items.length > 8) body.appendChild(searchBox("Filter " + title + "…", paint));
        body.appendChild(ul);
        paint("");
        return el("div", { class: "card" }, [
            el("div", { class: "card-head" }, [el("h3", { text: title }), el("span", { class: "spring" }), el("span", { class: "chip", text: String(items.length) })]),
            body
        ]);
    }

    // ── Diff ─────────────────────────────────────────────────────────────────
    RENDER.diff = function (root) {
        clear(root);
        var out = el("div", { class: "section-gap" });
        var dz = el("div", { class: "dropzone" }, [
            el("div", { style: "font-weight:700;margin-bottom:6px", text: "Compare against another build" }),
            el("div", { text: "Drop a build_report.json here, or click to choose a file." })
        ]);
        var fileInp = el("input", { type: "file", accept: "application/json,.json", class: "hidden" });
        dz.addEventListener("click", function () { fileInp.click(); });
        dz.addEventListener("dragover", function (e) { e.preventDefault(); dz.classList.add("hot"); });
        dz.addEventListener("dragleave", function () { dz.classList.remove("hot"); });
        dz.addEventListener("drop", function (e) {
            e.preventDefault(); dz.classList.remove("hot");
            if (e.dataTransfer.files[0]) readFile(e.dataTransfer.files[0]);
        });
        fileInp.addEventListener("change", function () { if (this.files[0]) readFile(this.files[0]); });
        function readFile(file) {
            var fr = new FileReader();
            fr.onload = function () {
                try { renderDiff(JSON.parse(fr.result)); }
                catch (err) { clear(out); out.appendChild(el("div", { class: "empty", text: "Could not parse JSON: " + err })); }
            };
            fr.readAsText(file);
        }
        function renderDiff(other) {
            clear(out);
            var oEnv = {}; (other.envelope_entries || []).forEach(function (e) { oEnv[e.envelope_name] = e; });
            var cEnv = {}; envelopes.forEach(function (e) { cEnv[e.envelope_name] = e; });
            var added = [], removed = [], changed = [];
            envelopes.forEach(function (e) {
                var o = oEnv[e.envelope_name];
                if (!o) added.push(e);
                else if ((o.filler_name || null) !== (e.filler_name || null) || (o.transform || null) !== (e.transform || null))
                    changed.push({ name: e.envelope_name, from: o.filler_name || "null", to: e.filler_name || "null", tf: (o.transform || "") + "→" + (e.transform || "") });
            });
            (other.envelope_entries || []).forEach(function (e) { if (!cEnv[e.envelope_name]) removed.push(e); });

            out.appendChild(el("div", { class: "grid kpis" }, [
                kpi(added.length, "Added envelopes"),
                kpi(removed.length, "Removed envelopes"),
                kpi(changed.length, "Changed assignments")
            ]));
            out.children[0].children[0].querySelector(".v").style.color = "var(--ok)";
            out.children[0].children[1].querySelector(".v").style.color = "var(--danger)";
            out.children[0].children[2].querySelector(".v").style.color = "var(--warn)";

            if (changed.length) {
                var tb = el("tbody");
                changed.forEach(function (c) {
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
            [["Added in this build", added, "tag-added"], ["Removed (only in other)", removed, "tag-removed"]].forEach(function (grp) {
                if (!grp[1].length) return;
                var chips = el("div", { class: "chips" });
                grp[1].forEach(function (e) { chips.appendChild(el("span", { class: "chip " + grp[2], text: e.envelope_name })); });
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
    var scrim = el("div", { class: "scrim", onclick: closeDrawer });
    var drawer = el("div", { class: "drawer" });
    document.body.appendChild(scrim);
    document.body.appendChild(drawer);
    document.addEventListener("keydown", function (e) { if (e.key === "Escape") closeDrawer(); });

    function openFiller(name) {
        var f = fillerByName[name];
        if (!f) return;
        clear(drawer);
        drawer.appendChild(el("div", { class: "drawer-head" }, [
            el("span", { class: "dot-swatch", style: "background:" + uColor(f.universe_id) + ";width:16px;height:16px;margin-top:4px" }),
            el("div", { class: "spring" }, [
                el("h3", { text: f.name }),
                f.description ? el("div", { class: "muted", style: "font-size:.84rem;margin-top:2px", text: f.description }) : null
            ]),
            el("button", { class: "iconbtn", title: "Close", onclick: closeDrawer, text: "✕" })
        ]));
        var body = el("div", { class: "drawer-body" });
        var kv = el("dl", { class: "kv" });
        function row(k, v) { kv.appendChild(el("dt", { text: k })); kv.appendChild(el("dd", { text: v })); }
        row("Universe id", String(f.universe_id));
        row("Envelopes filled", String(f.envelope_count));
        row("Cells", fmt(f.cell_count));
        row("Surfaces", fmt(f.surface_count));
        if (f.pbs) row("PBS", f.pbs);
        if (f.cell_id_range) row("Cell id range", fmt(f.cell_id_range.min) + " – " + fmt(f.cell_id_range.max));
        if (f.surface_id_range) row("Surface id range", fmt(f.surface_id_range.min) + " – " + fmt(f.surface_id_range.max));
        body.appendChild(kv);

        if (f.materials && f.materials.length) {
            body.appendChild(el("div", { class: "subhead", text: "Materials (" + f.materials.length + ")" }));
            var mc = el("div", { class: "chips" });
            f.materials.forEach(function (m) { mc.appendChild(el("span", { class: "chip", text: "mat " + m })); });
            body.appendChild(mc);
        }

        var filledEnvs = envelopes.filter(function (e) { return e.filler_name === name; });
        if (filledEnvs.length) {
            body.appendChild(el("div", { class: "subhead", text: "Fills " + filledEnvs.length + " envelope" + (filledEnvs.length > 1 ? "s" : "") }));
            var list = el("div", {});
            filledEnvs.forEach(function (e) {
                var r = el("div", { class: "leaf", style: "padding-left:8px" }, [
                    el("span", { class: "en", text: e.envelope_name })
                ]);
                if (e.transform) r.appendChild(el("span", { class: "chip", text: e.transform }));
                if (e.zone) r.appendChild(el("span", { class: "chip null", text: e.zone }));
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
            var saved = localStorage.getItem("gitronics-theme");
            if (saved) document.documentElement.setAttribute("data-theme", saved);
            else if (window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches)
                document.documentElement.setAttribute("data-theme", "dark");
        } catch (e) { }
        paintThemeIcon();
        var h = getHash();
        go(h.tab || "overview");
        if (h.filler) setTimeout(function () { openFiller(h.filler); }, 60);
    })();
})();
