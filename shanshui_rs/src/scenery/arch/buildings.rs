//! The four building types: arch01 (pavilion with people), arch02 (tiered
//! house), arch03 (pagoda), arch04 (open kiosk).

use super::parts::{BoxArgs, boxy, hut};
use super::roofs::{RailArgs, Span, pagroof, rail, roof};
use crate::chance::Chance;
use crate::ink::Sketch;
use crate::scenery::man::{ManArgs, man};

/// Arguments of `arch01` as `flatDec` passes them.
#[derive(Clone, Copy, Debug)]
pub struct Arch01Args {
    pub wid: f64,
    pub hei: f64,
    pub per: f64,
}

/// Thatched pavilion on a platform, with zero to two people.
/// Example: `arch01(&mut sk, &mut ch, x, y, 0.3, &Arch01Args { wid: 180.0, hei: 90.0, per: 0.5 })`.
pub fn arch01(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    seed: f64,
    a: &Arch01Args,
) {
    let p = 0.4 + chance.random() * 0.2;
    hut(sketch, chance, xoff, yoff - a.hei, a.hei * p, a.wid);
    let walls = BoxArgs {
        hei: a.hei * (1.0 - p),
        wid: a.wid * 2.0 / 3.0,
        per: a.per,
        bot: false,
        ..Default::default()
    };
    boxy(sketch, chance, xoff, yoff, &walls);
    let span = Span {
        hei: 10.0,
        wid: a.wid,
        rot: 0.7,
        per: a.per * 2.0,
        wei: 1.0,
    };
    let back = RailArgs {
        span,
        seg: (3.0 + chance.random() * 3.0) as usize,
        tra: true,
        fro: false,
    };
    rail(sketch, chance, xoff, yoff, seed, &back);
    visitors(sketch, chance, xoff, yoff, a.wid);
    let front = RailArgs {
        span,
        seg: (3.0 + chance.random() * 3.0) as usize,
        tra: false,
        fro: true,
    };
    rail(sketch, chance, xoff, yoff, seed, &front);
}

fn visitors(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, wid: f64) {
    match chance.choice(&[0, 1, 1, 2]) {
        1 => {
            let x = xoff + chance.norm_rand(-wid / 3.0, wid / 3.0);
            let fli = chance.random() < 0.5;
            let args = ManArgs {
                fli,
                sca: 0.42,
                ..ManArgs::new(chance)
            };
            man(sketch, chance, x, yoff, &args);
        }
        2 => {
            let x = xoff + chance.norm_rand(-wid / 4.0, -wid / 5.0);
            let args = ManArgs {
                fli: false,
                sca: 0.42,
                ..ManArgs::new(chance)
            };
            man(sketch, chance, x, yoff, &args);
            let x = xoff + chance.norm_rand(wid / 5.0, wid / 4.0);
            let args = ManArgs {
                fli: true,
                sca: 0.42,
                ..ManArgs::new(chance)
            };
            man(sketch, chance, x, yoff, &args);
        }
        _ => {}
    }
}

/// Arguments of `arch02`.
#[derive(Clone, Copy, Debug)]
pub struct Arch02Args {
    pub hei: f64,
    pub wid: f64,
    pub rot: f64,
    pub per: f64,
    pub sto: usize,
    pub sty: u8,
}

impl Default for Arch02Args {
    fn default() -> Self {
        Self {
            hei: 10.0,
            wid: 50.0,
            rot: 0.3,
            per: 5.0,
            sto: 3,
            sty: 1,
        }
    }
}

/// Lattice spacing per window style.
fn lattice(sty: u8) -> ([usize; 2], [usize; 2]) {
    match sty {
        3 => ([1, 4], [1, 3]),
        _ => ([1, 5], [1, 2]),
    }
}

/// Tiered house: `sto` storeys, each with lattice windows and a roof.
pub fn arch02(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    _seed: f64,
    a: &Arch02Args,
) {
    let (hsp, vsp) = lattice(a.sty);
    let mut hoff = 0.0;
    for i in 0..a.sto {
        let shrink = 0.85f64.powi(i as i32);
        let walls = BoxArgs {
            tra: false,
            hei: a.hei,
            wid: a.wid * shrink,
            rot: a.rot,
            wei: 1.5,
            per: a.per,
            dec: Some((a.sty, hsp, vsp)),
            ..Default::default()
        };
        boxy(sketch, chance, xoff, yoff - hoff, &walls);
        let span = Span {
            hei: a.hei,
            wid: a.wid * 0.9f64.powi(i as i32),
            rot: a.rot,
            per: a.per,
            wei: 1.5,
        };
        roof(sketch, chance, xoff, yoff - hoff - a.hei, &span);
        hoff += a.hei * 1.5;
    }
}

/// Storeys of a pagoda or kiosk: walls, a rail, then a pagoda cap.
struct Tier {
    hei: f64,
    wid: f64,
    per: f64,
    tra: bool,
    dec: Option<(u8, [usize; 2], [usize; 2])>,
    rail_seg: usize,
    rail_wid: f64,
    rail_hei: f64,
    cap_hei: f64,
    step: f64,
}

fn tiers(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, sto: usize, t: &Tier) {
    let rot = 0.7;
    let mut hoff = 0.0;
    for i in 0..sto {
        let shrink = 0.85f64.powi(i as i32);
        let walls = BoxArgs {
            tra: t.tra,
            hei: t.hei,
            wid: t.wid * shrink,
            rot,
            wei: 1.5,
            per: t.per / 2.0,
            dec: t.dec,
            ..Default::default()
        };
        boxy(sketch, chance, xoff, yoff - hoff, &walls);
        let span = Span {
            hei: t.hei * t.rail_hei,
            wid: t.wid * shrink * t.rail_wid,
            rot,
            per: t.per / 2.0,
            wei: 0.5,
        };
        rail(
            sketch,
            chance,
            xoff,
            yoff - hoff,
            i as f64 * 0.2,
            &RailArgs {
                span,
                seg: t.rail_seg,
                tra: t.tra,
                fro: true,
            },
        );
        let cap = Span {
            hei: t.hei * t.cap_hei,
            wid: t.wid * 0.9f64.powi(i as i32),
            rot,
            per: t.per,
            wei: 1.5,
        };
        pagroof(sketch, chance, xoff, yoff - hoff - t.hei, &cap);
        hoff += t.hei * t.step;
    }
}

/// Pagoda on a summit. Example: `arch03(&mut sk, &mut ch, x, y, 0.0, 7, 50.0)`.
pub fn arch03(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    _seed: f64,
    sto: usize,
    wid: f64,
) {
    let t = Tier {
        hei: 10.0,
        wid,
        per: 5.0,
        tra: false,
        dec: Some((1, [1, 4], [1, 2])),
        rail_seg: 5,
        rail_wid: 1.1,
        rail_hei: 0.5,
        cap_hei: 1.5,
        step: 1.5,
    };
    tiers(sketch, chance, xoff, yoff, sto, &t);
}

/// Small open kiosk. Example: `arch04(&mut sk, &mut ch, x, y, 0.0, 2)`.
pub fn arch04(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    _seed: f64,
    sto: usize,
) {
    let t = Tier {
        hei: 15.0,
        wid: 30.0,
        per: 5.0,
        tra: true,
        dec: None,
        rail_seg: 3,
        rail_wid: 1.2,
        rail_hei: 1.0 / 3.0,
        cap_hei: 1.0,
        step: 1.2,
    };
    tiers(sketch, chance, xoff, yoff, sto, &t);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_building_draws() {
        let mut c = Chance::from_seed(1);
        let mut sizes = Vec::new();
        for kind in 0..4 {
            let mut s = Sketch::new();
            match kind {
                0 => arch01(
                    &mut s,
                    &mut c,
                    0.0,
                    0.0,
                    0.0,
                    &Arch01Args {
                        wid: 180.0,
                        hei: 90.0,
                        per: 0.5,
                    },
                ),
                1 => arch02(&mut s, &mut c, 0.0, 0.0, 0.0, &Arch02Args::default()),
                2 => arch03(&mut s, &mut c, 0.0, 0.0, 0.0, 5, 50.0),
                _ => arch04(&mut s, &mut c, 0.0, 0.0, 0.0, 2),
            }
            sizes.push(s.len());
        }
        assert!(sizes.iter().all(|n| *n > 5), "{sizes:?}");
    }

    #[test]
    fn taller_pagoda_draws_more() {
        let (mut a, mut b, mut c) = (Sketch::new(), Sketch::new(), Chance::from_seed(2));
        arch03(&mut a, &mut c, 0.0, 0.0, 0.0, 3, 50.0);
        arch03(&mut b, &mut c, 0.0, 0.0, 0.0, 7, 50.0);
        assert!(b.len() > a.len());
    }
}
