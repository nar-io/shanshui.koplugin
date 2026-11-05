//! SVG export — produces fully compliant SVG identical to original Shan Shui,
//! with optional transparent sky for E-ink overlay / screensaver integration.

use std::fmt::Write as _;

use super::paint::Paint;
use super::sketch::{InkPolygon, Sketch};

/// Renders `sketch` inside the world rectangle `view = [x, y, w, h]` with given target dimensions.
pub fn to_svg(
    sketch: &Sketch,
    view: [f64; 4],
    width: usize,
    height: usize,
    is_transparent: bool,
) -> String {
    let [x, y, w, h] = view;
    let bg_style = if is_transparent {
        "background-color:transparent;"
    } else {
        "background-color:#ffffff;"
    };

    let bg_rect = if is_transparent {
        ""
    } else {
        "<rect x='-2000' y='-2000' width='10000' height='10000' fill='#ffffff'/>\n"
    };

    let mut out = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}' \
         viewBox='{x:.1} {y:.1} {w:.1} {h:.1}' style='{bg_style}'>\n\
         {bg_rect}\
         <g id='G' transform='translate(0,0)'>\n"
    );
    for polygon in &sketch.polygons {
        out.push_str(&polyline(polygon));
    }
    out.push_str("</g>\n</svg>\n");
    out
}

fn polyline(polygon: &InkPolygon) -> String {
    let mut points = String::new();
    for p in &polygon.pts {
        let _ = write!(points, " {:.1},{:.1}", p[0], p[1]);
    }
    let s = polygon.style;
    let fill_css = css(s.fill);
    let (stroke_css, stroke_width) = if s.outline.is_visible() && s.width > 0.0 {
        (css(s.outline), s.width)
    } else if s.fill.is_visible() && s.fill != Paint::Paper {
        (fill_css.clone(), 0.0)
    } else {
        ("none".into(), 0.0)
    };

    format!(
        "<polyline points='{points}' style='fill:{fill_css};stroke:{stroke_css};stroke-width:{stroke_width}'/>\n"
    )
}

fn css(paint: Paint) -> String {
    match paint {
        Paint::Clear => "none".into(),
        Paint::Paper => "white".into(),
        Paint::Ink { gray, alpha } => format!("rgba({gray},{gray},{gray},{alpha:.3})"),
    }
}
