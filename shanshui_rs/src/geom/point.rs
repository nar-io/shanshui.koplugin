//! 2-D point helpers (`distance`, `mapval`, `PolyTools.midPt`, the
//! ubiquitous `.map(v => [v[0]+xof, v[1]+yof])`).

/// World-space point; y grows downwards like the original SVG.
pub type Pt = [f64; 2];

pub fn distance(a: Pt, b: Pt) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Linear remap of `value` from `[istart, istop]` to `[ostart, ostop]`.
/// Example: `mapval(5.0, 0.0, 10.0, 0.0, 1.0) == 0.5`.
pub fn mapval(value: f64, istart: f64, istop: f64, ostart: f64, ostop: f64) -> f64 {
    ostart + (ostop - ostart) * ((value - istart) / (istop - istart))
}

/// Centroid of the vertices. Example: `mid_pt(&[[0.,0.],[2.,2.]]) == [1.,1.]`.
pub fn mid_pt(pts: &[Pt]) -> Pt {
    let n = pts.len() as f64;
    pts.iter()
        .fold([0.0, 0.0], |acc, p| [acc[0] + p[0] / n, acc[1] + p[1] / n])
}

/// `a` at `t = 0`, `b` at `t = 1`.
pub fn lerp(a: Pt, b: Pt, t: f64) -> Pt {
    [a[0] * (1.0 - t) + b[0] * t, a[1] * (1.0 - t) + b[1] * t]
}

/// Copy of `pts` shifted by `(dx, dy)`.
pub fn translated(pts: &[Pt], dx: f64, dy: f64) -> Vec<Pt> {
    pts.iter().map(|p| [p[0] + dx, p[1] + dy]).collect()
}

/// `pts` followed by `tail` reversed: the outline of a band between two
/// parallel edges, a pattern the original writes as `a.concat(b.reverse())`.
pub fn joined_reversed(pts: &[Pt], tail: &[Pt]) -> Vec<Pt> {
    pts.iter().chain(tail.iter().rev()).copied().collect()
}

/// Axis-aligned bounds `[xmin, ymin, xmax, ymax]`; `None` when empty.
pub fn bounds(pts: &[Pt]) -> Option<[f64; 4]> {
    let first = pts.first()?;
    let init = [first[0], first[1], first[0], first[1]];
    Some(pts.iter().fold(init, |b, p| {
        [
            b[0].min(p[0]),
            b[1].min(p[1]),
            b[2].max(p[0]),
            b[3].max(p[1]),
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_euclidean() {
        assert_eq!(distance([0.0, 0.0], [3.0, 4.0]), 5.0);
    }

    #[test]
    fn mapval_remaps() {
        assert_eq!(mapval(5.0, 0.0, 10.0, 100.0, 200.0), 150.0);
    }

    #[test]
    fn mid_pt_is_centroid() {
        assert_eq!(mid_pt(&[[0.0, 0.0], [2.0, 4.0]]), [1.0, 2.0]);
    }

    #[test]
    fn lerp_endpoints() {
        assert_eq!(lerp([0.0, 0.0], [2.0, 2.0], 0.5), [1.0, 1.0]);
    }

    #[test]
    fn translated_shifts() {
        assert_eq!(translated(&[[1.0, 1.0]], 2.0, -1.0), vec![[3.0, 0.0]]);
    }

    #[test]
    fn joined_reversed_order() {
        let j = joined_reversed(&[[0.0, 0.0]], &[[1.0, 0.0], [2.0, 0.0]]);
        assert_eq!(j, vec![[0.0, 0.0], [2.0, 0.0], [1.0, 0.0]]);
    }

    #[test]
    fn bounds_of_points() {
        assert_eq!(
            bounds(&[[1.0, 5.0], [-2.0, 3.0]]),
            Some([-2.0, 3.0, 1.0, 5.0])
        );
        assert_eq!(bounds(&[]), None);
    }
}
