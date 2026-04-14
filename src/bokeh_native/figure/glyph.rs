//! `GlyphRenderer` + `CDSView` helper.

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};

/// Build a `GlyphRenderer` with a given glyph and optional CDSView filter.
pub fn build_glyph_renderer(
    id_gen: &mut IdGen,
    source_ref: BokehValue,
    glyph: BokehObject,
    nonselection_glyph: Option<BokehObject>,
    filter_ref: Option<BokehValue>, // None → AllIndices
) -> BokehObject {
    let view_id = id_gen.next();
    let all_indices_id = id_gen.next();
    let renderer_id = id_gen.next();

    let filter_val = filter_ref.unwrap_or_else(|| {
        BokehObject::new("AllIndices", all_indices_id).into_value()
    });

    let view = BokehObject::new("CDSView", view_id)
        .attr("filter", filter_val);

    let nonsel = nonselection_glyph.unwrap_or_else(|| {
        let id = id_gen.next();
        BokehObject::new("Line", id)
    });

    BokehObject::new("GlyphRenderer", renderer_id)
        .attr("data_source", source_ref)
        .attr("view", view.into_value())
        .attr("glyph", glyph.into_value())
        .attr("nonselection_glyph", nonsel.into_value())
}
