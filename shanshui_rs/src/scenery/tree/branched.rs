//! tree04 (big gnarled tree) and tree05 (tall tree with level boughs):
//! a barked trunk from which limbs sprout, each carrying twigs. They share
//! the same construction and differ only in the rules below.

use std::f64::consts::PI;

use super::bark::barkify;
use super::branch::{BranchArgs, TwigArgs, branch, twig};
use super::ink_trunk;
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::joined_reversed;
use crate::ink::Sketch;

/// A limb chosen to sprout at trunk outline index `i`.
struct Limb {
    ang: f64,
    hei: f64,
    ben: f64,
}

/// What distinguishes tree04 from tree05.
struct Species {
    trunk_ben: f64,
    pick: fn(&mut Chance, usize, usize, f64) -> Option<Limb>,
    bark_limbs: bool,
    twig_at: fn(&mut Chance, usize, usize) -> bool,
    twig_dep: u32,
    twig_sca: f64,
    lea: (bool, f64),
}

fn pick04(chance: &mut Chance, i: usize, n: usize, hei: f64) -> Option<Limb> {
    let nf = n as f64;
    let middle = i as f64 >= nf * 0.3 && i as f64 <= nf * 0.7 && chance.random() < 0.1;
    if !(middle || i as f64 == nf / 2.0 - 1.0) {
        return None;
    }
    let ang = PI * 0.2 - PI * 1.4 * f64::from(u8::from(i as f64 > nf / 2.0));
    Some(Limb {
        ang,
        hei: hei * (chance.random() + 1.0) * 0.3,
        ben: PI * 0.2,
    })
}

fn pick05(chance: &mut Chance, i: usize, n: usize, hei: f64) -> Option<Limb> {
    let nf = n as f64;
    let p = (i as f64 - nf * 0.5).abs() / (nf * 0.5);
    let band = i as f64 >= nf * 0.2 && i as f64 <= nf * 0.8 && i.is_multiple_of(3);
    if !((band && chance.random() > p) || i as f64 == nf / 2.0 - 1.0) {
        return None;
    }
    let bar = chance.random() * 0.2;
    let ang = -bar * PI - (1.0 - bar * 2.0) * PI * f64::from(u8::from(i as f64 > nf / 2.0));
    Some(Limb {
        ang,
        hei: hei * (0.3 * p - chance.random() * 0.05),
        ben: 0.5,
    })
}

const TREE04: Species = Species {
    trunk_ben: PI * 0.2,
    pick: pick04,
    bark_limbs: true,
    twig_at: |chance, j, n| chance.random() < 0.2 || j == n - 1,
    twig_dep: 1,
    twig_sca: 0.5,
    lea: (true, 12.0),
};

const TREE05: Species = Species {
    trunk_ben: 0.0,
    pick: pick05,
    bark_limbs: false,
    twig_at: |_, j, n| j % 20 == 0 || j == n - 1,
    twig_dep: 0,
    twig_sca: 0.2,
    lea: (true, 5.0),
};

/// Big gnarled tree (300 tall). Example: `tree04(&mut sk, &mut ch, x, y)`.
pub fn tree04(sketch: &mut Sketch, chance: &mut Chance, x: f64, y: f64) {
    branched(sketch, chance, [x, y], 300.0, 6.0, &TREE04);
}

/// Tall tree with level boughs. Example: `tree05(&mut sk, &mut ch, x, y, 200.0)`.
pub fn tree05(sketch: &mut Sketch, chance: &mut Chance, x: f64, y: f64, hei: f64) {
    branched(sketch, chance, [x, y], hei, 5.0, &TREE05);
}

/// Bark and twigs paint over the trunk, so they collect in their own
/// sketches and are appended last.
struct Layers {
    bark: Sketch,
    twigs: Sketch,
}

fn branched(
    sketch: &mut Sketch,
    chance: &mut Chance,
    origin: Pt,
    hei: f64,
    wid: f64,
    sp: &Species,
) {
    let mut layers = Layers {
        bark: Sketch::new(),
        twigs: Sketch::new(),
    };
    let trunk = branch(
        chance,
        &BranchArgs {
            hei,
            wid,
            ang: -PI / 2.0,
            ben: sp.trunk_ben,
            ..Default::default()
        },
    );
    barkify(&mut layers.bark, chance, origin, &trunk);
    let trunk = joined_reversed(&trunk[0], &trunk[1]);
    let mut outline = Vec::with_capacity(trunk.len());
    for (i, &at) in trunk.iter().enumerate() {
        match (sp.pick)(chance, i, trunk.len(), hei) {
            Some(limb) => {
                outline.extend(sprout(&mut layers, chance, origin, at, &limb, hei, wid, sp))
            }
            None => outline.push(at),
        }
    }
    ink_trunk(sketch, chance, origin, &outline, 0.4, true);
    sketch.append(layers.bark);
    sketch.append(layers.twigs);
}

/// Grows one limb at trunk point `at`; returns its outline (trunk-relative).
#[allow(clippy::too_many_arguments)]
fn sprout(
    layers: &mut Layers,
    chance: &mut Chance,
    origin: Pt,
    at: Pt,
    limb: &Limb,
    hei: f64,
    wid: f64,
    sp: &Species,
) -> Vec<Pt> {
    let args = BranchArgs {
        hei: limb.hei,
        wid: wid * 0.5,
        ang: limb.ang,
        ben: limb.ben,
        ..Default::default()
    };
    let [mut l, mut r] = branch(chance, &args);
    l.remove(0);
    r.remove(0);
    let base = [origin[0] + at[0], origin[1] + at[1]];
    if sp.bark_limbs {
        barkify(&mut layers.bark, chance, base, &[l.clone(), r.clone()]);
    }
    let up = limb.ang > -PI / 2.0;
    let twig_args = TwigArgs {
        wid: hei / 300.0,
        ang: if up { limb.ang } else { limb.ang + PI },
        sca: sp.twig_sca * hei / 300.0,
        dir: if up { 1.0 } else { -1.0 },
        lea: sp.lea,
    };
    for (j, p) in l.iter().enumerate() {
        if (sp.twig_at)(chance, j, l.len()) {
            twig(
                &mut layers.twigs,
                chance,
                [base[0] + p[0], base[1] + p[1]],
                sp.twig_dep,
                &twig_args,
            );
        }
    }
    joined_reversed(&l, &r)
        .iter()
        .map(|p| [p[0] + at[0], p[1] + at[1]])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::Paint;

    #[test]
    fn tree04_has_trunk_bark_and_twigs() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        tree04(&mut s, &mut c, 0.0, 0.0);
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
        assert!(s.len() > 50, "only {}", s.len());
        let b = s.bbox().unwrap();
        assert!(b[1] < -250.0, "top {}", b[1]);
    }

    #[test]
    fn tree05_always_has_middle_bough() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        tree05(&mut s, &mut c, 0.0, 0.0, 150.0);
        assert!(s.len() > 10, "only {}", s.len());
    }

    #[test]
    fn pick04_forces_middle() {
        let mut c = Chance::from_seed(3);
        assert!(pick04(&mut c, 29, 60, 300.0).is_some());
        assert!(pick04(&mut c, 0, 60, 300.0).is_none());
    }
}
