//! The display list that replaces the original's SVG strings: an ordered
//! list of filled/outlined polygons in world coordinates.

use super::paint::Paint;
use crate::geom::Pt;
use crate::geom::point::bounds;

/// Fill + outline of one `poly(...)` call.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolyStyle {
    pub fill: Paint,
    pub outline: Paint,
    pub width: f64,
}

impl PolyStyle {
    /// `poly(p, {fil: paint})`: filled, no outline.
    pub fn filled(paint: Paint) -> Self {
        Self {
            fill: paint,
            outline: Paint::Clear,
            width: 0.0,
        }
    }

    /// `poly(p, {fil: "none", str: paint, wid})`: an open polyline.
    pub fn outlined(paint: Paint, width: f64) -> Self {
        Self {
            fill: Paint::Clear,
            outline: paint,
            width,
        }
    }

    /// `poly(p, {fil: "white", str: "none"})`: hides what lies behind.
    pub fn paper() -> Self {
        Self::filled(Paint::Paper)
    }

    /// Fill and outline with the same paint, as `stroke()` and the
    /// triangle shards do.
    pub fn solid(paint: Paint, width: f64) -> Self {
        Self {
            fill: paint,
            outline: paint,
            width,
        }
    }
}

/// One polygon with its bounding box cached for viewport culling.
#[derive(Clone, Debug, PartialEq)]
pub struct InkPolygon {
    pub pts: Vec<Pt>,
    pub style: PolyStyle,
    /// `[xmin, ymin, xmax, ymax]` including half the outline width.
    pub bbox: [f64; 4],
}

/// Ordered display list; later polygons paint over earlier ones.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sketch {
    pub polygons: Vec<InkPolygon>,
}

impl Sketch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a polygon unless it is empty or fully invisible.
    /// Example: `sketch.poly(pts, PolyStyle::filled(Paint::ink(100, 0.5)))`.
    pub fn poly(&mut self, pts: Vec<Pt>, style: PolyStyle) {
        let has_outline = style.outline.is_visible() && style.width > 0.0;
        if !style.fill.is_visible() && !has_outline {
            return;
        }
        // The original patched NaN coordinates after the fact (chunkloader's
        // "gotcha"); dropping the polygon up front is the same outcome.
        if pts.iter().flatten().any(|v| !v.is_finite()) {
            return;
        }
        let Some(b) = bounds(&pts) else { return };
        let pad = if has_outline { style.width / 2.0 } else { 0.0 };
        let bbox = [b[0] - pad, b[1] - pad, b[2] + pad, b[3] + pad];
        self.polygons.push(InkPolygon { pts, style, bbox });
    }

    /// Moves every polygon of `other` to the end of this list.
    pub fn append(&mut self, mut other: Sketch) {
        self.polygons.append(&mut other.polygons);
    }

    /// Union of all bounding boxes, `None` for an empty sketch.
    pub fn bbox(&self) -> Option<[f64; 4]> {
        self.polygons.iter().map(|p| p.bbox).reduce(|a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        })
    }

    pub fn len(&self) -> usize {
        self.polygons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.polygons.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRI: [Pt; 3] = [[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]];

    #[test]
    fn invisible_polygons_are_dropped() {
        let mut s = Sketch::new();
        s.poly(TRI.to_vec(), PolyStyle::filled(Paint::Clear));
        s.poly(TRI.to_vec(), PolyStyle::outlined(Paint::ink(100, 0.5), 0.0));
        s.poly(vec![], PolyStyle::paper());
        assert!(s.is_empty());
    }

    #[test]
    fn non_finite_points_are_dropped() {
        let mut s = Sketch::new();
        s.poly(vec![[f64::NAN, 0.0], [1.0, 1.0]], PolyStyle::paper());
        assert!(s.is_empty());
    }

    #[test]
    fn bbox_includes_outline() {
        let mut s = Sketch::new();
        s.poly(TRI.to_vec(), PolyStyle::solid(Paint::ink(100, 0.5), 2.0));
        assert_eq!(s.bbox(), Some([-1.0, -1.0, 11.0, 11.0]));
    }

    #[test]
    fn append_keeps_order() {
        let (mut a, mut b) = (Sketch::new(), Sketch::new());
        a.poly(TRI.to_vec(), PolyStyle::paper());
        b.poly(TRI.to_vec(), PolyStyle::filled(Paint::ink(1, 1.0)));
        a.append(b);
        assert_eq!(a.len(), 2);
        assert_eq!(a.polygons[1].style.fill, Paint::ink(1, 1.0));
    }
}
