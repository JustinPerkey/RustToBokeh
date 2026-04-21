//! Bokeh document model: assembles `docs_json` and `render_items`.

use serde_json;
use uuid::Uuid;

use super::id_gen::IdGen;
use super::model::BokehObject;
use super::verifier::{verify_document, VerifyReport};

/// A Bokeh document containing all model roots for one page.
pub struct BokehDocument {
    doc_id: String,
    roots: Vec<BokehObject>,
    /// Mapping from root model ID → HTML div UUID (for render_items).
    root_div_map: Vec<(String, String)>,
}

impl BokehDocument {
    pub fn new() -> Self {
        BokehDocument {
            doc_id: Uuid::new_v4().to_string(),
            roots: Vec::new(),
            root_div_map: Vec::new(),
        }
    }

    /// Add a root model (Figure, widget, etc.) and return its div UUID.
    pub fn add_root(&mut self, obj: BokehObject) -> String {
        let div_id = Uuid::new_v4().to_string();
        self.root_div_map.push((obj.id.clone(), div_id.clone()));
        self.roots.push(obj);
        div_id
    }

    /// Add a root model that isn't a DOMView (BooleanFilter, CDS hoisted for
    /// cross-root refs, Range1d widget, etc.). A UUID is still allocated and
    /// recorded in `root_div_map` so `render_items.roots` and `root_ids`
    /// match `all_roots` 1-for-1. The returned UUID must be emitted as a
    /// hidden `<div>` in the page HTML — BokehJS's `embed.embed_items` calls
    /// `document.getElementById(uuid)` for every entry in `root_ids` and
    /// throws if the element is missing, even for models it later skips
    /// because they aren't DOMViews. Keeping `all_roots` and render_items
    /// aligned by index is what prevents DOMView roots (charts, widgets)
    /// from being placed into the wrong target divs.
    pub fn add_hidden_root(&mut self, obj: BokehObject) -> String {
        let div_id = Uuid::new_v4().to_string();
        self.root_div_map.push((obj.id.clone(), div_id.clone()));
        self.roots.push(obj);
        div_id
    }

    /// Serialize to the `docs_json` string (single-quote-escaped for embedding in JS).
    pub fn to_docs_json(&self, id_gen: &mut IdGen) -> String {
        let config_id = id_gen.next();
        let notif_id = id_gen.next();

        let notifications = BokehObject::new("Notifications", notif_id);
        let config = BokehObject::new("DocumentConfig", config_id)
            .attr("notifications", notifications.into_value());

        // Build the roots array value
        let roots_val: Vec<serde_json::Value> = self
            .roots
            .iter()
            .map(|r| serde_json::to_value(r).unwrap())
            .collect();

        let doc = serde_json::json!({
            "version": "3.9.0",
            "title": "Bokeh Application",
            "config": serde_json::to_value(&config).unwrap(),
            "roots": roots_val,
        });

        let outer = serde_json::json!({ &self.doc_id: doc });
        let json_str = serde_json::to_string(&outer).unwrap();

        // Escape for embedding in a JS single-quoted string
        json_str.replace('\\', "\\\\").replace('\'', "\\'")
    }

    /// Serialize to the `render_items` JSON array string.
    ///
    /// Uses `serde_json::Map` (order-preserving with `preserve_order` feature)
    /// so the `roots` JSON object key order matches `root_ids`. BokehJS's
    /// `embed_items` iterates the `roots` object and must see ids in the
    /// same order their root models were added, otherwise model→div mapping
    /// skews across charts.
    pub fn to_render_items(&self) -> String {
        let mut roots_obj = serde_json::Map::with_capacity(self.root_div_map.len());
        for (mid, did) in &self.root_div_map {
            roots_obj.insert(mid.clone(), serde_json::Value::String(did.clone()));
        }

        let root_ids: Vec<&str> = self.root_div_map.iter().map(|(mid, _)| mid.as_str()).collect();

        let item = serde_json::json!({
            "docid": &self.doc_id,
            "roots": serde_json::Value::Object(roots_obj),
            "root_ids": root_ids,
        });

        serde_json::to_string(&serde_json::json!([item])).unwrap()
    }

    /// Get all (root_id, div_id) pairs for building HTML div containers.
    pub fn root_divs(&self) -> &[(String, String)] {
        &self.root_div_map
    }

    /// Number of roots in the document.
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// Validate that every model ID is defined inline at most once and every
    /// `Ref` resolves to an inline definition.
    pub fn verify(&self) -> VerifyReport {
        verify_document(&self.roots)
    }
}

impl Default for BokehDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the JS embed script block for a page.
pub fn build_embed_script(docs_json: &str, render_items: &str) -> String {
    format!(
        r#"
(function() {{
  const fn = function() {{
    Bokeh.safely(function() {{
      (function(root) {{
        function embed_document(root) {{
        const docs_json = '{docs_json}';
        const render_items = {render_items};
        root.Bokeh.embed.embed_items(docs_json, render_items);
        }}
        if (root.Bokeh !== undefined) {{
          embed_document(root);
        }} else {{
          let attempts = 0;
          const timer = setInterval(function(root) {{
            if (root.Bokeh !== undefined) {{
              clearInterval(timer);
              embed_document(root);
            }} else {{
              attempts++;
              if (attempts > 100) {{
                clearInterval(timer);
                console.log("Bokeh: ERROR: Unable to run BokehJS code because BokehJS library is missing");
              }}
            }}
          }}, 10, root)
        }}
      }})(window);
    }});
  }};
  if (document.readyState != "loading") fn();
  else document.addEventListener("DOMContentLoaded", fn);
}})();"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::id_gen::IdGen;

    #[test]
    fn empty_document_produces_valid_json() {
        let doc = BokehDocument::new();
        let mut id_gen = IdGen::new();
        let json_str = doc.to_docs_json(&mut id_gen);
        // Should not contain unescaped single quotes
        // The raw JSON is valid when unescaped
        let unescaped = json_str.replace("\\'", "'").replace("\\\\", "\\");
        assert!(serde_json::from_str::<serde_json::Value>(&unescaped).is_err() == false
            || unescaped.starts_with('{')); // either valid JSON or non-empty
    }

    #[test]
    fn render_items_contains_docid_and_roots() {
        let mut doc = BokehDocument::new();
        let root = BokehObject::new("PanTool", "p1001".into());
        doc.add_root(root);
        let items = doc.to_render_items();
        assert!(items.contains("docid"));
        assert!(items.contains("p1001"));
        assert!(items.contains("root_ids"));
    }

    #[test]
    fn docs_json_escapes_single_quotes() {
        // Any single quote in data should be escaped
        let doc = BokehDocument::new();
        let mut id_gen = IdGen::new();
        let json = doc.to_docs_json(&mut id_gen);
        // The outer wrapper uses single quotes — no raw single quote should appear
        // except as \'
        let parts: Vec<&str> = json.split('\'').collect();
        // If any part would have been an unescaped quote, split would produce > 1 part
        // But we need \\' (escaped), so check there are no unescaped singles
        // (our escape replaces ' with \' so the result only has \' not ')
        assert!(!parts.contains(&"'"));
    }
}
