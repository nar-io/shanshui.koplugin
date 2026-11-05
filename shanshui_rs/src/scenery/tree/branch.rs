//! `branch` (the two edges of a tapering, kinked limb) and `twig` (thin
//! recursive sprigs ending in leaf clusters).

use std::f64::consts::PI;

use crate::brush::blob::leaf;
use crate::brush::{BlobStyle, StrokeStyle, blob, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::ink::{Paint, Sketch};

/// Arguments of `branch(args)`; `Default` mirrors the JS defaults.
#[derive(Clone, Copy, Debug)]
pub struct BranchArgs {
    pub hei: f64,
    pub wid: f64,
    pub ang: f64,
    /// Samples per spine segment (may be fractional, as `hei / 20`).
    pub det: f64,
    pub ben: f64,
}

impl Default for BranchArgs {
    fn default() -> Self {
        Self {
            hei: 300.0,
            wid: 6.0,
            ang: 0.0,
            det: 10.0,
            ben: PI * 0.2,
        }
    }
}

/// Left and right edges of a limb starting at the origin.
/// Example: `let [l, r] = branch(&mut ch, &BranchArgs { ang: -PI / 2.0, ..Default::default() });`
pub fn branch(chance: &mut Chance, a: &BranchArgs) -> [Vec<Pt>; 2] {
    let spine = spine(chance, a);
    edges(chance, &spine, a)
}

/// Three kinks of random bend, rotated so the tip points along `ang`.
fn spine(chance: &mut Chance, a: &BranchArgs) -> Vec<Pt> {
    let (mut p, mut a0) = ([0.0, 0.0], 0.0);
    let mut pts = vec![p];
    for _ in 0..3 {
        a0 += (a.ben / 2.0 + chance.random() * a.ben / 2.0) * chance.sign();
        p = [p[0] + a0.cos() * a.hei / 3.0, p[1] - a0.sin() * a.hei / 3.0];
        pts.push(p);
    }
    let ta = p[1].atan2(p[0]);
    pts.iter()
        .map(|q| {
            let (ang, d) = (q[1].atan2(q[0]), q[0].hypot(q[1]));
            [d * (ang - ta + a.ang).cos(), d * (ang - ta + a.ang).sin()]
        })
        .collect()
}

fn edges(chance: &mut Chance, spine: &[Pt], a: &BranchArgs) -> [Vec<Pt>; 2] {
    let tl = (spine.len() - 1) as f64 * a.det;
    let (mut left, mut right) = (Vec::new(), Vec::new());
    let mut last = [0.0, 0.0];
    let mut i = 0.0;
    while i < tl {
        let (lo, hi) = (
            spine[(i / a.det).floor() as usize],
            spine[(i / a.det).ceil() as usize],
        );
        let p = (i % a.det) / a.det;
        let n = [lo[0] * (1.0 - p) + hi[0] * p, lo[1] * (1.0 - p) + hi[1] * p];
        let ang = (n[1] - last[1]).atan2(n[0] - last[0]);
        let woff = (chance.noise1(i * 0.3) - 0.5) * a.wid * a.hei / 80.0;
        let b = if p == 0.0 {
            chance.random() * a.wid
        } else {
            0.0
        };
        let nw = a.wid * ((tl - i) / tl * 0.5 + 0.5);
        left.push(offset(n, ang + PI / 2.0, nw + woff + b));
        right.push(offset(n, ang - PI / 2.0, nw - woff + b));
        last = n;
        i += 1.0;
    }
    [left, right]
}

fn offset(p: Pt, ang: f64, d: f64) -> Pt {
    [p[0] + ang.cos() * d, p[1] + ang.sin() * d]
}

/// Arguments of `twig(tx, ty, dep, args)`.
#[derive(Clone, Copy, Debug)]
pub struct TwigArgs {
    pub dir: f64,
    pub sca: f64,
    pub wid: f64,
    pub ang: f64,
    /// Whether to end in leaves, and how far the leaves hang.
    pub lea: (bool, f64),
}

impl Default for TwigArgs {
    fn default() -> Self {
        Self {
            dir: 1.0,
            sca: 1.0,
            wid: 1.0,
            ang: 0.0,
            lea: (true, 12.0),
        }
    }
}

const TWIG_LEN: usize = 10;

/// Example: `twig(&mut sk, &mut ch, [x, y], 1, &TwigArgs::default())`.
pub fn twig(sketch: &mut Sketch, chance: &mut Chance, t: Pt, dep: u32, a: &TwigArgs) {
    let hs = chance.random() * 0.5 + 0.5;
    let a0 = chance.random() * PI / 6.0 * a.dir + a.ang;
    let mut line = Vec::with_capacity(TWIG_LEN);
    for i in 0..TWIG_LEN {
        let p = twig_point(t, i, a0, a.dir * a.sca * hs, a.sca);
        line.push(p);
        if (i == TWIG_LEN / 3 || i == TWIG_LEN * 2 / 3) && dep > 0 {
            let sub = TwigArgs {
                sca: a.sca * 0.8,
                dir: a.dir * chance.sign(),
                ..*a
            };
            twig(sketch, chance, p, dep - 1, &sub);
        }
    }
    if a.lea.0 {
        twig_leaves(sketch, chance, line[TWIG_LEN - 1], dep, a);
    }
    let fade = |x: f64| (x * PI / 2.0).cos();
    let style = StrokeStyle {
        wid: 1.0,
        profile: &fade,
        col: Paint::ink(100, 0.5),
        ..Default::default()
    };
    stroke(sketch, chance, &line, &style);
}

fn twig_point(t: Pt, i: usize, a0: f64, xscale: f64, sca: f64) -> Pt {
    let f = 1.0 - 1.0 / (i as f64 / TWIG_LEN as f64 + 1.0).powi(5);
    let (mx, my) = (f * 50.0 * xscale, -(i as f64) * 5.0 * sca);
    let (ang, d) = (my.atan2(mx), mx.hypot(my));
    [t[0] + (ang + a0).cos() * d, t[1] + (ang + a0).sin() * d]
}

fn twig_leaves(sketch: &mut Sketch, chance: &mut Chance, tip: Pt, dep: u32, a: &TwigArgs) {
    let col = Paint::ink(100, 0.5 + f64::from(dep) * 0.2);
    for j in 0..5 {
        let dj = (j as f64 - 2.5) * 5.0;
        let bx = tip[0] + a.ang.cos() * dj * a.wid;
        let by = tip[1] + (a.ang.sin() * dj - a.lea.1 / f64::from(dep + 1)) * a.wid;
        let wid = (6.0 + 3.0 * chance.random()) * a.wid;
        let len = (15.0 + 12.0 * chance.random()) * a.wid;
        let ang = a.ang / 2.0 + PI / 2.0 + PI * 0.2 * (chance.random() - 0.5);
        let style = BlobStyle {
            wid,
            len,
            ang,
            col,
            profile: &leaf,
            ..Default::default()
        };
        blob(sketch, chance, bx, by, &style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_edges_have_det_samples_per_segment() {
        let mut c = Chance::from_seed(1);
        let [l, r] = branch(&mut c, &BranchArgs::default());
        assert_eq!(l.len(), 30);
        assert_eq!(r.len(), 30);
    }

    #[test]
    fn branch_points_along_angle() {
        let mut c = Chance::from_seed(2);
        let a = BranchArgs {
            ang: -PI / 2.0,
            ben: 0.0,
            ..Default::default()
        };
        let spine = spine(&mut c, &a);
        let tip = spine[spine.len() - 1];
        assert!(
            tip[0].abs() < 1e-6 && (tip[1] + 300.0).abs() < 1e-6,
            "tip {tip:?}"
        );
    }

    #[test]
    fn fractional_det_still_works() {
        let mut c = Chance::from_seed(3);
        let [l, _] = branch(
            &mut c,
            &BranchArgs {
                det: 4.5,
                ..Default::default()
            },
        );
        assert_eq!(l.len(), 14);
    }

    #[test]
    fn twig_recurses_and_leafs() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(4));
        twig(&mut s, &mut c, [0.0, 0.0], 1, &TwigArgs::default());
        // 1 stroke + 5 leaves, twice for each of the 2 sub-twigs.
        assert_eq!(s.len(), 6 * 3);
    }

    #[test]
    fn leafless_twig_is_one_stroke() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(5));
        twig(
            &mut s,
            &mut c,
            [0.0, 0.0],
            0,
            &TwigArgs {
                lea: (false, 0.0),
                ..Default::default()
            },
        );
        assert_eq!(s.len(), 1);
    }
}
