//! `Mount.mountain`: the main peaks — mesh, silhouette, feet, cun texture,
//! then forests, buildings, pylons and rocks placed by noise rules.

use std::f64::consts::PI;

use super::foot::foot;
use super::rock::{RockArgs, rock};
use super::vegetate::{in_grove, sites};
use super::{Mesh, foliage_ink, silhouette};
use crate::brush::{TextureStyle, texture};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::ink::Sketch;
use crate::scenery::arch::{Arch02Args, arch02, arch03, arch04, transmission_tower};
use crate::scenery::tree::{tree01, tree02, tree03};

const RESO: [usize; 2] = [10, 50];

/// Arguments of `mountain(x, y, seed, args)`; `None` means "random", as in
/// the original's defaults.
#[derive(Clone, Copy, Debug)]
pub struct MountainArgs {
    pub hei: Option<f64>,
    pub wid: Option<f64>,
    pub tex: usize,
    pub veg: bool,
}

impl Default for MountainArgs {
    fn default() -> Self {
        Self {
            hei: None,
            wid: None,
            tex: 200,
            veg: true,
        }
    }
}

/// Example: `mountain(&mut sk, &mut ch, 400.0, 500.0, 1.3, &MountainArgs::default())`.
pub fn mountain(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    seed: f64,
    a: &MountainArgs,
) {
    let hei = a.hei.unwrap_or_else(|| 100.0 + chance.random() * 400.0);
    let wid = a.wid.unwrap_or_else(|| 400.0 + chance.random() * 200.0);
    let mesh = mountain_mesh(chance, yoff, seed, hei, wid);
    let m = Mount {
        mesh: &mesh,
        origin: [xoff, yoff],
        seed,
        hei,
    };
    m.rim(sketch, chance);
    silhouette(sketch, chance, &mesh[0], m.origin, RESO[0] as f64 * 4.0);
    foot(sketch, chance, &mesh, m.origin);
    let sha = chance.choice(&[0, 0, 0, 0, 5]);
    let shifted: Mesh = mesh.iter().map(|row| translated(row, xoff, yoff)).collect();
    texture(
        sketch,
        chance,
        &shifted,
        &TextureStyle {
            tex: a.tex,
            sha,
            ..Default::default()
        },
    );
    m.crown(sketch, chance);
    if a.veg {
        m.forest(sketch, chance);
        m.pines(sketch, chance);
    }
    m.buildings(sketch, chance);
    m.rocks(sketch, chance);
}

/// Contours of a cosine hump modulated by noise, stacked downwards.
fn mountain_mesh(chance: &mut Chance, yoff: f64, seed: f64, hei: f64, wid: f64) -> Mesh {
    let mut hoff = 0.0;
    (0..RESO[0])
        .map(|j| {
            hoff += chance.random() * yoff / 100.0;
            let p = 1.0 - j as f64 / RESO[0] as f64;
            (0..RESO[1])
                .map(|i| {
                    let x = (i as f64 / RESO[1] as f64 - 0.5) * PI;
                    let y = x.cos() * chance.noise(x + 10.0, j as f64 * 0.15, seed);
                    [x / PI * wid * p, -y * hei * p + hoff]
                })
                .collect()
        })
        .collect()
}

/// A built mountain; each method is one `vegetate` pass of the original.
struct Mount<'a> {
    mesh: &'a Mesh,
    origin: Pt,
    seed: f64,
    hei: f64,
}

impl Mount<'_> {
    fn at(&self, p: Pt) -> Pt {
        [p[0] + self.origin[0], p[1] + self.origin[1]]
    }

    fn rel_height(&self, i: usize, j: usize) -> f64 {
        self.mesh[i][j][1].abs() / self.hei
    }

    /// Moss peeking over the ridge (drawn before the paper silhouette).
    fn rim(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            let ns = chance.noise2(j as f64 * 0.1, self.seed);
            i == 0 && ns.powi(3) < 0.1 && self.rel_height(i, j) > 0.2
        });
        for p in spots {
            let [x, y] = self.at(p);
            tree02(sketch, chance, x, y - 5.0, 2, foliage_ink(chance, p, 0.5));
        }
    }

    /// Moss on the high slopes.
    fn crown(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            let ns = chance.noise(i as f64 * 0.1, j as f64 * 0.1, self.seed + 2.0);
            ns.powi(3) < 0.1 && self.rel_height(i, j) > 0.5
        });
        for p in spots {
            let [x, y] = self.at(p);
            tree02(sketch, chance, x, y, 5, foliage_ink(chance, p, 0.5));
        }
    }

    /// Groves of thin trees on the lower slopes.
    fn forest(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            let ns = chance.noise(i as f64 * 0.2, j as f64 * 0.05, self.seed);
            j % 2 == 1 && ns.powi(4) < 0.012 && self.rel_height(i, j) < 0.3
        });
        for k in 0..spots.len() {
            if !in_grove(&spots, k, 30.0) {
                continue;
            }
            let p = spots[k];
            let ht = (self.hei + p[1]) / self.hei * 70.0;
            let ht = ht * 0.3 + chance.random() * ht * 0.7;
            let wid = chance.random() * 3.0 + 1.0;
            let [x, y] = self.at(p);
            tree01(sketch, chance, x, y, ht, wid, foliage_ink(chance, p, 0.3));
        }
    }

    /// Bent pines at both ends of each contour.
    fn pines(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            let ns = chance.noise(i as f64 * 0.2, j as f64 * 0.05, self.seed);
            (j == 0 || j == self.mesh[i].len() - 1) && ns.powi(4) < 0.012
        });
        for p in spots {
            let ht = (self.hei + p[1]) / self.hei * 120.0;
            let ht = ht * 0.5 + chance.random() * ht * 0.5;
            let bc = chance.random() * 0.1;
            let [x, y] = self.at(p);
            tree03(
                sketch,
                chance,
                x,
                y,
                ht,
                &|t| t * bc,
                foliage_ink(chance, p, 0.3),
            );
        }
    }

    fn buildings(&self, sketch: &mut Sketch, chance: &mut Chance) {
        self.houses(sketch, chance);
        self.pagodas(sketch, chance);
        self.pylons(sketch, chance);
    }

    /// Houses and pavilions near the edges of lower contours.
    fn houses(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            let ns = chance.noise(i as f64 * 0.2, j as f64 * 0.05, self.seed + 10.0);
            i != 0 && (j == 1 || j == self.mesh[i].len() - 2) && ns.powi(4) < 0.008
        });
        for p in spots {
            let [x, y] = self.at(p);
            match chance.choice(&[0, 0, 1, 1, 1, 2]) {
                1 => {
                    let args = Arch02Args {
                        wid: chance.norm_rand(40.0, 70.0),
                        sto: chance.choice(&[1, 2, 2, 3]),
                        rot: chance.random(),
                        sty: chance.choice(&[1, 2, 3]),
                        ..Default::default()
                    };
                    arch02(sketch, chance, x, y, self.seed, &args);
                }
                2 => {
                    let sto = chance.choice(&[1, 1, 1, 2, 2]);
                    arch04(sketch, chance, x, y, self.seed, sto);
                }
                _ => {}
            }
        }
    }

    /// A rare pagoda on the summit.
    fn pagodas(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            i == 1
                && (j as f64 - self.mesh[i].len() as f64 / 2.0).abs() < 1.0
                && chance.random() < 0.02
        });
        for p in spots {
            let [x, y] = self.at(p);
            let sto = chance.choice(&[5, 7]);
            let wid = 40.0 + chance.random() * 20.0;
            arch03(sketch, chance, x, y, self.seed, sto, wid);
        }
    }

    /// Transmission towers — the modern world creeping into the scroll.
    fn pylons(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            let ns = chance.noise(i as f64 * 0.2, j as f64 * 0.05, self.seed + 20.0 * PI);
            i % 2 == 0 && (j == 1 || j == self.mesh[i].len() - 2) && ns.powi(4) < 0.002
        });
        for p in spots {
            let [x, y] = self.at(p);
            transmission_tower(sketch, chance, x, y);
        }
    }

    /// Boulders around the base.
    fn rocks(&self, sketch: &mut Sketch, chance: &mut Chance) {
        let spots = sites(self.mesh, |i, j| {
            (j == 0 || j == self.mesh[i].len() - 1) && chance.random() < 0.1
        });
        for p in spots {
            let [x, y] = self.at(p);
            let args = RockArgs {
                wid: 20.0 + chance.random() * 20.0,
                hei: 20.0 + chance.random() * 20.0,
                sha: 2,
                ..Default::default()
            };
            rock(sketch, chance, x, y, self.seed, &args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ink::Paint;

    #[test]
    fn mesh_shape() {
        let mut c = Chance::from_seed(1);
        let mesh = mountain_mesh(&mut c, 400.0, 0.5, 300.0, 500.0);
        assert_eq!(mesh.len(), 10);
        assert!(mesh.iter().all(|row| row.len() == 50));
        let top = mesh[0].iter().map(|p| p[1]).fold(f64::MAX, f64::min);
        assert!(top < -50.0, "peak {top}");
    }

    #[test]
    fn mountain_draws_a_lot() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        mountain(
            &mut s,
            &mut c,
            0.0,
            500.0,
            1.0,
            &MountainArgs {
                hei: Some(300.0),
                ..Default::default()
            },
        );
        assert!(s.len() > 200, "only {}", s.len());
        assert!(s.polygons.iter().any(|p| p.style.fill == Paint::Paper));
    }
}
