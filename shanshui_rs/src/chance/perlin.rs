//! Perlin noise ported from p5.js (the `#PerlinNoise` script of index.html,
//! itself from processing/p5.js `src/math/noise.js`). The look of every
//! ridge and brush wobble depends on this exact flavour of noise.

use super::xoshiro::Xoshiro256;

const YWRAPB: usize = 4;
const YWRAP: usize = 1 << YWRAPB;
const ZWRAPB: usize = 8;
const ZWRAP: usize = 1 << ZWRAPB;
const SIZE: usize = 4095;
const OCTAVES: usize = 4;
const AMP_FALLOFF: f64 = 0.5;

/// Immutable lattice of random values; shared by every chunk of one world.
#[derive(Debug)]
pub struct Perlin {
    lattice: Vec<f64>,
}

/// Integer and fractional parts of one coordinate, advanced per octave.
#[derive(Clone, Copy)]
struct Axis {
    i: usize,
    f: f64,
}

impl Axis {
    fn new(v: f64) -> Self {
        let v = v.abs();
        Self {
            i: v.floor() as usize,
            f: v - v.floor(),
        }
    }

    fn double(self) -> Self {
        let (mut i, mut f) = (self.i << 1, self.f * 2.0);
        if f >= 1.0 {
            i += 1;
            f -= 1.0;
        }
        Self { i, f }
    }
}

fn scaled_cosine(i: f64) -> f64 {
    0.5 * (1.0 - (i * std::f64::consts::PI).cos())
}

impl Perlin {
    /// Fills the lattice from `rng`. Example: `Perlin::new(&mut Xoshiro256::from_u64(1))`.
    pub fn new(rng: &mut Xoshiro256) -> Self {
        let lattice = (0..=SIZE).map(|_| rng.next_f64()).collect();
        Self { lattice }
    }

    fn at(&self, of: usize) -> f64 {
        self.lattice[of & SIZE]
    }

    /// 3-D noise in roughly `[0, 1)`; negative inputs are mirrored like p5.
    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        let (mut ax, mut ay, mut az) = (Axis::new(x), Axis::new(y), Axis::new(z));
        let mut r = 0.0;
        let mut ampl = 0.5;
        for _ in 0..OCTAVES {
            r += self.octave(ax, ay, az) * ampl;
            ampl *= AMP_FALLOFF;
            (ax, ay, az) = (ax.double(), ay.double(), az.double());
        }
        r
    }

    fn octave(&self, x: Axis, y: Axis, z: Axis) -> f64 {
        let of =
            x.i.wrapping_add(y.i.wrapping_shl(YWRAPB as u32))
                .wrapping_add(z.i.wrapping_shl(ZWRAPB as u32));
        let (rxf, ryf) = (scaled_cosine(x.f), scaled_cosine(y.f));
        let near = self.plane(of, rxf, ryf);
        let far = self.plane(of.wrapping_add(ZWRAP), rxf, ryf);
        near + scaled_cosine(z.f) * (far - near)
    }

    fn plane(&self, of: usize, rxf: f64, ryf: f64) -> f64 {
        let mut n1 = self.at(of);
        n1 += rxf * (self.at(of.wrapping_add(1)) - n1);
        let mut n2 = self.at(of.wrapping_add(YWRAP));
        n2 += rxf * (self.at(of.wrapping_add(YWRAP + 1)) - n2);
        n1 + ryf * (n2 - n1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perlin() -> Perlin {
        Perlin::new(&mut Xoshiro256::from_u64(99))
    }

    #[test]
    fn noise_is_bounded() {
        let p = perlin();
        for i in 0..2000 {
            let v = p.noise(i as f64 * 0.37, i as f64 * 0.11, 3.0);
            assert!((0.0..1.0).contains(&v), "noise out of range: {v}");
        }
    }

    #[test]
    fn noise_is_continuous() {
        let p = perlin();
        let a = p.noise(10.0, 2.0, 0.0);
        let b = p.noise(10.001, 2.0, 0.0);
        assert!((a - b).abs() < 0.01, "jump {a} -> {b}");
    }

    #[test]
    fn negative_input_mirrors() {
        let p = perlin();
        assert_eq!(p.noise(-3.5, 0.0, 0.0), p.noise(3.5, 0.0, 0.0));
    }

    #[test]
    fn axis_double_carries() {
        let a = Axis::new(1.75).double();
        assert_eq!(a.i, 3);
        assert!((a.f - 0.5).abs() < 1e-12);
    }
}
