//! Document invariant checks for the native Bokeh renderer.
//!
//! BokehJS rejects a document when:
//! - the same model ID is fully defined (`{"type":"object","name":...,"id":...,"attributes":...}`)
//!   more than once anywhere in the doc — the second copy overwrites the
//!   first's attributes (typically with `{}`), wiping out renderers/data and
//!   causing charts to not render. (See commit 3102a18 / a90e73d.)
//! - a `Ref` (`{"id":"..."}`) points to an ID that has no inline definition
//!   anywhere in the doc.
//!
//! [`verify_document`] walks the entire root tree, collects every inline
//! `Object` ID and every `Ref` target, and reports both classes of error.

use std::collections::{HashMap, HashSet};

use super::model::{BokehObject, BokehValue};

/// Result of a document invariant check.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct VerifyReport {
    /// IDs that appear as a full inline `Object` more than once.
    /// Each entry: `(id, count)`.
    pub duplicate_inline: Vec<(String, usize)>,
    /// IDs referenced via `Ref` that have no inline `Object` definition.
    pub dangling_refs: Vec<String>,
    /// `(referrer_root_id, target_id)` for every `Ref` whose target is defined
    /// inline *later* in the root order than the root that contains the Ref.
    /// BokehJS decodes roots top-to-bottom and resolves each `Ref` against
    /// already-seen models, so a forward reference produces
    /// "reference X isn't known" at runtime even though the model is present.
    pub forward_refs: Vec<(String, String)>,
}

impl VerifyReport {
    pub fn is_ok(&self) -> bool {
        self.duplicate_inline.is_empty()
            && self.dangling_refs.is_empty()
            && self.forward_refs.is_empty()
    }

    /// One-line summary suitable for an error message.
    pub fn summary(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.duplicate_inline.is_empty() {
            let dups: Vec<String> = self
                .duplicate_inline
                .iter()
                .map(|(id, n)| format!("{id} x{n}"))
                .collect();
            parts.push(format!("duplicate inline objects: {}", dups.join(", ")));
        }
        if !self.dangling_refs.is_empty() {
            parts.push(format!("dangling refs: {}", self.dangling_refs.join(", ")));
        }
        if !self.forward_refs.is_empty() {
            let fwds: Vec<String> = self
                .forward_refs
                .iter()
                .map(|(r, t)| format!("{r}->{t}"))
                .collect();
            parts.push(format!("forward refs: {}", fwds.join(", ")));
        }
        parts.join("; ")
    }
}

/// Walk the roots and report duplicate inline objects, dangling refs, and
/// forward refs (Refs to targets that are only defined in a *later* root).
pub fn verify_document(roots: &[BokehObject]) -> VerifyReport {
    let mut inline_counts: HashMap<String, usize> = HashMap::new();
    let mut ref_targets: HashSet<String> = HashSet::new();

    for root in roots {
        walk_object(root, &mut inline_counts, &mut ref_targets);
    }

    let duplicate_inline: Vec<(String, usize)> = inline_counts
        .iter()
        .filter(|(_, &n)| n > 1)
        .map(|(id, &n)| (id.clone(), n))
        .collect();

    let dangling_refs: Vec<String> = ref_targets
        .iter()
        .filter(|id| !inline_counts.contains_key(id.as_str()))
        .cloned()
        .collect();

    // Second pass: walk in document order, register inline IDs as we reach
    // them, and flag any Ref whose target has not yet been seen.
    let mut seen: HashSet<String> = HashSet::new();
    let mut forward_refs: Vec<(String, String)> = Vec::new();
    for root in roots {
        walk_object_ordered(root, &root.id, &mut seen, &mut forward_refs);
    }

    let mut report = VerifyReport {
        duplicate_inline,
        dangling_refs,
        forward_refs,
    };
    report.duplicate_inline.sort();
    report.dangling_refs.sort();
    report.forward_refs.sort();
    report
}

fn walk_object(
    obj: &BokehObject,
    inline_counts: &mut HashMap<String, usize>,
    ref_targets: &mut HashSet<String>,
) {
    *inline_counts.entry(obj.id.clone()).or_insert(0) += 1;
    for (_, val) in &obj.attributes {
        walk_value(val, inline_counts, ref_targets);
    }
}

fn walk_object_ordered(
    obj: &BokehObject,
    referrer_root: &str,
    seen: &mut HashSet<String>,
    forward_refs: &mut Vec<(String, String)>,
) {
    seen.insert(obj.id.clone());
    for (_, val) in &obj.attributes {
        walk_value_ordered(val, referrer_root, seen, forward_refs);
    }
}

fn walk_value_ordered(
    val: &BokehValue,
    referrer_root: &str,
    seen: &mut HashSet<String>,
    forward_refs: &mut Vec<(String, String)>,
) {
    match val {
        BokehValue::Object(o) => walk_object_ordered(o, referrer_root, seen, forward_refs),
        BokehValue::Ref(id) => {
            if !seen.contains(id) {
                forward_refs.push((referrer_root.to_string(), id.clone()));
            }
        }
        BokehValue::Array(arr) => {
            for v in arr {
                walk_value_ordered(v, referrer_root, seen, forward_refs);
            }
        }
        BokehValue::Map(entries) => {
            for (_, v) in entries {
                walk_value_ordered(v, referrer_root, seen, forward_refs);
            }
        }
        BokehValue::Value(boxed) | BokehValue::FieldTransform { transform: boxed, .. } => {
            walk_value_ordered(boxed, referrer_root, seen, forward_refs);
        }
        BokehValue::Null
        | BokehValue::Bool(_)
        | BokehValue::Int(_)
        | BokehValue::Float(_)
        | BokehValue::NaN
        | BokehValue::Str(_)
        | BokehValue::Field(_) => {}
    }
}

fn walk_value(
    val: &BokehValue,
    inline_counts: &mut HashMap<String, usize>,
    ref_targets: &mut HashSet<String>,
) {
    match val {
        BokehValue::Object(o) => walk_object(o, inline_counts, ref_targets),
        BokehValue::Ref(id) => {
            ref_targets.insert(id.clone());
        }
        BokehValue::Array(arr) => {
            for v in arr {
                walk_value(v, inline_counts, ref_targets);
            }
        }
        BokehValue::Map(entries) => {
            for (_, v) in entries {
                walk_value(v, inline_counts, ref_targets);
            }
        }
        BokehValue::Value(boxed) | BokehValue::FieldTransform { transform: boxed, .. } => {
            walk_value(boxed, inline_counts, ref_targets);
        }
        BokehValue::Null
        | BokehValue::Bool(_)
        | BokehValue::Int(_)
        | BokehValue::Float(_)
        | BokehValue::NaN
        | BokehValue::Str(_)
        | BokehValue::Field(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(name: &'static str, id: &str) -> BokehObject {
        BokehObject::new(name, id.into())
    }

    #[test]
    fn empty_roots_is_ok() {
        let report = verify_document(&[]);
        assert!(report.is_ok());
    }

    #[test]
    fn single_root_no_refs_is_ok() {
        let r = obj("PanTool", "p1");
        let report = verify_document(&[r]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn ref_to_existing_root_is_ok() {
        let target = obj("BooleanFilter", "p1")
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true)]));
        let user = obj("CDSView", "p2").attr("filter", BokehValue::ref_of("p1"));
        let report = verify_document(&[target, user]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn ref_to_nested_inline_object_is_ok() {
        let nested = obj("Selection", "p2");
        let parent = obj("ColumnDataSource", "p1").attr("selected", nested.into_value());
        let user = obj("Other", "p3").attr("ref_to_nested", BokehValue::ref_of("p2"));
        let report = verify_document(&[parent, user]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn dangling_ref_is_reported() {
        let user = obj("CDSView", "p2").attr("filter", BokehValue::ref_of("does_not_exist"));
        let report = verify_document(&[user]);
        assert!(!report.is_ok());
        assert_eq!(report.dangling_refs, vec!["does_not_exist".to_string()]);
    }

    #[test]
    fn duplicate_root_inline_is_reported() {
        let a = obj("Range1d", "p1").attr("start", BokehValue::Float(0.0));
        let b = obj("Range1d", "p1").attr("start", BokehValue::Float(10.0));
        let report = verify_document(&[a, b]);
        assert!(!report.is_ok());
        assert_eq!(report.duplicate_inline, vec![("p1".to_string(), 2)]);
    }

    #[test]
    fn duplicate_inline_in_nested_attr_is_reported() {
        // BooleanFilter "p2" defined as root AND inlined inside CDSView attribute
        // — exactly the bug fixed in commit a90e73d.
        let bf_root = obj("BooleanFilter", "p2")
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true)]));
        let bf_inline = obj("BooleanFilter", "p2")
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true)]));
        let view = obj("CDSView", "p3").attr("filter", bf_inline.into_value());
        let report = verify_document(&[bf_root, view]);
        assert!(!report.is_ok());
        assert_eq!(report.duplicate_inline, vec![("p2".to_string(), 2)]);
    }

    #[test]
    fn ref_in_map_is_resolved() {
        let target = obj("ColumnDataSource", "p1");
        let cb = obj("CustomJS", "p2").attr(
            "args",
            BokehValue::Map(vec![("source".into(), BokehValue::ref_of("p1"))]),
        );
        let report = verify_document(&[target, cb]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn ref_in_array_is_resolved() {
        let f1 = obj("BooleanFilter", "f1");
        let f2 = obj("BooleanFilter", "f2");
        let isect = obj("IntersectionFilter", "i1").attr(
            "operands",
            BokehValue::Array(vec![BokehValue::ref_of("f1"), BokehValue::ref_of("f2")]),
        );
        let report = verify_document(&[f1, f2, isect]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn ref_inside_value_spec_is_resolved() {
        let mapper = obj("CategoricalColorMapper", "m1");
        let glyph = obj("VBar", "g1").attr(
            "fill_color",
            BokehValue::value_of(BokehValue::ref_of("m1")),
        );
        let report = verify_document(&[mapper, glyph]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn ref_inside_field_transform_is_resolved() {
        let mapper = obj("CategoricalColorMapper", "m1");
        let glyph = obj("VBar", "g1").attr(
            "fill_color",
            BokehValue::field_transform("category", BokehValue::ref_of("m1")),
        );
        let report = verify_document(&[mapper, glyph]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn forward_ref_in_root_order_is_reported() {
        // Range1d-shaped scenario: root #1 (CustomJS) holds a Ref to a
        // BooleanFilter (`bf`) whose inline definition appears as root #2.
        // BokehJS decodes roots in order and would fail with
        // "reference bf isn't known" while decoding root #1.
        let cb = obj("CustomJS", "cb")
            .attr("args", BokehValue::Map(vec![("bf".into(), BokehValue::ref_of("bf"))]));
        let bf = obj("BooleanFilter", "bf")
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true)]));
        let report = verify_document(&[cb, bf]);
        assert!(!report.is_ok(), "{}", report.summary());
        assert_eq!(
            report.forward_refs,
            vec![("cb".to_string(), "bf".to_string())]
        );
    }

    #[test]
    fn ref_inside_same_root_after_inline_def_is_ok() {
        // A Ref to a model defined earlier in the *same* root is fine:
        // depth-first walk has already registered it before reaching the Ref.
        let nested = obj("Selection", "s1");
        let parent = obj("ColumnDataSource", "cds")
            .attr("selected", nested.into_value())
            .attr("self_ref", BokehValue::ref_of("s1"));
        let report = verify_document(&[parent]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn backward_ref_across_roots_is_ok() {
        let bf = obj("BooleanFilter", "bf")
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true)]));
        let cb = obj("CustomJS", "cb")
            .attr("args", BokehValue::Map(vec![("bf".into(), BokehValue::ref_of("bf"))]));
        let report = verify_document(&[bf, cb]);
        assert!(report.is_ok(), "{}", report.summary());
    }

    #[test]
    fn summary_lists_both_classes() {
        let dup = obj("X", "d1");
        let dup2 = obj("X", "d1");
        let bad_ref = obj("Y", "d2").attr("r", BokehValue::ref_of("missing"));
        let report = verify_document(&[dup, dup2, bad_ref]);
        let s = report.summary();
        assert!(s.contains("duplicate inline objects"));
        assert!(s.contains("d1"));
        assert!(s.contains("dangling refs"));
        assert!(s.contains("missing"));
    }
}
