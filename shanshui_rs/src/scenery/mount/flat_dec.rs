//! `Mount.flatDec`: furnishes a plateau top with one of five themes
//! (boulders, a row of tall trees, big gnarled trees, fractal trees or a
//! dark grove), plus moss and sometimes a pavilion.

use super::rock::{RockArgs, rock};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::ink::Paint;
use crate::ink::Sketch;
use crate::scenery::arch::{Arch01Args, arch01};
use crate::scenery::tree::{tree02, tree04, tree05, tree06, tree07, tree08};

/// Bounds of the flat ground, relative to the plateau origin.
#[derive(Clone, Copy, Debug)]
pub struct Ground {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl Ground {
    fn mid_y(&self) -> f64 {
        (self.ymin + self.ymax) / 2.0
    }

    fn rand_x(&self, chance: &mut Chance) -> f64 {
        chance.norm_rand(self.xmin, self.xmax)
    }

    /// A random sub-range of `[xmin, xmax]` (start in the left half, end
    /// in the right half).
    fn stretch(&self, chance: &mut Chance) -> (f64, f64) {
        let (pmin, pmax) = (chance.random() * 0.5, chance.random() * 0.5 + 0.5);
        let at = |p: f64| self.xmin * (1.0 - p) + self.xmax * p;
        (at(pmin), at(pmax))
    }
}

/// Example: `flat_dec(&mut sk, &mut ch, [xoff, yoff], &ground)`.
pub fn flat_dec(sketch: &mut Sketch, chance: &mut Chance, origin: Pt, g: &Ground) {
    let theme = chance.choice(&[0, 0, 1, 2, 3, 4]);
    pebbles(sketch, chance, origin, g);
    saplings(sketch, chance, origin, g);
    match theme {
        0 => boulders(sketch, chance, origin, g, 3.0),
        1 => {
            tree_row(sketch, chance, origin, g);
            boulders(sketch, chance, origin, g, 4.0);
        }
        2 => gnarled(sketch, chance, origin, g),
        3 => fractal_grove(sketch, chance, origin, g),
        _ => dark_grove(sketch, chance, origin, g),
    }
    moss(sketch, chance, origin, g);
    if chance.choice(&[0, 0, 0, 0, 1]) == 1 && theme != 4 {
        pavilion(sketch, chance, origin, g);
    }
}

fn big_rock(chance: &mut Chance) -> RockArgs {
    RockArgs {
        wid: 50.0 + chance.random() * 20.0,
        hei: 40.0 + chance.random() * 20.0,
        sha: 5,
        ..Default::default()
    }
}

fn pebbles(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    let count = chance.random() * 5.0;
    let mut j = 0.0;
    while j < count {
        let x = o[0] + g.rand_x(chance);
        let y = o[1] + g.mid_y() + chance.norm_rand(-10.0, 10.0) + 10.0;
        let seed = chance.random() * 100.0;
        let args = RockArgs {
            wid: 10.0 + chance.random() * 20.0,
            hei: 10.0 + chance.random() * 20.0,
            sha: 2,
            ..Default::default()
        };
        rock(sketch, chance, x, y, seed, &args);
        j += 1.0;
    }
}

fn saplings(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    for _ in 0..chance.choice(&[0, 0, 1, 2]) {
        let xr = o[0] + g.rand_x(chance);
        let yr = o[1] + g.mid_y() + chance.norm_rand(-5.0, 5.0) + 20.0;
        let count = 2.0 + chance.random() * 3.0;
        let mut k = 0.0;
        while k < count {
            let x = xr + chance.norm_rand(-30.0, 30.0).clamp(g.xmin, g.xmax);
            let hei = 60.0 + chance.random() * 40.0;
            tree08(sketch, chance, x, yr, hei);
            k += 1.0;
        }
    }
}

fn boulders(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground, most: f64) {
    let count = chance.random() * most;
    let mut j = 0.0;
    while j < count {
        let x = o[0] + g.rand_x(chance);
        let y = o[1] + g.mid_y() + chance.norm_rand(-5.0, 5.0) + 20.0;
        let seed = chance.random() * 100.0;
        let args = big_rock(chance);
        rock(sketch, chance, x, y, seed, &args);
        j += 1.0;
    }
}

fn tree_row(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    let (xmin, xmax) = g.stretch(chance);
    let mut i = xmin;
    while i < xmax {
        let x = o[0] + i + 20.0 * chance.norm_rand(-1.0, 1.0);
        let hei = 100.0 + chance.random() * 200.0;
        tree05(sketch, chance, x, o[1] + g.mid_y() + 20.0, hei);
        i += 30.0;
    }
}

fn gnarled(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    for i in 0..chance.choice(&[1, 1, 1, 1, 2, 2, 3]) {
        let xr = g.rand_x(chance);
        let yr = g.mid_y();
        tree04(sketch, chance, o[0] + xr, o[1] + yr + 20.0);
        let count = chance.random() * 2.0;
        let mut j = 0.0;
        while j < count {
            let x = o[0] + (xr + chance.norm_rand(-50.0, 50.0)).clamp(g.xmin, g.xmax);
            let y = o[1] + yr + chance.norm_rand(-5.0, 5.0) + 20.0;
            let seed = j * i as f64 * chance.random() * 100.0;
            let args = big_rock(chance);
            rock(sketch, chance, x, y, seed, &args);
            j += 1.0;
        }
    }
}

fn fractal_grove(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    for _ in 0..chance.choice(&[1, 1, 1, 1, 2, 2, 3]) {
        let x = o[0] + g.rand_x(chance);
        let hei = 60.0 + chance.random() * 60.0;
        tree06(sketch, chance, x, o[1] + g.mid_y(), hei);
    }
}

fn dark_grove(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    let (xmin, xmax) = g.stretch(chance);
    let mut i = xmin;
    while i < xmax {
        let x = o[0] + i + 20.0 * chance.norm_rand(-1.0, 1.0);
        let y = o[1] + g.mid_y() + chance.norm_rand(-1.0, 1.0);
        let hei = chance.norm_rand(40.0, 80.0);
        tree07(sketch, chance, x, y, hei);
        i += 20.0;
    }
}

fn moss(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    let count = 50.0 * chance.random();
    let mut i = 0.0;
    while i < count {
        let x = o[0] + g.rand_x(chance);
        let y = o[1] + chance.norm_rand(g.ymin, g.ymax);
        tree02(sketch, chance, x, y, 5, Paint::ink(100, 0.5));
        i += 1.0;
    }
}

fn pavilion(sketch: &mut Sketch, chance: &mut Chance, o: Pt, g: &Ground) {
    let x = o[0] + g.rand_x(chance);
    let seed = chance.random();
    let args = Arch01Args {
        wid: chance.norm_rand(160.0, 200.0),
        hei: chance.norm_rand(80.0, 100.0),
        per: chance.random(),
    };
    arch01(sketch, chance, x, o[1] + g.mid_y() + 20.0, seed, &args);
}

#[cfg(test)]
mod tests {
    use super::*;

    const GROUND: Ground = Ground {
        xmin: -200.0,
        ymin: -60.0,
        xmax: 200.0,
        ymax: -40.0,
    };

    #[test]
    fn decorates_something() {
        let mut total = 0;
        for seed in 0..5 {
            let (mut s, mut c) = (Sketch::new(), Chance::from_seed(seed));
            flat_dec(&mut s, &mut c, [0.0, 600.0], &GROUND);
            total += s.len();
        }
        assert!(total > 50, "only {total}");
    }

    #[test]
    fn stretch_is_ordered_and_inside() {
        let mut c = Chance::from_seed(1);
        let (a, b) = GROUND.stretch(&mut c);
        assert!(GROUND.xmin <= a && a <= b && b <= GROUND.xmax);
    }
}
