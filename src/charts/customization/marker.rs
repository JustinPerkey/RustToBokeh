/// Bokeh scatter marker shape.
///
/// Controls the glyph drawn at each data point in a scatter (or line) chart.
/// The default when `None` is [`MarkerType::Circle`].
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = ScatterConfig::builder()
///     .x("revenue")
///     .y("profit")
///     .x_label("Revenue (k)")
///     .y_label("Profit (k)")
///     .marker(MarkerType::Diamond)
///     .build()?;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum MarkerType {
    /// Filled circle (default).
    Circle,
    /// Filled square.
    Square,
    /// Filled diamond.
    Diamond,
    /// Filled upward-pointing triangle.
    Triangle,
    /// Filled downward-pointing triangle.
    InvertedTriangle,
    /// Filled hexagon.
    Hex,
    /// Filled star.
    Star,
    /// Asterisk (six-spoke star, no fill).
    Asterisk,
    /// Plus sign (no fill).
    Plus,
    /// X mark (no fill).
    X,
    /// Y mark (no fill).
    Y,
    /// Horizontal dash (no fill).
    Dash,
    /// Small dot.
    Dot,
    /// Cross (no fill).
    Cross,
    /// Circle with a cross inscribed.
    CircleCross,
    /// Circle with a dot at the centre.
    CircleDot,
    /// Circle with an X inscribed.
    CircleX,
    /// Circle with a Y inscribed.
    CircleY,
    /// Hexagon with a dot at the centre.
    HexDot,
    /// Square with a cross inscribed.
    SquareCross,
    /// Square with a dot at the centre.
    SquareDot,
    /// Square with a pin/anchor shape.
    SquarePin,
    /// Square with an X inscribed.
    SquareX,
    /// Star with a dot at the centre.
    StarDot,
    /// Triangle with a dot at the centre.
    TriangleDot,
    /// Triangle with a pin/anchor shape.
    TrianglePin,
}

impl MarkerType {
    /// Return the Bokeh marker string expected by the Python renderer.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            MarkerType::Circle => "circle",
            MarkerType::Square => "square",
            MarkerType::Diamond => "diamond",
            MarkerType::Triangle => "triangle",
            MarkerType::InvertedTriangle => "inverted_triangle",
            MarkerType::Hex => "hex",
            MarkerType::Star => "star",
            MarkerType::Asterisk => "asterisk",
            MarkerType::Plus => "plus",
            MarkerType::X => "x",
            MarkerType::Y => "y",
            MarkerType::Dash => "dash",
            MarkerType::Dot => "dot",
            MarkerType::Cross => "cross",
            MarkerType::CircleCross => "circle_cross",
            MarkerType::CircleDot => "circle_dot",
            MarkerType::CircleX => "circle_x",
            MarkerType::CircleY => "circle_y",
            MarkerType::HexDot => "hex_dot",
            MarkerType::SquareCross => "square_cross",
            MarkerType::SquareDot => "square_dot",
            MarkerType::SquarePin => "square_pin",
            MarkerType::SquareX => "square_x",
            MarkerType::StarDot => "star_dot",
            MarkerType::TriangleDot => "triangle_dot",
            MarkerType::TrianglePin => "triangle_pin",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_basic_variants() {
        assert_eq!(MarkerType::Circle.as_str(), "circle");
        assert_eq!(MarkerType::Square.as_str(), "square");
        assert_eq!(MarkerType::Diamond.as_str(), "diamond");
        assert_eq!(MarkerType::Triangle.as_str(), "triangle");
        assert_eq!(MarkerType::InvertedTriangle.as_str(), "inverted_triangle");
        assert_eq!(MarkerType::Hex.as_str(), "hex");
        assert_eq!(MarkerType::Star.as_str(), "star");
    }

    #[test]
    fn as_str_compound_variants() {
        assert_eq!(MarkerType::CircleCross.as_str(), "circle_cross");
        assert_eq!(MarkerType::CircleDot.as_str(), "circle_dot");
        assert_eq!(MarkerType::CircleX.as_str(), "circle_x");
        assert_eq!(MarkerType::HexDot.as_str(), "hex_dot");
        assert_eq!(MarkerType::SquareCross.as_str(), "square_cross");
        assert_eq!(MarkerType::StarDot.as_str(), "star_dot");
        assert_eq!(MarkerType::TriangleDot.as_str(), "triangle_dot");
        assert_eq!(MarkerType::TrianglePin.as_str(), "triangle_pin");
    }
}
