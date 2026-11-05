//! tree03: the bent pines at the foot of mountains — a paper-filled trunk
//! wrapped in foliage dabs that widen towards the bottom.

use std::f64::consts::PI;

use super::{leaf_paint, noise_pairs};
use crate::brush::{BlobStyle, blob};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::joined_reversed;
use crate::ink::{Paint, PolyStyle, Sketch};

const RESO: usize = 10;
const WID: f64 = 5.0;

/// `ben(t)` bends the trunk sideways (×100) at relative height `t`.
/// Example: `tree03(&mut sk, &mut ch, x, y, 80.0, &|t| t * 0.05, col)`.
pub fn tree03(
    sketch: &mut Sketch,
    chance: &mut Chance,
    x: f64,
    y: f64,
    hei: f64,
    ben: &dyn Fn(f64) -> f64,
    col: Paint,
) {
    let ns = noise_pairs(chance, RESO);
    let mut foliage = Sketch::new();
    let (mut line1, mut line2): (Vec<Pt>, Vec<Pt>) = (Vec::new(), Vec::new());
    for (i, n) in ns.iter().enumerate() {
        let nx = x + ben(i as f64 / RESO as f64) * 100.0;
        let ny = y - i as f64 * hei / RESO as f64;
        if i >= RESO / 5 {
            foliage_level(&mut foliage, chance, [nx, ny], RESO - i, col);
        }
        let taper = (RESO - i) as f64 / RESO as f64;
        line1.push([nx + ((n[0] - 0.5) * WID - WID / 2.0) * taper, ny]);
        line2.push([nx + ((n[1] - 0.5) * WID + WID / 2.0) * taper, ny]);
    }
    let trunk = joined_reversed(&line1, &line2);
    sketch.poly(
        trunk,
        PolyStyle {
            fill: Paint::Paper,
            outline: col,
            width: 1.5,
        },
    );
    sketch.append(foliage);
}

/// Log-shaped spread: wide at the base, narrowing quickly upwards.
fn spread(t: f64) -> f64 {
    (50.0 * t + 1.0).ln() / 3.95
}

fn foliage_level(sketch: &mut Sketch, chance: &mut Chance, at: Pt, rest: usize, col: Paint) {
    for _ in 0..rest * 2 {
        let ox = chance.random() * WID * 2.0 * spread(rest as f64 / RESO as f64);
        let bx = at[0] + ox * chance.sign();
        let by = at[1] + (chance.random() - 0.5) * WID * 2.0;
        let wid = chance.random() * 6.0 + 3.0;
        let ang = (chance.random() - 0.5) * PI / 6.0;
        let style = BlobStyle {
            len: ox * 2.0,
            wid,
            ang,
            col: leaf_paint(chance, col),
            ..Default::default()
        };
        blob(sketch, chance, bx, by, &style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trunk_comes_first_and_hides() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        tree03(
            &mut s,
            &mut c,
            0.0,
            0.0,
            60.0,
            &|_| 0.0,
            Paint::ink(100, 0.4),
        );
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
        assert!(s.len() > 20);
    }

    #[test]
    fn bend_moves_the_top() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        tree03(
            &mut s,
            &mut c,
            0.0,
            0.0,
            60.0,
            &|t| t * 0.5,
            Paint::ink(100, 0.4),
        );
        let top = s.polygons[0]
            .pts
            .iter()
            .map(|p| p[0])
            .fold(f64::MIN, f64::max);
        assert!(top > 30.0, "top x {top}");
    }

    #[test]
    fn spread_shape() {
        assert_eq!(spread(0.0), 0.0);
        assert!(spread(1.0) > 0.99 && spread(1.0) < 1.0);
    }
}
