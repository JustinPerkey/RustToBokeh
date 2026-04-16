// ── Embedded CSS ──────────────────────────────────────────────────────────────

pub const INLINE_CSS: &str = r#"<style>
        :root {
            --s-1: 0.25rem; --s-2: 0.5rem; --s-3: 0.75rem; --s-4: 1rem;
            --s-5: 1.5rem;  --s-6: 2rem;   --s-7: 3rem;    --s-8: 4rem;
            --fs-cap: 0.67em; --fs-xs: 0.72em; --fs-sm: 0.83em;
            --fs-base: 0.9em; --fs-md: 0.95em; --fs-lg: 1em;
            --c-primary: #4C72B0; --c-primary-deep: #3a5a8f; --c-primary-tint: #f0f4fb; --c-primary-border: #a0b8d8; --c-primary-pale: #e8eef7;
            --c-accent: #E8923C; --c-accent-pale: #fef4e8; --c-accent-deep: #c47420;
            --c-success: #2F9E6D; --c-warning: #D97706; --c-error: #D94747; --c-info: #4C8FBF;
            --c-text-dark: #2c3e50; --c-text: #333; --c-text-mute: #666; --c-text-faint: #999;
            --c-bg: oklch(98% 0.006 240); --c-surface: #fff; --c-border: #dde2e8; --c-border-soft: #e9ecef;
            --c-sidebar-bg: linear-gradient(180deg, #2c3e50 0%, #34495e 100%);
            --grad-accent: linear-gradient(90deg, var(--c-primary) 0%, var(--c-accent) 100%);
            --c-badge-polars: #CE422B; --c-badge-polars-bg: #fdefeb;
            --c-badge-arrow: #1F8A8A; --c-badge-arrow-bg: #e6f5f5;
            --c-badge-bokeh: #7B3FA0; --c-badge-bokeh-bg: #f3ebf8;
            --c-badge-pyo3: #D9A441; --c-badge-pyo3-bg: #fbf4e3;
            --c-badge-jinja: #B11E33; --c-badge-jinja-bg: #fbe9ec;
            --c-badge-rust: #CE422B; --c-badge-rust-bg: #fdefeb;
            --r-sm: 4px; --r-md: 6px; --r-lg: 8px;
            --sh-sm: 0 1px 4px rgba(0,0,0,0.06);
            --sh-md: 0 2px 8px rgba(0,0,0,0.08);
            --sh-lg: 0 6px 24px rgba(0,0,0,0.12);
            --sidebar-w: 220px; --page-max: 1600px; --nav-h: 44px;
        }
        * { box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            background: var(--c-bg);
            color: var(--c-text);
            margin: 0;
            padding: 0;
        }
        /* ── Horizontal nav ────────────────────────────────────────────────── */
        .nav-horizontal {
            background: var(--c-surface);
            border-bottom: 1px solid var(--c-border);
            position: sticky;
            top: 0;
            z-index: 100;
            box-shadow: var(--sh-sm);
        }
        .nav-horizontal .nav-header {
            display: flex;
            align-items: stretch;
            padding: 0 var(--s-5);
        }
        .nav-horizontal .nav-report-title {
            font-size: var(--fs-md);
            font-weight: 700;
            color: var(--c-text-dark);
            white-space: nowrap;
            padding: 0 var(--s-4) 0 0;
            margin-right: var(--s-1);
            border-right: 2px solid var(--c-border);
            flex-shrink: 0;
            display: flex;
            align-items: center;
        }
        .nav-horizontal .nav-tabs-scroll {
            display: flex;
            align-items: stretch;
            overflow-x: auto;
            scrollbar-width: none;
            -ms-overflow-style: none;
            flex: 1;
        }
        .nav-horizontal .nav-tabs-scroll::-webkit-scrollbar { display: none; }
        .nav-horizontal .nav-tab {
            display: flex;
            align-items: center;
            text-decoration: none;
            padding: 0 var(--s-4);
            height: var(--nav-h);
            font-size: var(--fs-sm);
            font-weight: 500;
            color: #555;
            white-space: nowrap;
            border-bottom: 3px solid transparent;
            flex-shrink: 0;
            transition: color 0.15s, border-color 0.15s, background 0.15s;
        }
        .nav-horizontal .nav-tab:hover { color: var(--c-primary); background: var(--c-primary-tint); border-bottom-color: var(--c-primary-border); }
        .nav-horizontal .nav-tab.active { color: var(--c-primary); font-weight: 700; border-bottom-color: var(--c-primary); background: var(--c-primary-tint); }
        .nav-horizontal .nav-dd { position: relative; display: flex; align-items: stretch; flex-shrink: 0; }
        .nav-horizontal .nav-dd-trigger {
            display: flex; align-items: center; gap: var(--s-1); padding: 0 var(--s-4); height: var(--nav-h);
            border: none; background: none; font-family: inherit; font-size: var(--fs-sm);
            font-weight: 500; color: #555; white-space: nowrap; cursor: pointer;
            border-bottom: 3px solid transparent;
            transition: color 0.15s, border-color 0.15s, background 0.15s;
        }
        .nav-horizontal .nav-dd-trigger .caret { font-size: var(--fs-xs); opacity: 0.55; }
        .nav-horizontal .nav-dd:hover > .nav-dd-trigger,
        .nav-horizontal .nav-dd.open > .nav-dd-trigger { color: var(--c-primary); background: var(--c-primary-tint); border-bottom-color: var(--c-primary-border); }
        .nav-horizontal .nav-dd.has-active > .nav-dd-trigger { color: var(--c-primary); font-weight: 700; border-bottom-color: var(--c-primary); background: var(--c-primary-tint); }
        .nav-horizontal .nav-dd-menu {
            display: none; position: fixed; background: var(--c-surface); border: 1px solid var(--c-border);
            border-radius: var(--r-md); box-shadow: var(--sh-lg);
            min-width: 190px; z-index: 1000; padding: var(--s-1) 0;
        }
        .nav-horizontal .nav-dd-item { display: block; padding: var(--s-2) var(--s-5); font-size: var(--fs-sm); font-weight: 500; color: #444; text-decoration: none; white-space: nowrap; transition: background 0.1s, color 0.1s; }
        .nav-horizontal .nav-dd-item:hover { background: var(--c-primary-tint); color: var(--c-primary); }
        .nav-horizontal .nav-dd-item.active { background: var(--c-primary-tint); color: var(--c-primary); font-weight: 700; }
        .nav-horizontal .nav-dd-divider { border: none; border-top: 1px solid #eee; margin: var(--s-1) 0; }
        .nav-horizontal .nav-dd-sub { position: relative; }
        .nav-horizontal .nav-dd-sub-trigger {
            display: flex; justify-content: space-between; align-items: center; width: 100%;
            padding: var(--s-2) var(--s-4) var(--s-2) var(--s-5); border: none; background: none; font-family: inherit;
            font-size: var(--fs-sm); font-weight: 500; color: #444; white-space: nowrap; cursor: pointer;
            transition: background 0.1s, color 0.1s; text-align: left;
        }
        .nav-horizontal .nav-dd-sub-trigger .caret { font-size: var(--fs-xs); opacity: 0.55; }
        .nav-horizontal .nav-dd-sub:hover > .nav-dd-sub-trigger,
        .nav-horizontal .nav-dd-sub.open > .nav-dd-sub-trigger { background: var(--c-primary-tint); color: var(--c-primary); }
        .nav-horizontal .nav-dd-sub.has-active > .nav-dd-sub-trigger { color: var(--c-primary); font-weight: 700; }
        .nav-horizontal .nav-dd-sub-menu {
            display: none; position: fixed; background: var(--c-surface); border: 1px solid var(--c-border);
            border-radius: var(--r-md); box-shadow: var(--sh-lg); min-width: 190px; z-index: 1001; padding: var(--s-1) 0;
        }
        /* ── Vertical nav ──────────────────────────────────────────────────── */
        .nav-vertical { position: fixed; left: 0; top: 0; width: var(--sidebar-w); height: 100vh; overflow-y: auto; background: var(--c-sidebar-bg); z-index: 100; padding-bottom: var(--s-5); }
        .nav-vertical .nav-report-title { color: white; font-size: var(--fs-md); font-weight: 700; padding: var(--s-5) var(--s-4) var(--s-3); border-bottom: 1px solid rgba(255,255,255,0.12); line-height: 1.3; }
        .nav-vertical details > summary { list-style: none; display: flex; align-items: center; justify-content: space-between; cursor: pointer; user-select: none; color: rgba(255,255,255,0.45); font-size: var(--fs-cap); font-weight: 700; text-transform: uppercase; letter-spacing: 0.09em; padding: var(--s-3) var(--s-4) var(--s-1); }
        .nav-vertical details > summary::-webkit-details-marker { display: none; }
        .nav-vertical details > summary::after { content: "▸"; font-size: 0.9em; opacity: 0.6; transition: transform 0.15s; }
        .nav-vertical details[open] > summary::after { transform: rotate(90deg); }
        .nav-vertical .nav-indent { padding-left: var(--s-2); }
        .nav-vertical a { display: block; text-decoration: none; padding: var(--s-2) var(--s-4) var(--s-2) var(--s-5); font-size: var(--fs-sm); font-weight: 500; color: rgba(255,255,255,0.72); transition: background 0.12s, color 0.12s; }
        .nav-vertical a:hover { background: rgba(255,255,255,0.08); color: white; }
        .nav-vertical a.active { background: var(--c-primary); color: white; font-weight: 700; box-shadow: inset 3px 0 0 var(--c-accent); }
        .nav-vertical .nav-uncategorized { margin-top: var(--s-2); }
        .nav-vertical .nav-search { padding: var(--s-2) var(--s-3) var(--s-3); border-bottom: 1px solid rgba(255,255,255,0.08); }
        .nav-vertical .nav-search-input { width: 100%; box-sizing: border-box; background: rgba(255,255,255,0.10); border: 1px solid rgba(255,255,255,0.20); border-radius: var(--r-sm); color: white; padding: var(--s-2) var(--s-2) var(--s-2) 28px; font-size: var(--fs-sm); outline: none; background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='14' height='14' viewBox='0 0 24 24' fill='none' stroke='rgba(255,255,255,0.5)' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Ccircle cx='11' cy='11' r='8'/%3E%3Cline x1='21' y1='21' x2='16.65' y2='16.65'/%3E%3C/svg%3E"); background-repeat: no-repeat; background-position: var(--s-2) center; background-size: 14px 14px; }
        .nav-vertical .nav-search-input::placeholder { color: rgba(255,255,255,0.40); }
        .nav-vertical .nav-search-input:focus { background-color: rgba(255,255,255,0.15); border-color: rgba(255,255,255,0.40); }
        /* ── Page layout ───────────────────────────────────────────────────── */
        .layout-horizontal .page-content { max-width: var(--page-max); margin: 0 auto; padding: 0 var(--s-5) var(--s-7); }
        .layout-vertical .page-content { margin-left: var(--sidebar-w); padding: 0 var(--s-6) var(--s-7); }
        h1 { color: var(--c-text-dark); border-bottom: 3px solid transparent; border-image: var(--grad-accent) 1; padding-bottom: var(--s-3); margin: var(--s-6) 0 var(--s-3); }
        .subtitle { color: var(--c-text-mute); margin: 0 0 var(--s-6); font-size: var(--fs-base); }
        .grid-layout { display: grid; gap: var(--s-5); margin-bottom: var(--s-5); }
        .chart-container { background: var(--c-surface); border-radius: var(--r-lg); padding: var(--s-4); box-shadow: var(--sh-md); min-width: 0; }
        .chart-container:has(.paragraph-module) { background: transparent; box-shadow: none; padding: var(--s-2) 0; }
        .chart-title { color: var(--c-text-dark); font-size: var(--fs-lg); margin: 0 0 var(--s-2); padding-bottom: var(--s-2); border-bottom: 1px solid var(--c-primary-pale); }
        footer { margin-top: var(--s-5); color: var(--c-text-faint); font-size: var(--fs-base); text-align: center; }
        .tech-badge { display: inline-block; background: var(--c-primary-pale); color: var(--c-primary); border-radius: var(--r-sm); padding: var(--s-1) var(--s-2); margin: 0 var(--s-1) 0 0; font-weight: 600; font-size: var(--fs-sm); }
        .tech-badge.tech-polars { background: var(--c-badge-polars-bg); color: var(--c-badge-polars); }
        .tech-badge.tech-arrow  { background: var(--c-badge-arrow-bg);  color: var(--c-badge-arrow); }
        .tech-badge.tech-bokeh  { background: var(--c-badge-bokeh-bg);  color: var(--c-badge-bokeh); }
        .tech-badge.tech-pyo3   { background: var(--c-badge-pyo3-bg);   color: var(--c-badge-pyo3); }
        .tech-badge.tech-jinja  { background: var(--c-badge-jinja-bg);  color: var(--c-badge-jinja); }
        .tech-badge.tech-rust   { background: var(--c-badge-rust-bg);   color: var(--c-badge-rust); }
        .filter-bar { display: flex; flex-wrap: wrap; gap: var(--s-4); padding: var(--s-4) var(--s-5); margin-bottom: var(--s-6); background: var(--c-surface); border-left: 3px solid var(--c-accent); border-radius: var(--r-lg); box-shadow: var(--sh-md); align-items: center; }
        .filter-bar-label { font-weight: 700; color: var(--c-accent-deep); font-size: var(--fs-sm); white-space: nowrap; text-transform: uppercase; letter-spacing: 0.05em; }
        .filter-widget { flex: 1; min-width: 200px; }
        .switch-label { display: flex; align-items: center; gap: var(--s-3); font-size: var(--fs-base); color: var(--c-text-dark); cursor: pointer; }
        .paragraph-module { height: 100%; box-sizing: border-box; }
        .paragraph-module p { color: #555; line-height: 1.7; margin: 0 0 var(--s-3); font-size: var(--fs-md); }
        .paragraph-module p:last-child { margin-bottom: 0; }
        .table-module { overflow: hidden; }
        .table-wrapper { overflow-x: auto; max-height: 420px; overflow-y: auto; }
        .table-module table { width: 100%; border-collapse: collapse; font-size: var(--fs-sm); }
        .table-module thead th { background: var(--c-primary); color: white; padding: var(--s-2) var(--s-3); text-align: left; position: sticky; top: 0; white-space: nowrap; font-weight: 600; }
        .table-module tbody tr:nth-child(even) { background: var(--c-bg); }
        .table-module tbody tr:hover { background: var(--c-primary-pale); }
        .table-module tbody td { padding: var(--s-2) var(--s-3); border-bottom: 1px solid var(--c-border-soft); color: var(--c-text-dark); }
        .module-title { color: var(--c-text-dark); font-size: var(--fs-lg); margin: 0 0 var(--s-3); padding-bottom: var(--s-2); border-bottom: 1px solid var(--c-primary-pale); font-weight: 600; }
    </style>"#;

pub const NAV_DROPDOWN_SCRIPT: &str = r#"    <script>
    (function () {
        function showMenu(menu, x, y) {
            clearTimeout(menu._ht);
            menu.style.left = x + 'px';
            menu.style.top  = y + 'px';
            menu.style.display = 'block';
            var vw = window.innerWidth;
            var mw = menu.offsetWidth;
            if (x + mw > vw) menu.style.left = Math.max(0, vw - mw) + 'px';
        }
        function hideMenu(menu) { menu._ht = setTimeout(function () { menu.style.display = 'none'; }, 150); }
        function keepOpen(menu) { clearTimeout(menu._ht); }
        function wire(trigger, menu, openRight) {
            trigger.addEventListener('mouseenter', function () {
                var r = trigger.getBoundingClientRect();
                showMenu(menu, openRight ? r.right : r.left, openRight ? r.top : r.bottom);
            });
            trigger.addEventListener('mouseleave', function () { hideMenu(menu); });
            menu.addEventListener('mouseenter', function () { keepOpen(menu); });
            menu.addEventListener('mouseleave', function () { hideMenu(menu); });
        }
        document.querySelectorAll('.nav-horizontal .nav-dd').forEach(function (dd) {
            var t = dd.querySelector(':scope > .nav-dd-trigger');
            var m = dd.querySelector(':scope > .nav-dd-menu');
            if (!t || !m) return;
            wire(t, m, false);
            t.addEventListener('click', function (e) {
                e.stopPropagation();
                if (m.style.display === 'block') { m.style.display = 'none'; } else { var r = t.getBoundingClientRect(); showMenu(m, r.left, r.bottom); }
            });
        });
        document.querySelectorAll('.nav-horizontal .nav-dd-sub').forEach(function (sub) {
            var t = sub.querySelector(':scope > .nav-dd-sub-trigger');
            var m = sub.querySelector(':scope > .nav-dd-sub-menu');
            if (!t || !m) return;
            wire(t, m, true);
        });
        document.addEventListener('click', function () {
            document.querySelectorAll('.nav-horizontal .nav-dd-menu, .nav-horizontal .nav-dd-sub-menu').forEach(function (m) { m.style.display = 'none'; });
        });
    })();
    (function () {
        var input = document.getElementById('nav-search-input');
        if (!input) return;
        var sidebar = document.querySelector('.nav-vertical');
        if (!sidebar) return;
        sidebar.querySelectorAll('details').forEach(function (d) {
            if (d.open) d.setAttribute('data-was-open', '');
        });
        input.addEventListener('input', function () {
            var q = this.value.trim().toLowerCase();
            var links = sidebar.querySelectorAll('a[href]');
            var details = sidebar.querySelectorAll('details');
            if (!q) {
                links.forEach(function (a) { a.style.display = ''; });
                details.forEach(function (d) { d.style.display = ''; d.open = d.hasAttribute('data-was-open'); });
                return;
            }
            links.forEach(function (a) { a.style.display = 'none'; });
            details.forEach(function (d) { d.style.display = 'none'; });
            links.forEach(function (a) {
                if (a.textContent.trim().toLowerCase().indexOf(q) !== -1) {
                    a.style.display = '';
                    var el = a.parentElement;
                    while (el && el !== sidebar) {
                        if (el.tagName === 'DETAILS') { el.style.display = ''; el.open = true; }
                        el = el.parentElement;
                    }
                }
            });
        });
    })();
    </script>"#;
