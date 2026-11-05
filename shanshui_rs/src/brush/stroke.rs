//! `stroke()`: turns a centre line into a brush-stroke polygon whose width
//! follows `profile` along the line and wobbles with noise.

use std::f64::consts::PI;

use crate::chance::Chance;
use crate::geom::Pt;
use crate::ink::{Paint, PolyStyle, Sketch};

/// Arguments of `stroke(ptlist, args)`; `Default` mirrors the JS defaults.
#[derive(Clone, Copy)]
pub struct StrokeStyle<'a> {
    pub wid: f64,
    pub col: Paint,
    /// 0 = exact width, 1 = width fully modulated by noise.
    pub noi: f64,
    /// Outline width drawn around the stroke polygon.
    pub out: f64,
    /// Width multiplier at relative position `t ∈ [0, 1)`.
    pub profile: &'a dyn Fn(f64) -> f64,
}

fn sine_profile(t: f64) -> f64 {
    (t * PI).sin()
}

impl Default for StrokeStyle<'_> {
    fn default() -> Self {
        Self {
            wid: 2.0,
            col: Paint::ink(200, 0.9),
            noi: 0.5,
            out: 1.0,
            profile: &sine_profile,
        }
    }
}

/// Draws one stroke along `pts`. Example:
/// `stroke(&mut sk, &mut ch, &line, &StrokeStyle { wid: 3.0, ..Default::default() })`.
pub fn stroke(sketch: &mut Sketch, chance: &mut Chance, pts: &[Pt], style: &StrokeStyle) {
    if pts.len() < 2 {
        return;
    }
    let n0 = chance.random() * 10.0;
    let (left, right) = edges(chance, pts, style, n0);
    let mut outline = Vec::with_capacity(left.len() * 2 + 3);
    outline.push(pts[0]);
    outline.extend(left);
    outline.push(pts[pts.len() - 1]);
    outline.extend(right.into_iter().rev());
    outline.push(pts[0]);
    sketch.poly(outline, PolyStyle::solid(style.col, style.out));
}

fn edges(chance: &Chance, pts: &[Pt], style: &StrokeStyle, n0: f64) -> (Vec<Pt>, Vec<Pt>) {
    let n = pts.len();
    (1..n - 1)
        .map(|i| {
            let w = style.wid * (style.profile)(i as f64 / n as f64);
            let w = w * (1.0 - style.noi) + w * style.noi * chance.noise2(i as f64 * 0.5, n0);
            let a = bisector(pts[i - 1], pts[i], pts[i + 1]);
            let (dx, dy) = (w * a.cos(), w * a.sin());
            (
                [pts[i][0] + dx, pts[i][1] + dy],
                [pts[i][0] - dx, pts[i][1] - dy],
            )
        })
        .unzip()
}

/// Direction perpendicular-ish to the polyline at `p`, as the original
/// computes it (average of the two segment angles, flipped to one side).
pub fn bisector(prev: Pt, p: Pt, next: Pt) -> f64 {
    let a1 = (p[1] - prev[1]).atan2(p[0] - prev[0]);
    let a2 = (p[1] - next[1]).atan2(p[0] - next[0]);
    let a = (a1 + a2) / 2.0;
    if a < a2 { a + PI } else { a }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: usize) -> Vec<Pt> {
        (0..n).map(|i| [i as f64 * 10.0, 0.0]).collect()
    }

    #[test]
    fn polygon_has_both_edges_and_closes() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        stroke(&mut s, &mut c, &line(6), &StrokeStyle::default());
        let pts = &s.polygons[0].pts;
        assert_eq!(pts.len(), 2 * 4 + 3);
        assert_eq!(pts[0], pts[pts.len() - 1]);
    }

    #[test]
    fn width_is_respected_without_noise() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        let flat = |_: f64| 1.0;
        let style = StrokeStyle {
            wid: 3.0,
            noi: 0.0,
            profile: &flat,
            ..Default::default()
        };
        stroke(&mut s, &mut c, &line(5), &style);
        let ys: Vec<f64> = s.polygons[0].pts.iter().map(|p| p[1].abs()).collect();
        assert!(ys.iter().all(|y| *y < 3.0 + 1e-9));
        assert!(ys.iter().any(|y| (*y - 3.0).abs() < 1e-9));
    }

    #[test]
    fn short_input_draws_nothing() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(3));
        stroke(&mut s, &mut c, &line(1), &StrokeStyle::default());
        assert!(s.is_empty());
    }

    #[test]
    fn bisector_of_straight_line_is_vertical() {
        let a = bisector([0.0, 0.0], [1.0, 0.0], [2.0, 0.0]);
        assert!((a.sin().abs() - 1.0).abs() < 1e-9, "angle {a}");
    }
}
