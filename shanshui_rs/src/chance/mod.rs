//! Seeded randomness for the generators: a per-chunk random stream plus the
//! world-wide Perlin lattice. Replaces the global `Math.random` override and
//! the `Noise` singleton of index.html; it is passed explicitly everywhere.

mod loop_noise;
mod perlin;
mod sampling;
mod xoshiro;

use std::sync::Arc;

pub use loop_noise::loop_noise;
pub use perlin::Perlin;
use xoshiro::{Xoshiro256, splitmix64};

/// Random stream + noise field handed to every scenery generator.
#[derive(Clone, Debug)]
pub struct Chance {
    rng: Xoshiro256,
    perlin: Arc<Perlin>,
}

impl Chance {
    /// Stream `stream` of the world `world_seed`; chunks use their index as
    /// the stream so they come out the same whatever the scroll order.
    /// Example: `Chance::new(seed_from_text("naught"), 3, perlin)`.
    pub fn new(world_seed: u64, stream: u64, perlin: Arc<Perlin>) -> Self {
        let mut mix = world_seed ^ stream.wrapping_mul(0xA24B_AED4_963E_E407);
        let rng = Xoshiro256::from_u64(splitmix64(&mut mix));
        Self { rng, perlin }
    }

    /// Convenience for tests and one-off sketches: own lattice, stream 0.
    pub fn from_seed(world_seed: u64) -> Self {
        Self::new(world_seed, 0, Arc::new(world_perlin(world_seed)))
    }

    /// Uniform `[0, 1)`, the `Math.random()` of the original.
    pub fn random(&mut self) -> f64 {
        self.rng.next_f64()
    }

    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        self.perlin.noise(x, y, z)
    }

    pub fn noise1(&self, x: f64) -> f64 {
        self.perlin.noise(x, 0.0, 0.0)
    }

    pub fn noise2(&self, x: f64, y: f64) -> f64 {
        self.perlin.noise(x, y, 0.0)
    }
}

/// The lattice shared by every chunk of the world `world_seed`.
pub fn world_perlin(world_seed: u64) -> Perlin {
    Perlin::new(&mut Xoshiro256::from_u64(world_seed ^ 0x5EED_0FFA_7E00))
}

/// FNV-1a of the user-facing seed text (`--seed naught`).
pub fn seed_from_text(text: &str) -> u64 {
    text.bytes().fold(0xCBF2_9CE4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_differ_but_repeat() {
        let p = Arc::new(world_perlin(1));
        let mut a = Chance::new(1, 0, p.clone());
        let mut b = Chance::new(1, 1, p.clone());
        let mut a2 = Chance::new(1, 0, p);
        let (x, y, x2) = (a.random(), b.random(), a2.random());
        assert_ne!(x, y);
        assert_eq!(x, x2);
    }

    #[test]
    fn seed_text_is_stable() {
        assert_eq!(seed_from_text("naught"), seed_from_text("naught"));
        assert_ne!(seed_from_text("naught"), seed_from_text("Naught"));
    }

    #[test]
    fn noise_helpers_match_full_call() {
        let c = Chance::from_seed(5);
        assert_eq!(c.noise1(1.5), c.noise(1.5, 0.0, 0.0));
        assert_eq!(c.noise2(1.5, 2.0), c.noise(1.5, 2.0, 0.0));
    }
}
