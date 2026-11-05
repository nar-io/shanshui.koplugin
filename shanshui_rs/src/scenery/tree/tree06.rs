//! tree06: a fractal tree — every limb may sprout smaller barked limbs,
//! three levels deep, with leafless twigs.

use std::f64::consts::PI;

use super::bark::barkify;
use super::branch::{BranchArgs, TwigArgs, branch, twig};
use super::ink_trunk;
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::joined_reversed;
use crate::ink::Sketch;

/// Example: `tree06(&mut sk, &mut ch, x, y, 90.0)`.
pub fn tree06(sketch: &mut Sketch, chance: &mut Chance, x: f64, y: f64, hei: f64) {
    let mut grow = Growth {
        bark: Sketch::new(),
        twigs: Sketch::new(),
    };
    let limb = Limb {
        hei,
        wid: 6.0,
        ang: -PI / 2.0,
        ben: 0.0,
    };
    let outline = grow.frac(chance, [x, y], 3, &limb);
    ink_trunk(sketch, chance, [x, y], &outline, 0.4, true);
    sketch.append(grow.bark);
    sketch.append(grow.twigs);
}

#[derive(Clone, Copy)]
struct Limb {
    hei: f64,
    wid: f64,
    ang: f64,
    ben: f64,
}

struct Growth {
    bark: Sketch,
    twigs: Sketch,
}

impl Growth {
    /// Outline of a limb and all its sub-limbs, relative to `origin`.
    fn frac(&mut self, chance: &mut Chance, origin: Pt, dep: u32, limb: &Limb) -> Vec<Pt> {
        let args = BranchArgs {
            hei: limb.hei,
            wid: limb.wid,
            ang: limb.ang,
            ben: limb.ben,
            det: limb.hei / 20.0,
        };
        let edges = branch(chance, &args);
        barkify(&mut self.bark, chance, origin, &edges);
        let trunk = joined_reversed(&edges[0], &edges[1]);
        let mut out = Vec::with_capacity(trunk.len());
        for (i, &at) in trunk.iter().enumerate() {
            if dep > 0 && sprouts(chance, i, trunk.len()) {
                out.extend(self.sub_limb(
                    chance,
                    origin,
                    at,
                    dep,
                    limb,
                    i as f64 > trunk.len() as f64 / 2.0,
                ));
            } else {
                out.push(at);
            }
        }
        out
    }

    fn sub_limb(
        &mut self,
        chance: &mut Chance,
        origin: Pt,
        at: Pt,
        dep: u32,
        limb: &Limb,
        right: bool,
    ) -> Vec<Pt> {
        let bar = 0.02 + chance.random() * 0.08;
        let ba = bar * PI - bar * 2.0 * PI * f64::from(u8::from(right));
        let child = Limb {
            hei: limb.hei * (0.7 + chance.random() * 0.2),
            wid: limb.wid * 0.6,
            ang: limb.ang + ba,
            ben: 0.55,
        };
        let base = [origin[0] + at[0], origin[1] + at[1]];
        let sub = self.frac(chance, base, dep - 1, &child);
        for p in &sub {
            if chance.random() < 0.03 {
                let args = TwigArgs {
                    ang: ba * (chance.random() * 0.5 + 0.75),
                    sca: 0.3,
                    dir: if ba > 0.0 { 1.0 } else { -1.0 },
                    lea: (false, 0.0),
                    ..Default::default()
                };
                twig(
                    &mut self.twigs,
                    chance,
                    [base[0] + p[0], base[1] + p[1]],
                    2,
                    &args,
                );
            }
        }
        sub.iter().map(|p| [p[0] + at[0], p[1] + at[1]]).collect()
    }
}

/// Rare sprouts in the middle band, plus two forced ones by the tip.
fn sprouts(chance: &mut Chance, i: usize, n: usize) -> bool {
    let nf = n as f64;
    let half = n / 2;
    (chance.random() < 0.025 && i as f64 >= nf * 0.2 && i as f64 <= nf * 0.8)
        || i + 1 == half
        || i == half + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::Paint;

    #[test]
    fn grows_a_tree() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        tree06(&mut s, &mut c, 0.0, 0.0, 80.0);
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
        assert!(s.len() > 20, "only {}", s.len());
    }

    #[test]
    fn sprouts_forced_near_middle() {
        let mut c = Chance::from_seed(2);
        assert!(sprouts(&mut c, 9, 20));
        assert!(sprouts(&mut c, 11, 20));
    }
}
