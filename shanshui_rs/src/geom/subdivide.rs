//! `div(plist, reso)`: inserts `reso - 1` evenly spaced points on every
//! segment of a polyline, so later noise can bend it smoothly.

use super::point::{Pt, lerp};

/// Example: `subdivide(&[[0.,0.],[10.,0.]], 5)` yields x = 0, 2, 4, 6, 8, 10.
pub fn subdivide(pts: &[Pt], reso: usize) -> Vec<Pt> {
    let Some(&last) = pts.last() else {
        return Vec::new();
    };
    let reso = reso.max(1);
    let total = (pts.len() - 1) * reso;
    let mut out: Vec<Pt> = (0..total)
        .map(|i| {
            let (a, b) = (pts[i / reso], pts[i.div_ceil(reso)]);
            lerp(a, b, (i % reso) as f64 / reso as f64)
        })
        .collect();
    out.push(last);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evenly_spaced() {
        let d = subdivide(&[[0.0, 0.0], [10.0, 0.0]], 5);
        let xs: Vec<f64> = d.iter().map(|p| p[0]).collect();
        assert_eq!(xs, vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
    }

    #[test]
    fn multi_segment_length() {
        let d = subdivide(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], 4);
        assert_eq!(d.len(), 9);
        assert_eq!(d[4], [1.0, 0.0]);
    }

    #[test]
    fn empty_and_single() {
        assert!(subdivide(&[], 3).is_empty());
        assert_eq!(subdivide(&[[1.0, 2.0]], 3), vec![[1.0, 2.0]]);
    }
}
