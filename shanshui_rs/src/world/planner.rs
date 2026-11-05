//! `mountplanner`: decides what stands where in one 512-unit chunk —
//! mountains at noise peaks, distant ranges every 1000 units, plateaus in
//! the gaps between mountains, and boats on the water.
//!
//! Unlike the original, each chunk plans from its own random streams, and
//! the "is this spot free" matrix is rebuilt from neighbouring chunks'
//! mountains, so the landscape is identical whichever way you scroll.

use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::Arc;

use crate::chance::{Chance, Perlin};

pub const CHUNK_WIDTH: f64 = 512.0;
const XSTEP: f64 = 5.0;
const SAMP: f64 = 0.03;
/// Half-width of the ground a mountain claims (`mwid`).
const MOUNT_CLAIM: f64 = 200.0;
/// How far (in chunks) a mountain can be thrown from where it was planned.
const MOUNT_REACH: i64 = 2;

/// Kind of landform or object placed by the planner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Mount,
    DistMount,
    FlatMount,
    Boat,
}

/// One planned scene; `y` is also its depth (larger = nearer).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub kind: Kind,
    pub x: f64,
    pub y: f64,
}

/// Seeds and noise of one world.
#[derive(Clone, Debug)]
pub struct Planner {
    pub world_seed: u64,
    pub perlin: Arc<Perlin>,
}

/// Random-stream slots, so planning and drawing never share a stream.
#[derive(Clone, Copy)]
pub enum Slot {
    Mounts = 1,
    Extras = 2,
    Scene = 3,
}

/// Distinct stream per `(chunk, slot, item)`.
pub fn stream_id(chunk: i64, slot: Slot, item: u64) -> u64 {
    (chunk as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ ((slot as u64) << 56)
        ^ item.wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl Planner {
    pub fn chance(&self, chunk: i64, slot: Slot, item: u64) -> Chance {
        Chance::new(
            self.world_seed,
            stream_id(chunk, slot, item),
            self.perlin.clone(),
        )
    }

    /// Mountain-peak strength at `x`.
    fn peak(&self, x: f64) -> f64 {
        (self.perlin.noise(x * SAMP, 0.0, 0.0) - 0.55).max(0.0) * 2.0
    }

    fn is_local_max(&self, x: f64) -> bool {
        let z0 = self.peak(x);
        z0 > 0.3 && (-2..2).all(|d| self.peak(x + f64::from(d)) <= z0)
    }

    /// Everything placed in chunk `k`. Example: `planner.plan(0)`.
    pub fn plan(&self, k: i64) -> Vec<Placement> {
        let mut reg = Registry::default();
        for m in self.mounts(k) {
            reg.add(m, 10.0);
        }
        let mut chance = self.chance(k, Slot::Extras, 0);
        self.dist_mounts(k, &mut chance, &mut reg);
        let claimed = self.claims(k);
        self.flat_mounts(k, &mut chance, &mut reg, &claimed);
        self.boats(k, &mut chance, &mut reg);
        reg.items
    }

    fn xs(k: i64) -> impl Iterator<Item = f64> {
        let x0 = k as f64 * CHUNK_WIDTH;
        (0..(CHUNK_WIDTH / XSTEP) as usize).map(move |i| x0 + i as f64 * XSTEP)
    }

    /// Mountains planned in chunk `k` (they may land in neighbours).
    fn mounts(&self, k: i64) -> Vec<Placement> {
        let mut chance = self.chance(k, Slot::Mounts, 0);
        let mut reg = Registry::default();
        for x in Self::xs(k).filter(|&x| self.is_local_max(x)) {
            let depth = self.perlin.noise(x * 0.01, PI, 0.0) * 480.0;
            let mut j = 0.0;
            while j < depth {
                let mx = x + 2.0 * (chance.random() - 0.5) * 500.0;
                reg.add(
                    Placement {
                        kind: Kind::Mount,
                        x: mx,
                        y: j + 300.0,
                    },
                    10.0,
                );
                j += 30.0;
            }
        }
        reg.items
    }

    /// `planmtx` for chunk `k`: how many mountains claim each 5-unit cell.
    fn claims(&self, k: i64) -> HashMap<i64, u32> {
        let mut claimed = HashMap::new();
        for n in k - MOUNT_REACH..=k + MOUNT_REACH {
            for m in self.mounts(n) {
                let first = ((m.x - MOUNT_CLAIM) / XSTEP).floor() as i64;
                let last = ((m.x + MOUNT_CLAIM) / XSTEP).ceil() as i64;
                (first..last).for_each(|cell| *claimed.entry(cell).or_insert(0) += 1);
            }
        }
        claimed
    }

    fn dist_mounts(&self, k: i64, chance: &mut Chance, reg: &mut Registry) {
        for x in Self::xs(k).filter(|x| x.abs() % 1000.0 < XSTEP - 1.0) {
            reg.add(
                Placement {
                    kind: Kind::DistMount,
                    x,
                    y: 280.0 - chance.random() * 50.0,
                },
                10.0,
            );
        }
    }

    fn flat_mounts(
        &self,
        k: i64,
        chance: &mut Chance,
        reg: &mut Registry,
        claimed: &HashMap<i64, u32>,
    ) {
        for x in Self::xs(k) {
            let free = !claimed.contains_key(&((x / XSTEP).floor() as i64));
            if !free || chance.random() >= 0.01 {
                continue;
            }
            let mut j = 0.0;
            while j < 4.0 * chance.random() {
                let fx = x + 2.0 * (chance.random() - 0.5) * 700.0;
                reg.add(
                    Placement {
                        kind: Kind::FlatMount,
                        x: fx,
                        y: 700.0 - j * 50.0,
                    },
                    10.0,
                );
                j += 1.0;
            }
        }
    }

    fn boats(&self, k: i64, chance: &mut Chance, reg: &mut Registry) {
        for x in Self::xs(k) {
            if chance.random() < 0.2 {
                let y = 300.0 + chance.random() * 390.0;
                reg.add(
                    Placement {
                        kind: Kind::Boat,
                        x,
                        y,
                    },
                    400.0,
                );
            }
        }
    }
}

/// `chadd`: keeps placements apart horizontally.
#[derive(Default)]
struct Registry {
    items: Vec<Placement>,
}

impl Registry {
    fn add(&mut self, p: Placement, min_gap: f64) -> bool {
        if self.items.iter().any(|q| (q.x - p.x).abs() < min_gap) {
            return false;
        }
        self.items.push(p);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chance::world_perlin;

    fn planner(seed: u64) -> Planner {
        Planner {
            world_seed: seed,
            perlin: Arc::new(world_perlin(seed)),
        }
    }

    #[test]
    fn plan_is_deterministic() {
        let p = planner(7);
        assert_eq!(p.plan(3), p.plan(3));
        assert_ne!(p.plan(3), p.plan(4));
    }

    #[test]
    fn world_has_mountains_somewhere() {
        let p = planner(11);
        let mounts = (-10..10)
            .flat_map(|k| p.plan(k))
            .filter(|q| q.kind == Kind::Mount)
            .count();
        assert!(mounts > 3, "only {mounts} mountains in 20 chunks");
    }

    #[test]
    fn dist_mount_every_thousand() {
        let p = planner(1);
        let d: Vec<f64> = (0..20)
            .flat_map(|k| p.plan(k))
            .filter(|q| q.kind == Kind::DistMount)
            .map(|q| q.x)
            .collect();
        // A nearby mountain may veto one (`chadd`), but most survive.
        assert!(d.len() >= 5, "{d:?}");
        assert!(d.iter().all(|x| x.abs() % 1000.0 < XSTEP - 1.0), "{d:?}");
    }

    #[test]
    fn registry_keeps_gap() {
        let mut r = Registry::default();
        let p = Placement {
            kind: Kind::Boat,
            x: 0.0,
            y: 0.0,
        };
        assert!(r.add(p, 400.0));
        assert!(!r.add(Placement { x: 399.0, ..p }, 400.0));
        assert!(r.add(Placement { x: 401.0, ..p }, 400.0));
    }

    #[test]
    fn stream_ids_differ() {
        assert_ne!(stream_id(1, Slot::Scene, 0), stream_id(1, Slot::Scene, 1));
        assert_ne!(stream_id(1, Slot::Mounts, 0), stream_id(1, Slot::Extras, 0));
        assert_ne!(stream_id(-1, Slot::Scene, 0), stream_id(1, Slot::Scene, 0));
    }
}
