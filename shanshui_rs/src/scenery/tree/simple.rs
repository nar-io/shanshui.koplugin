//! tree01 (thin trunk with leaf dabs, the mid-slope forest) and tree02
//! (round clusters of leaves, the ridge-line moss).

use std::f64::consts::PI;

use super::{leaf_paint, noise_pairs};
use crate::brush::blob::leaf;
use crate::brush::{BlobStyle, blob};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::ink::{Paint, PolyStyle, Sketch};

const RESO: usize = 10;

/// Example: `tree01(&mut sk, &mut ch, x, y, 40.0, 2.0, Paint::ink(100, 0.4))`.
pub fn tree01(
    sketch: &mut Sketch,
    chance: &mut Chance,
    x: f64,
    y: f64,
    hei: f64,
    wid: f64,
    col: Paint,
) {
    let ns = noise_pairs(chance, RESO);
    let (mut line1, mut line2): (Vec<Pt>, Vec<Pt>) = (Vec::new(), Vec::new());
    for (i, n) in ns.iter().enumerate() {
        let ny = y - i as f64 * hei / RESO as f64;
        if i as f64 >= RESO as f64 / 4.0 {
            tree01_leaves(sketch, chance, [x, ny], wid, RESO - i, col);
        }
        line1.push([x + (n[0] - 0.5) * wid - wid / 2.0, ny]);
        line2.push([x + (n[1] - 0.5) * wid + wid / 2.0, ny]);
    }
    sketch.poly(line1, PolyStyle::outlined(col, 1.5));
    sketch.poly(line2, PolyStyle::outlined(col, 1.5));
}

fn tree01_leaves(
    sketch: &mut Sketch,
    chance: &mut Chance,
    at: Pt,
    wid: f64,
    rest: usize,
    col: Paint,
) {
    let r = rest as f64;
    for _ in 0..rest.div_ceil(5) {
        let bx = at[0] + (chance.random() - 0.5) * wid * 1.2 * r;
        let by = at[1] + (chance.random() - 0.5) * wid;
        let len = chance.random() * 20.0 * r * 0.2 + 10.0;
        let bw = chance.random() * 6.0 + 3.0;
        let ang = (chance.random() - 0.5) * PI / 6.0;
        // tree01 rounds the opacity to one decimal (`toFixed(1)`).
        let a = (leaf_paint(chance, col).alpha() * 10.0).round() / 10.0;
        let style = BlobStyle {
            len,
            wid: bw,
            ang,
            col: col.with_alpha(a),
            ..Default::default()
        };
        blob(sketch, chance, bx, by, &style);
    }
}

/// `clu` leaf clusters scattered around `(x, y)`.
/// Example: `tree02(&mut sk, &mut ch, x, y, 5, Paint::ink(100, 0.5))`.
pub fn tree02(sketch: &mut Sketch, chance: &mut Chance, x: f64, y: f64, clu: usize, col: Paint) {
    let (hei, wid) = (16.0, 8.0);
    let spread = clu as f64 * 4.0;
    for _ in 0..clu {
        let bx = x + chance.gaussian() * spread;
        let by = y + chance.gaussian() * spread;
        let bw = chance.random() * wid * 0.75 + wid * 0.5;
        let len = chance.random() * hei * 0.75 + hei * 0.5;
        let style = BlobStyle {
            ang: PI / 2.0,
            profile: &leaf,
            wid: bw,
            len,
            col,
            ..Default::default()
        };
        blob(sketch, chance, bx, by, &style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree01_draws_trunk_lines_and_leaves() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        tree01(&mut s, &mut c, 0.0, 0.0, 50.0, 3.0, Paint::ink(100, 0.5));
        let outlines = s
            .polygons
            .iter()
            .filter(|p| p.style.fill == Paint::Clear)
            .count();
        assert_eq!(outlines, 2);
        assert!(s.len() > 2);
    }

    #[test]
    fn tree01_grows_upwards() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        tree01(&mut s, &mut c, 0.0, 100.0, 50.0, 3.0, Paint::ink(100, 0.5));
        let b = s.bbox().unwrap();
        assert!(b[1] < 100.0 - 30.0, "top {}", b[1]);
    }

    #[test]
    fn tree02_draws_clu_blobs() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(3));
        tree02(&mut s, &mut c, 0.0, 0.0, 4, Paint::ink(100, 0.5));
        assert_eq!(s.len(), 4);
    }
}
