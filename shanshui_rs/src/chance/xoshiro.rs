//! xoshiro256** — small, fast, and stable across crate upgrades, so a seed
//! keeps producing the same landscape forever (an external RNG crate may
//! change its stream between major versions).

/// Raw 64-bit generator; [`super::Chance`] is the API the generators use.
#[derive(Clone, Debug)]
pub struct Xoshiro256 {
    state: [u64; 4],
}

impl Xoshiro256 {
    /// Expands one `u64` into the full state with SplitMix64, as the
    /// xoshiro authors recommend. Example: `Xoshiro256::from_u64(42)`.
    pub fn from_u64(seed: u64) -> Self {
        let mut sm = seed;
        let state = [
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
        ];
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.state;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform in `[0, 1)` using the top 53 bits.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// One SplitMix64 step; also used to mix chunk indices into seeds.
pub fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Xoshiro256::from_u64(7);
        let mut b = Xoshiro256::from_u64(7);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn next_f64_stays_in_unit_interval() {
        let mut r = Xoshiro256::from_u64(1);
        for _ in 0..10_000 {
            let v = r.next_f64();
            assert!((0.0..1.0).contains(&v), "got {v}");
        }
    }

    #[test]
    fn splitmix_advances_state() {
        let mut x = 0;
        let a = splitmix64(&mut x);
        let b = splitmix64(&mut x);
        assert_ne!(a, b);
    }
}
