//! CDS placeholder rewriting.
//!
//! Filter widgets embed a placeholder `BokehValue::Ref(__cds_<source_key>)`
//! in their CustomJS args. After charts are built and real `ColumnDataSource`
//! IDs are known, those placeholder refs are patched in place.

use super::model::{BokehObject, BokehValue};

/// Extract the ID of the first `ColumnDataSource` in a figure's renderers.
pub(super) fn extract_first_cds_id(fig: &BokehObject) -> Option<String> {
    for (key, val) in &fig.attributes {
        if key == "renderers" {
            if let BokehValue::Array(renderers) = val {
                for renderer in renderers {
                    if let Some(id) = find_cds_id_in_value(renderer) {
                        return Some(id);
                    }
                }
            }
        }
    }
    None
}

fn find_cds_id_in_value(val: &BokehValue) -> Option<String> {
    if let BokehValue::Object(obj) = val {
        if obj.name == "GlyphRenderer" {
            for (k, v) in &obj.attributes {
                if k == "data_source" {
                    return match v {
                        BokehValue::Object(cds) => Some(cds.id.clone()),
                        BokehValue::Ref(id) => Some(id.clone()),
                        _ => None,
                    };
                }
            }
        }
    }
    None
}

/// Recursively replace `BokehValue::Ref(placeholder)` with `BokehValue::Ref(real_id)`.
pub(super) fn replace_placeholder_in_obj(obj: &mut BokehObject, placeholder: &str, real_id: &str) {
    for (_, v) in &mut obj.attributes {
        replace_placeholder(v, placeholder, real_id);
    }
}

fn replace_placeholder(val: &mut BokehValue, placeholder: &str, real_id: &str) {
    match val {
        BokehValue::Ref(id) if id == placeholder => *id = real_id.to_string(),
        BokehValue::Array(arr) => {
            for v in arr {
                replace_placeholder(v, placeholder, real_id);
            }
        }
        BokehValue::Map(entries) => {
            for (_, v) in entries {
                replace_placeholder(v, placeholder, real_id);
            }
        }
        BokehValue::Object(obj) => {
            replace_placeholder_in_obj(obj, placeholder, real_id);
        }
        BokehValue::Value(inner) => replace_placeholder(inner, placeholder, real_id),
        _ => {}
    }
}

/// Find the inline `ColumnDataSource` with the given ID inside a figure (in its
/// renderers' `data_source` slot), extract it out, and replace the inline
/// definition with a `Ref` to the same ID. Returns the extracted CDS so the
/// caller can add it as a separate document root, making the ID backward-
/// resolvable for later Refs (e.g. from a Range1d widget's CustomJS args).
///
/// Returns `None` if no inline CDS with that ID is found.
pub(super) fn hoist_inline_cds_with_id(
    fig: &mut BokehObject,
    target_id: &str,
) -> Option<BokehObject> {
    for (key, val) in &mut fig.attributes {
        if key == "renderers" {
            if let BokehValue::Array(renderers) = val {
                for renderer in renderers.iter_mut() {
                    if let Some(extracted) = hoist_from_renderer(renderer, target_id) {
                        return Some(extracted);
                    }
                }
            }
        }
    }
    None
}

fn hoist_from_renderer(val: &mut BokehValue, target_id: &str) -> Option<BokehObject> {
    if let BokehValue::Object(obj) = val {
        if obj.name == "GlyphRenderer" {
            for (k, v) in &mut obj.attributes {
                if k == "data_source" {
                    if let BokehValue::Object(cds) = v {
                        if cds.id == target_id {
                            let extracted = cds.clone();
                            *v = BokehValue::Ref(target_id.to_string());
                            return Some(*extracted);
                        }
                    }
                }
            }
        }
    }
    None
}
