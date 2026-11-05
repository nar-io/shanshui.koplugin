//! tree07: a dark, faceted tree — trunk and leaf blobs are shattered into
//! small triangles, each inked with its own noise-picked grey.

use std::f64::consts::PI;

use super::noise_pairs;
use crate::brush::{BlobStyle, blob_outline};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::{joined_reversed, mid_pt};
use crate::geom::triangulate::{Tri, triangulate};
use crate::ink::{Paint, PolyStyle, Sketch};

const RESO: usize = 10;
const WID: f64 = 4.0;

/// Pointed leaf mass.
fn crown(p: f64) -> f64 {
    if p <= 1.0 {
        2.75 * p * (1.0 - p).powf(1.0 / 1.8)
    } else {
        2.75 * (p - 2.0) * (p - 1.0).powf(1.0 / 1.8)
    }
}

/// Example: `tree07(&mut sk, &mut ch, x, y, 60.0)`.
pub fn tree07(sketch: &mut Sketch, chance: &mut Chance, x: f64, y: f64, hei: f64) {
    let ns = noise_pairs(chance, RESO);
    let mut leaves: Vec<Tri> = Vec::new();
    let (mut line1, mut line2): (Vec<Pt>, Vec<Pt>) = (Vec::new(), Vec::new());
    for (i, n) in ns.iter().enumerate() {
        let t = i as f64 / RESO as f64;
        let nx = x + t.sqrt() * 0.2 * 100.0;
        let ny = y - i as f64 * hei / RESO as f64;
        if i as f64 >= RESO as f64 / 4.0 {
            leaves.extend(leaf_mass(chance, [nx, ny], RESO - i));
        }
        line1.push([nx + (n[0] - 0.5) * WID - WID / 2.0, ny]);
        line2.push([nx + (n[1] - 0.5) * WID + WID / 2.0, ny]);
    }
    let mut shards = triangulate(&joined_reversed(&line1, &line2), 50.0, true);
    shards.extend(leaves);
    for tri in shards {
        let m = mid_pt(&tri);
        let gray = (chance.noise2(m[0] * 0.02, m[1] * 0.02) * 200.0 + 50.0) as u8;
        sketch.poly(tri.to_vec(), PolyStyle::filled(Paint::ink(gray, 0.8)));
    }
}

fn leaf_mass(chance: &mut Chance, at: Pt, rest: usize) -> Vec<Tri> {
    let bx = at[0] + (chance.random() - 0.5) * WID * 1.2 * rest as f64 * 0.5;
    let by = at[1] + (chance.random() - 0.5) * WID * 0.5;
    let len = chance.random() * 50.0 + 20.0;
    let wid = chance.random() * 12.0 + 12.0;
    let ang = -chance.random() * PI / 6.0;
    let outline = blob_outline(
        chance,
        bx,
        by,
        &BlobStyle {
            len,
            wid,
            ang,
            profile: &crown,
            ..Default::default()
        },
    );
    triangulate(&outline, 50.0, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faceted_tree_has_many_shards() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        tree07(&mut s, &mut c, 0.0, 0.0, 60.0);
        assert!(s.len() > 30, "only {}", s.len());
        assert!(s.polygons.iter().all(|p| p.pts.len() == 3));
    }

    #[test]
    fn crown_is_closed() {
        assert_eq!(crown(0.0), 0.0);
        assert!(crown(1.0).abs() < 1e-12 && crown(2.0).abs() < 1e-12);
    }
}
