//! `PolyTools.triangulate`, reduced to the convex mode — the only one the
//! original ever calls (tree07 leaves, distant mountains). Ears are cut off
//! and every triangle is shattered along its longest side until it is
//! smaller than `max_area`, so each shard can get its own noise-tinted ink.

use super::point::{Pt, distance, mid_pt};

pub type Tri = [Pt; 3];

/// Example: `triangulate(&square, 50.0, false)` → shards of area < 50.
pub fn triangulate(poly: &[Pt], max_area: f64, optimize: bool) -> Vec<Tri> {
    let mut rest = poly.to_vec();
    let mut out = Vec::new();
    while rest.len() > 3 {
        let ear = if optimize { best_ear(&rest) } else { 0 };
        let n = rest.len();
        let tri = [rest[(ear + n - 1) % n], rest[ear], rest[(ear + 1) % n]];
        shatter(tri, max_area, &mut out);
        rest.remove(ear);
    }
    if let [a, b, c] = rest[..] {
        shatter([a, b, c], max_area, &mut out);
    }
    out
}

/// Index of the ear whose triangle is least sliver-like (last one on ties).
fn best_ear(poly: &[Pt]) -> usize {
    let n = poly.len();
    let ratio = |i: usize| sliver_ratio(&[poly[(i + n - 1) % n], poly[i], poly[(i + 1) % n]]);
    (0..n).fold(0, |best, i| if ratio(i) >= ratio(best) { i } else { best })
}

fn sides(t: &Tri) -> [f64; 3] {
    [
        distance(t[0], t[1]),
        distance(t[1], t[2]),
        distance(t[2], t[0]),
    ]
}

fn area(t: &Tri) -> f64 {
    let [a, b, c] = *t;
    ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
}

fn sliver_ratio(t: &Tri) -> f64 {
    area(t) / sides(t).iter().sum::<f64>().max(f64::EPSILON)
}

fn shatter(t: Tri, max_area: f64, out: &mut Vec<Tri>) {
    if area(&t) < max_area {
        out.push(t);
        return;
    }
    let s = sides(&t);
    let ind = (0..3).fold(0, |m, i| if s[i] > s[m] { i } else { m });
    let (nind, lind) = ((ind + 1) % 3, (ind + 2) % 3);
    let mid = mid_pt(&[t[ind], t[nind]]);
    shatter([t[ind], mid, t[lind]], max_area, out);
    shatter([t[lind], t[nind], mid], max_area, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: [Pt; 4] = [[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]];

    #[test]
    fn preserves_total_area() {
        for optimize in [false, true] {
            let tris = triangulate(&SQUARE, 30.0, optimize);
            let total: f64 = tris.iter().map(area).sum();
            assert!((total - 400.0).abs() < 1e-6, "total {total}");
        }
    }

    #[test]
    fn shards_are_small() {
        let tris = triangulate(&SQUARE, 30.0, false);
        assert!(tris.iter().all(|t| area(t) < 30.0));
    }

    #[test]
    fn triangle_input_passes_through() {
        let t = triangulate(&SQUARE[..3], 1000.0, true);
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn best_ear_prefers_fat_triangle() {
        let poly = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 0.1]];
        let ear = best_ear(&poly);
        assert!(ear == 1 || ear == 2, "ear {ear}");
    }
}
