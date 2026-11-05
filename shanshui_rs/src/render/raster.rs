//! Paints a display list into a supersampled RGBA buffer (painter's order,
//! source-over), then box-filters it down to one colour per dot. This is
//! the part of the browser's SVG renderer the port has to provide itself.

use super::coverage::Coverage;
use super::viewport::Viewport;
use crate::ink::{InkPolygon, Paint};

/// Linear RGB in `[0, 1]`.
pub type Rgb = [f32; 3];

const WHITE: Rgb = [1.0, 1.0, 1.0];

/// Supersampled, premultiplied RGBA canvas for one frame.
pub struct InkLayer {
    view: Viewport,
    ss: usize,
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
    cov: Coverage,
}

/// One flattened colour (ink over white paper) per dot.
#[derive(Clone, Debug, PartialEq)]
pub struct DotImage {
    pub w: usize,
    pub h: usize,
    pub rgb: Vec<Rgb>,
}

impl DotImage {
    pub fn at(&self, x: usize, y: usize) -> Rgb {
        self.rgb[y * self.w + x]
    }
}

fn pigment(paint: Paint) -> Option<(Rgb, f32)> {
    match paint {
        Paint::Clear => None,
        Paint::Paper => Some((WHITE, 1.0)),
        Paint::Ink { gray, alpha } => Some(([f32::from(gray) / 255.0; 3], alpha)),
    }
}

impl InkLayer {
    /// Transparent layer covering `view`, `ss × ss` samples per dot.
    /// Example: `InkLayer::new(view, 3)`.
    pub fn new(view: Viewport, ss: usize) -> Self {
        let (w, h) = (view.dots_w * ss, view.dots_h * ss);
        let px = vec![[0.0; 4]; w * h];
        Self {
            view,
            ss,
            w,
            h,
            px,
            cov: Coverage::default(),
        }
    }

    /// Paints one polygon: fill first, then its outline, like SVG.
    pub fn paint(&mut self, polygon: &InkPolygon) {
        if !self.view.sees(polygon.bbox) {
            return;
        }
        let k = self.ss as f64;
        let pts: Vec<[f64; 2]> = polygon
            .pts
            .iter()
            .map(|p| {
                let d = self.view.to_dots(*p);
                [d[0] * k, d[1] * k]
            })
            .collect();
        let style = polygon.style;
        if let Some(pig) = pigment(style.fill) {
            self.reset_cov(polygon.bbox);
            self.cov.fill(&pts);
            self.composite(pig);
        }
        if let (Some(pig), true) = (pigment(style.outline), style.width > 0.0) {
            self.reset_cov(polygon.bbox);
            self.cov.line(&pts, style.width / self.view.scale * k);
            self.composite(pig);
        }
    }

    /// Mask over the polygon bbox (clipped to the layer).
    fn reset_cov(&mut self, bbox: [f64; 4]) {
        let k = self.ss as f64;
        let a = self.view.to_dots([bbox[0], bbox[1]]);
        let b = self.view.to_dots([bbox[2], bbox[3]]);
        let x0 = ((a[0] * k).floor() as i64 - 1).max(0);
        let y0 = ((a[1] * k).floor() as i64 - 1).max(0);
        let x1 = ((b[0] * k).ceil() as i64 + 1).min(self.w as i64);
        let y1 = ((b[1] * k).ceil() as i64 + 1).min(self.h as i64);
        let (w, h) = ((x1 - x0).max(0) as usize, (y1 - y0).max(0) as usize);
        self.cov.reset(x0, y0, w, h);
    }

    fn composite(&mut self, (rgb, alpha): (Rgb, f32)) {
        let src = [rgb[0] * alpha, rgb[1] * alpha, rgb[2] * alpha, alpha];
        let (w, px) = (self.w, &mut self.px);
        for (x, y) in self.cov.covered() {
            let dst = &mut px[y as usize * w + x as usize];
            for c in 0..4 {
                dst[c] = src[c] + dst[c] * (1.0 - alpha);
            }
        }
    }

    /// Box-filters to dots and flattens over white.
    pub fn resolve(&self) -> DotImage {
        let (dw, dh) = (self.view.dots_w, self.view.dots_h);
        let rgb = (0..dw * dh)
            .map(|i| self.dot_color(i % dw, i / dw))
            .collect();
        DotImage { w: dw, h: dh, rgb }
    }

    fn dot_color(&self, dx: usize, dy: usize) -> Rgb {
        let ss = self.ss;
        let mut acc = [0.0f32; 3];
        for sy in 0..ss {
            let row = (dy * ss + sy) * self.w + dx * ss;
            for p in &self.px[row..row + ss] {
                for c in 0..3 {
                    acc[c] += p[c] + (1.0 - p[3]);
                }
            }
        }
        let n = (ss * ss) as f32;
        [acc[0] / n, acc[1] / n, acc[2] / n]
    }
}

/// Renders polygons in order into a dot image. Example:
/// `rasterize(chunks.iter().flat_map(|c| &c.sketch.polygons), view, 3)`.
pub fn rasterize<'a>(
    polygons: impl IntoIterator<Item = &'a InkPolygon>,
    view: Viewport,
    ss: usize,
) -> DotImage {
    let mut layer = InkLayer::new(view, ss);
    polygons.into_iter().for_each(|p| layer.paint(p));
    layer.resolve()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::{PolyStyle, Sketch};

    fn view() -> Viewport {
        Viewport {
            left: 0.0,
            top: 0.0,
            scale: 1.0,
            dots_w: 10,
            dots_h: 10,
        }
    }

    fn square(x0: f64, x1: f64) -> Vec<[f64; 2]> {
        vec![[x0, 0.0], [x1, 0.0], [x1, 10.0], [x0, 10.0]]
    }

    #[test]
    fn opaque_ink_darkens_covered_dots_only() {
        let mut s = Sketch::new();
        s.poly(square(0.0, 5.0), PolyStyle::filled(Paint::ink(0, 1.0)));
        let img = rasterize(&s.polygons, view(), 2);
        assert_eq!(img.at(2, 2), [0.0; 3]);
        assert_eq!(img.at(7, 2), [1.0; 3]);
    }

    #[test]
    fn paper_hides_earlier_ink() {
        let mut s = Sketch::new();
        s.poly(square(0.0, 10.0), PolyStyle::filled(Paint::ink(0, 1.0)));
        s.poly(square(0.0, 5.0), PolyStyle::paper());
        let img = rasterize(&s.polygons, view(), 2);
        assert_eq!(img.at(2, 2), [1.0; 3]);
        assert_eq!(img.at(7, 2), [0.0; 3]);
    }

    #[test]
    fn half_covered_dot_is_grey() {
        let mut s = Sketch::new();
        s.poly(square(0.0, 2.5), PolyStyle::filled(Paint::ink(0, 1.0)));
        let img = rasterize(&s.polygons, view(), 2);
        assert!((img.at(2, 5)[0] - 0.5).abs() < 1e-6, "{:?}", img.at(2, 5));
    }

    #[test]
    fn alpha_composites() {
        let mut s = Sketch::new();
        s.poly(square(0.0, 10.0), PolyStyle::filled(Paint::ink(0, 0.5)));
        let img = rasterize(&s.polygons, view(), 1);
        assert!((img.at(5, 5)[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn outline_draws_open_line() {
        let mut s = Sketch::new();
        s.poly(
            vec![[0.0, 5.0], [10.0, 5.0]],
            PolyStyle::outlined(Paint::ink(0, 1.0), 1.0),
        );
        let img = rasterize(&s.polygons, view(), 2);
        assert!(img.at(5, 5)[0] < 0.6 || img.at(5, 4)[0] < 0.6);
        assert_eq!(img.at(5, 0), [1.0; 3]);
    }
}
