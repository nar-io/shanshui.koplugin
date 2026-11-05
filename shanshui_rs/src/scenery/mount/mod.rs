//! Mountains, plateaus, distant ranges and rocks (`Mount.*`).
//!
//! Mountains are built as a mesh: `mesh[i][j]` is the j-th point of the
//! i-th contour, from the outer ridge line (`i = 0`) shrinking inwards.

mod dist_mount;
mod flat_dec;
mod flat_mount;
mod foot;
mod mountain;
mod rock;
mod vegetate;

pub use dist_mount::dist_mount;
pub use flat_mount::{FlatMountArgs, flat_mount};
pub use mountain::{MountainArgs, mountain};
pub use rock::{RockArgs, rock};

use crate::brush::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::ink::{Paint, PolyStyle, Sketch};

/// Contours of a mountain-like shape, outermost first.
pub type Mesh = Vec<Vec<Pt>>;

/// Paper silhouette (ridge closed down to `(0, base)`) plus the inked
/// ridge line — the "WHITE BG" + "OUTLINE" pair every landform starts with.
fn silhouette(sketch: &mut Sketch, chance: &mut Chance, ridge: &[Pt], origin: Pt, base: f64) {
    let mut fill = translated(ridge, origin[0], origin[1]);
    fill.push([origin[0], origin[1] + base]);
    sketch.poly(fill, PolyStyle::paper());
    let style = StrokeStyle {
        col: Paint::ink(100, 0.3),
        noi: 1.0,
        wid: 3.0,
        ..Default::default()
    };
    stroke(
        sketch,
        chance,
        &translated(ridge, origin[0], origin[1]),
        &style,
    );
}

/// Vegetation ink: grey whose opacity drifts with position.
fn foliage_ink(chance: &Chance, p: Pt, base: f64) -> Paint {
    Paint::ink(
        100,
        chance.noise2(0.01 * p[0], 0.01 * p[1]) * 0.5 * 0.3 + base,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silhouette_is_paper_then_stroke() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        let ridge: Vec<Pt> = (0..10)
            .map(|i| [i as f64 * 10.0 - 50.0, -((i as f64 - 5.0).abs())])
            .collect();
        silhouette(&mut s, &mut c, &ridge, [100.0, 100.0], 40.0);
        assert_eq!(s.len(), 2);
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
        assert_eq!(*s.polygons[0].pts.last().unwrap(), [100.0, 140.0]);
    }

    #[test]
    fn foliage_ink_in_band() {
        let c = Chance::from_seed(2);
        let a = foliage_ink(&c, [10.0, 20.0], 0.5).alpha();
        assert!((0.5..=0.65).contains(&a), "alpha {a}");
    }
}
