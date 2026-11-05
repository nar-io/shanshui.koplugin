//! `Arch.boat01`: a sampan with a boatman poling it.

use std::f64::consts::PI;

use crate::brush::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::{joined_reversed, translated};
use crate::ink::{Paint, PolyStyle, Sketch};
use crate::scenery::man::{Hat, Item, ManArgs, man};

/// `sca` shrinks boats further up the scroll (further away); `fli` points
/// the bow left. Example: `boat01(&mut sk, &mut ch, x, y, 0.6, false)`.
pub fn boat01(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, sca: f64, fli: bool) {
    let dir = if fli { -1.0 } else { 1.0 };
    let boatman = ManArgs {
        item: Item::Pole,
        hat: Hat::Brim,
        sca: 0.5 * sca,
        fli: !fli,
        len: [0.0, 30.0, 20.0, 30.0, 10.0, 30.0, 30.0, 30.0, 30.0],
        ..ManArgs::new(chance)
    };
    man(sketch, chance, xoff + 20.0 * sca * dir, yoff, &boatman);
    let hull = hull(sca, dir);
    sketch.poly(translated(&hull, xoff, yoff), PolyStyle::paper());
    let wave = |x: f64| (x * PI * 2.0).sin();
    let style = StrokeStyle {
        wid: 1.0,
        profile: &wave,
        col: Paint::ink(100, 0.4),
        ..Default::default()
    };
    stroke(sketch, chance, &translated(&hull, xoff, yoff), &style);
}

fn hull(sca: f64, dir: f64) -> Vec<Pt> {
    let len = 120.0;
    let depth = |x: f64, k: f64| (x * PI).sin().max(0.0).sqrt() * k * sca;
    let (mut top, mut bottom) = (Vec::new(), Vec::new());
    let mut i = 0.0;
    while i < len * sca {
        top.push([i * dir, depth(i / len, 7.0)]);
        bottom.push([i * dir, depth(i / len, 10.0)]);
        i += 5.0 * sca;
    }
    joined_reversed(&top, &bottom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hull_follows_direction() {
        assert!(hull(1.0, -1.0).iter().all(|p| p[0] <= 0.0));
        assert_eq!(hull(1.0, 1.0).len(), 48);
    }

    #[test]
    fn boat_has_boatman_and_hull() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        boat01(&mut s, &mut c, 0.0, 0.0, 0.6, false);
        assert!(s.len() > 10);
    }
}
