//! The eight tree species of `Tree.*`. Each function appends to a sketch in
//! world coordinates, `(x, y)` being the foot of the trunk.

mod bark;
mod branch;
mod branched;
mod simple;
mod tree03;
mod tree06;
mod tree07;
mod tree08;

pub use branched::{tree04, tree05};
pub use simple::{tree01, tree02};
pub use tree03::tree03;
pub use tree06::tree06;
pub use tree07::tree07;
pub use tree08::tree08;

use crate::brush::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::ink::{Paint, PolyStyle, Sketch};

/// Trunk wobble samples: `[noise(i/2), noise(i/2, 0.5)]` per level.
fn noise_pairs(chance: &Chance, reso: usize) -> Vec<[f64; 2]> {
    (0..reso)
        .map(|i| {
            [
                chance.noise1(i as f64 * 0.5),
                chance.noise2(i as f64 * 0.5, 0.5),
            ]
        })
        .collect()
}

/// Leaf ink: the trunk colour with `+ rand * 0.2` opacity, as the original
/// rebuilds `rgba(...)` strings from `leafcol`.
fn leaf_paint(chance: &mut Chance, col: Paint) -> Paint {
    col.with_alpha(chance.random() * 0.2 + col.alpha())
}

/// Constant-width profile used by the big trunks (`Math.sin(1)`).
fn sin_one(_t: f64) -> f64 {
    1f64.sin()
}

/// Paper-filled trunk silhouette plus its outline stroke. `pts` are
/// relative to `(x, y)`; `trim` drops the first and last point from the
/// stroke like `trmlist.splice(...)` does.
fn ink_trunk(
    sketch: &mut Sketch,
    chance: &mut Chance,
    origin: Pt,
    pts: &[Pt],
    alpha: f64,
    trim: bool,
) {
    let abs = translated(pts, origin[0], origin[1]);
    sketch.poly(abs.clone(), PolyStyle::paper());
    let line = if trim && abs.len() > 2 {
        &abs[1..abs.len() - 1]
    } else {
        &abs[..]
    };
    let col = Paint::ink(100, alpha + chance.random() * 0.1);
    let style = StrokeStyle {
        col,
        wid: 2.5,
        profile: &sin_one,
        noi: 0.9,
        out: 0.0,
    };
    stroke(sketch, chance, line, &style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_paint_adds_opacity() {
        let mut c = Chance::from_seed(1);
        let a = leaf_paint(&mut c, Paint::ink(100, 0.5)).alpha();
        assert!((0.5..=0.7).contains(&a), "alpha {a}");
    }

    #[test]
    fn trunk_has_paper_then_stroke() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        let pts: Vec<Pt> = (0..8).map(|i| [i as f64, -(i as f64) * 10.0]).collect();
        ink_trunk(&mut s, &mut c, [5.0, 5.0], &pts, 0.4, true);
        assert_eq!(s.len(), 2);
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
    }

    #[test]
    fn noise_pairs_len() {
        assert_eq!(noise_pairs(&Chance::from_seed(3), 10).len(), 10);
    }
}
