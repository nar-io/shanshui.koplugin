//! Turns placements into drawn scenes (the body of `chunkloader`).

use std::f64::consts::PI;

use super::planner::{Kind, Placement, Planner, Slot};
use crate::chance::Chance;
use crate::ink::Sketch;
use crate::scenery::arch::boat01;
use crate::scenery::mount::{FlatMountArgs, MountainArgs, dist_mount, flat_mount, mountain};
use crate::scenery::water::water;

/// A drawn landform with its painting depth.
#[derive(Clone, Debug)]
pub struct Scene {
    /// Painter's-order key: scenes with larger `depth` are painted later.
    pub depth: f64,
    pub sketch: Sketch,
    pub bbox: [f64; 4],
}

impl Scene {
    fn new(depth: f64, sketch: Sketch) -> Option<Self> {
        let bbox = sketch.bbox()?;
        Some(Self {
            depth,
            sketch,
            bbox,
        })
    }
}

/// Draws everything planned for chunk `k`.
/// Example: `chunk_scenes(&planner, 0)`.
pub fn chunk_scenes(planner: &Planner, k: i64) -> Vec<Scene> {
    planner
        .plan(k)
        .iter()
        .enumerate()
        .flat_map(|(i, p)| draw(p, &mut planner.chance(k, Slot::Scene, i as u64)))
        .collect()
}

fn draw(p: &Placement, chance: &mut Chance) -> Vec<Scene> {
    let mut sketch = Sketch::new();
    match p.kind {
        Kind::Mount => {
            let seed = chance.random() * 20.0;
            mountain(
                &mut sketch,
                chance,
                p.x,
                p.y,
                seed,
                &MountainArgs::default(),
            );
            let mut ripples = Sketch::new();
            water(&mut ripples, chance, p.x, p.y);
            // Water lies under everything, as `y - 10000` did.
            return [Scene::new(p.y, sketch), Scene::new(p.y - 10_000.0, ripples)]
                .into_iter()
                .flatten()
                .collect();
        }
        Kind::FlatMount => {
            let seed = 2.0 * chance.random() * PI;
            let args = FlatMountArgs {
                wid: 600.0 + chance.random() * 400.0,
                hei: 100.0,
                cho: 0.5 + chance.random() * 0.2,
                tex: 80,
            };
            flat_mount(&mut sketch, chance, p.x, p.y, seed, &args);
        }
        Kind::DistMount => {
            let (seed, len) = (
                chance.random() * 100.0,
                chance.choice(&[500.0, 1000.0, 1500.0]),
            );
            dist_mount(&mut sketch, chance, p.x, p.y, seed, 150.0, len);
        }
        Kind::Boat => {
            let fli = chance.random() < 0.5;
            boat01(&mut sketch, chance, p.x, p.y, p.y / 800.0, fli);
        }
    }
    Scene::new(p.y, sketch).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chance::world_perlin;
    use std::sync::Arc;

    #[test]
    fn mount_brings_water_below() {
        let mut c = Chance::from_seed(1);
        let scenes = draw(
            &Placement {
                kind: Kind::Mount,
                x: 0.0,
                y: 500.0,
            },
            &mut c,
        );
        assert_eq!(scenes.len(), 2);
        assert!(scenes[1].depth < 0.0);
    }

    #[test]
    fn chunk_scenes_are_deterministic() {
        let p = Planner {
            world_seed: 5,
            perlin: Arc::new(world_perlin(5)),
        };
        let (a, b) = (chunk_scenes(&p, 0), chunk_scenes(&p, 0));
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(&b).all(|(x, y)| x.sketch == y.sketch));
    }
}
