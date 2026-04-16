//! Navigation HTML builder for native Bokeh rendering.
//!
//! Mirrors the Python `build_nav_tree()` logic and the Jinja2 nav macros
//! from `templates/chart.html`, but generates HTML directly in Rust.

use crate::pages::Page;

/// A node in the navigation tree (a category or the root).
struct NavNode {
    label: String,
    pages: Vec<NavPage>,
    children: Vec<NavNode>,
}

struct NavPage {
    slug: String,
    label: String,
}

/// Build the navigation HTML for all pages, highlighting `current_slug`.
pub fn build_nav_html(
    pages: &[Page],
    report_title: &str,
    nav_style: &str,
    current_slug: &str,
) -> String {
    let tree = build_tree(pages, current_slug);
    let home_slug = pages.first().map(|p| p.slug.as_str()).unwrap_or("");

    if nav_style == "vertical" {
        build_vertical_nav(&tree, report_title, current_slug, home_slug)
    } else {
        build_horizontal_nav(&tree, report_title, current_slug)
    }
}

// ── Tree builder ──────────────────────────────────────────────────────────────

fn build_tree(pages: &[Page], _current_slug: &str) -> NavNode {
    let mut root = NavNode { label: String::new(), pages: Vec::new(), children: Vec::new() };

    for page in pages {
        let nav_page = NavPage {
            slug: page.slug.clone(),
            label: page.nav_label.clone(),
        };
        match &page.category {
            None => root.pages.push(nav_page),
            Some(cat) => insert_into_tree(&mut root, cat, nav_page),
        }
    }

    root
}

/// Insert a page into the tree, creating intermediate nodes for `"A/B/C"` paths.
fn insert_into_tree(node: &mut NavNode, path: &str, page: NavPage) {
    let (head, rest) = match path.split_once('/') {
        Some((h, r)) => (h, Some(r)),
        None => (path, None),
    };

    let child = node.children.iter_mut().find(|c| c.label == head);
    let child = if let Some(c) = child {
        c
    } else {
        node.children.push(NavNode { label: head.to_string(), pages: Vec::new(), children: Vec::new() });
        node.children.last_mut().unwrap()
    };

    match rest {
        None => child.pages.push(page),
        Some(r) => insert_into_tree(child, r, page),
    }
}

fn node_has_active(node: &NavNode, current_slug: &str) -> bool {
    node.pages.iter().any(|p| p.slug == current_slug)
        || node.children.iter().any(|c| node_has_active(c, current_slug))
}

// ── Horizontal nav ────────────────────────────────────────────────────────────

fn build_horizontal_nav(tree: &NavNode, report_title: &str, current_slug: &str) -> String {
    let mut html = String::from(r#"<nav class="nav-horizontal"><div class="nav-header">"#);

    if !report_title.is_empty() {
        html.push_str(&format!(
            r#"<div class="nav-report-title">{}</div>"#,
            escape_html(report_title)
        ));
    }

    html.push_str(r#"<div class="nav-tabs-scroll">"#);

    // Ungrouped pages
    for page in &tree.pages {
        let active = if page.slug == current_slug { " active" } else { "" };
        html.push_str(&format!(
            r#"<a href="{slug}.html" class="nav-tab{active}">{label}</a>"#,
            slug = page.slug,
            label = escape_html(&page.label),
            active = active,
        ));
    }

    // Category nodes
    for child in &tree.children {
        html.push_str(&build_h_dd_node(child, current_slug));
    }

    html.push_str("</div></div></nav>");
    html
}

fn build_h_dd_node(node: &NavNode, current_slug: &str) -> String {
    let has_active = node_has_active(node, current_slug);
    let active_cls = if has_active { " has-active" } else { "" };
    let mut html = format!(
        r#"<div class="nav-dd{active_cls}"><button class="nav-dd-trigger">{label}<span class="caret">▾</span></button><div class="nav-dd-menu">"#,
        active_cls = active_cls,
        label = escape_html(&node.label),
    );

    for page in &node.pages {
        let active = if page.slug == current_slug { " active" } else { "" };
        html.push_str(&format!(
            r#"<a href="{slug}.html" class="nav-dd-item{active}">{label}</a>"#,
            slug = page.slug,
            label = escape_html(&page.label),
            active = active,
        ));
    }

    if !node.pages.is_empty() && !node.children.is_empty() {
        html.push_str(r#"<hr class="nav-dd-divider">"#);
    }

    for child in &node.children {
        html.push_str(&build_h_dd_sub_node(child, current_slug));
    }

    html.push_str("</div></div>");
    html
}

fn build_h_dd_sub_node(node: &NavNode, current_slug: &str) -> String {
    let has_active = node_has_active(node, current_slug);
    let active_cls = if has_active { " has-active" } else { "" };
    let mut html = format!(
        r#"<div class="nav-dd-sub{active_cls}"><button class="nav-dd-sub-trigger">{label}<span class="caret">▸</span></button><div class="nav-dd-sub-menu">"#,
        active_cls = active_cls,
        label = escape_html(&node.label),
    );

    for page in &node.pages {
        let active = if page.slug == current_slug { " active" } else { "" };
        html.push_str(&format!(
            r#"<a href="{slug}.html" class="nav-dd-item{active}">{label}</a>"#,
            slug = page.slug,
            label = escape_html(&page.label),
            active = active,
        ));
    }

    if !node.pages.is_empty() && !node.children.is_empty() {
        html.push_str(r#"<hr class="nav-dd-divider">"#);
    }

    for child in &node.children {
        html.push_str(&build_h_dd_sub_node(child, current_slug));
    }

    html.push_str("</div></div>");
    html
}

// ── Vertical nav ──────────────────────────────────────────────────────────────

fn build_vertical_nav(tree: &NavNode, report_title: &str, current_slug: &str, home_slug: &str) -> String {
    let mut html = String::from(r#"<nav class="nav-vertical">"#);

    if !report_title.is_empty() {
        let title_inner = if !home_slug.is_empty() {
            format!(
                r#"<a href="{home}.html" style="color:inherit;text-decoration:none;">{title}</a>"#,
                home = home_slug,
                title = escape_html(report_title),
            )
        } else {
            escape_html(report_title)
        };
        html.push_str(&format!(r#"<div class="nav-report-title">{title_inner}</div>"#));
    }

    html.push_str(r#"<div class="nav-search"><input type="search" id="nav-search-input" class="nav-search-input" placeholder="Search&#x2026;" autocomplete="off"></div>"#);

    html.push_str(r#"<div class="nav-uncategorized">"#);
    for page in &tree.pages {
        let active = if page.slug == current_slug { r#" class="active""# } else { "" };
        html.push_str(&format!(
            r#"<a href="{slug}.html"{active}>{label}</a>"#,
            slug = page.slug,
            label = escape_html(&page.label),
            active = active,
        ));
    }
    html.push_str("</div>");

    for child in &tree.children {
        html.push_str(&build_v_node(child, current_slug));
    }

    html.push_str("</nav>");
    html
}

fn build_v_node(node: &NavNode, current_slug: &str) -> String {
    let has_active = node_has_active(node, current_slug);
    let open = if has_active { " open" } else { "" };
    let mut html = format!(
        r#"<details{open}><summary>{label}</summary><div class="nav-indent">"#,
        open = open,
        label = escape_html(&node.label),
    );

    for page in &node.pages {
        let active = if page.slug == current_slug { r#" class="active""# } else { "" };
        html.push_str(&format!(
            r#"<a href="{slug}.html"{active}>{label}</a>"#,
            slug = page.slug,
            label = escape_html(&page.label),
            active = active,
        ));
    }

    for child in &node.children {
        html.push_str(&build_v_node(child, current_slug));
    }

    html.push_str("</div></details>");
    html
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
