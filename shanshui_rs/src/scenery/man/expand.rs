//! `expand`: both edges of a band of varying width around a centre line,
//! including the end caps (unlike `stroke`, which pinches the ends).

use std::f64::consts::PI;

use crate::brush::stroke::bisector;
use crate::geom::Pt;

/// Example: `let (left, right) = expand(&line, &|t| 3.0 * (1.0 - t));`
pub fn expand(pts: &[Pt], width: &dyn Fn(f64) -> f64) -> (Vec<Pt>, Vec<Pt>) {
    let n = pts.len();
    if n < 2 {
        return (pts.to_vec(), pts.to_vec());
    }
    let mut left = vec![cap(pts[0], pts[1], pts[0], width(0.0))];
    let mut right = vec![cap(pts[0], pts[1], pts[0], -width(0.0))];
    for i in 1..n - 1 {
        let w = width(i as f64 / n as f64);
        let a = bisector(pts[i - 1], pts[i], pts[i + 1]);
        left.push([pts[i][0] + w * a.cos(), pts[i][1] + w * a.sin()]);
        right.push([pts[i][0] - w * a.cos(), pts[i][1] - w * a.sin()]);
    }
    left.push(cap(pts[n - 2], pts[n - 1], pts[n - 1], width(1.0)));
    right.push(cap(pts[n - 2], pts[n - 1], pts[n - 1], -width(1.0)));
    (left, right)
}

/// Point `w` away from `at`, perpendicular to segment `a → b`.
fn cap(a: Pt, b: Pt, at: Pt, w: f64) -> Pt {
    let ang = (b[1] - a[1]).atan2(b[0] - a[0]) - PI / 2.0;
    [at[0] + w * ang.cos(), at[1] + w * ang.sin()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_has_constant_width() {
        let line: Vec<Pt> = (0..5).map(|i| [i as f64, 0.0]).collect();
        let (l, r) = expand(&line, &|_| 2.0);
        assert_eq!(l.len(), 5);
        for (a, b) in l.iter().zip(&r) {
            assert!(((a[1] - b[1]).abs() - 4.0).abs() < 1e-9, "{a:?} {b:?}");
        }
    }

    #[test]
    fn short_line_passthrough() {
        let (l, _) = expand(&[[1.0, 1.0]], &|_| 1.0);
        assert_eq!(l, vec![[1.0, 1.0]]);
    }
}
