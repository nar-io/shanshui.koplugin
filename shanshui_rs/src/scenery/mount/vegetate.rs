//! `vegetate(treeFunc, growthRule, proofRule)`: picks mesh points where
//! something may grow; the caller draws on the survivors.

use super::Mesh;
use crate::geom::Pt;

/// Mesh points where `grows(i, j)` holds, in mesh order.
/// Example: `sites(&mesh, |i, _| i == 0)` → the whole ridge line.
pub fn sites(mesh: &Mesh, mut grows: impl FnMut(usize, usize) -> bool) -> Vec<Pt> {
    let mut out = Vec::new();
    for (i, row) in mesh.iter().enumerate() {
        for (j, p) in row.iter().enumerate() {
            if grows(i, j) {
                out.push(*p);
            }
        }
    }
    out
}

/// The "middle forest" proof rule: keep a site only if more than two
/// other sites lie within `radius`, so trees come in groves.
pub fn in_grove(all: &[Pt], k: usize, radius: f64) -> bool {
    let r2 = radius * radius;
    let p = all[k];
    all.iter()
        .enumerate()
        .filter(|&(m, q)| m != k && (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) < r2)
        .nth(2)
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sites_follow_rule() {
        let mesh = vec![vec![[0.0, 0.0], [1.0, 0.0]], vec![[2.0, 0.0]]];
        assert_eq!(sites(&mesh, |i, _| i == 0), vec![[0.0, 0.0], [1.0, 0.0]]);
        assert_eq!(sites(&mesh, |_, j| j == 1), vec![[1.0, 0.0]]);
    }

    #[test]
    fn grove_needs_three_neighbours() {
        let pts = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], [500.0, 0.0]];
        assert!(in_grove(&pts, 0, 30.0));
        assert!(!in_grove(&pts, 4, 30.0));
        assert!(!in_grove(&pts[..3], 0, 30.0));
    }
}
