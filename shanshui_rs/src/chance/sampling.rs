//! Sampling helpers from the `#Util` script (`normRand`, `randChoice`,
//! `wtrand`, `randGaussian`).

use super::Chance;

impl Chance {
    /// Uniform in `[lo, hi)`. Example: `chance.norm_rand(-1.0, 1.0)`.
    pub fn norm_rand(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.random()
    }

    /// Picks one element. Example: `chance.choice(&[0, 0, 1, 2])`.
    pub fn choice<T: Copy>(&mut self, options: &[T]) -> T {
        let i = (options.len() as f64 * self.random()) as usize;
        options[i.min(options.len() - 1)]
    }

    /// `randChoice([-1, 1])`, used all over for mirroring.
    pub fn sign(&mut self) -> f64 {
        if self.random() < 0.5 { -1.0 } else { 1.0 }
    }

    /// Rejection-samples `x` in `[0, 1)` with density proportional to
    /// `weight(x)` (which must stay in `[0, 1]`).
    pub fn weighted(&mut self, weight: impl Fn(f64) -> f64) -> f64 {
        loop {
            let x = self.random();
            if self.random() < weight(x) {
                return x;
            }
        }
    }

    /// Bell-shaped sample in `[-1, 1)`.
    pub fn gaussian(&mut self) -> f64 {
        self.weighted(|x| (-24.0 * (x - 0.5).powi(2)).exp()) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_rand_in_range() {
        let mut c = Chance::from_seed(3);
        for _ in 0..1000 {
            let v = c.norm_rand(40.0, 70.0);
            assert!((40.0..70.0).contains(&v), "got {v}");
        }
    }

    #[test]
    fn choice_hits_every_option() {
        let mut c = Chance::from_seed(4);
        let mut seen = [false; 3];
        for _ in 0..200 {
            seen[c.choice(&[0usize, 1, 2])] = true;
        }
        assert_eq!(seen, [true; 3]);
    }

    #[test]
    fn gaussian_centres_on_zero() {
        let mut c = Chance::from_seed(5);
        let mean: f64 = (0..5000).map(|_| c.gaussian()).sum::<f64>() / 5000.0;
        assert!(mean.abs() < 0.05, "mean {mean}");
    }

    #[test]
    fn weighted_respects_zero_weight() {
        let mut c = Chance::from_seed(6);
        for _ in 0..500 {
            assert!(c.weighted(|x| if x < 0.5 { 0.0 } else { 1.0 }) >= 0.5);
        }
    }

    #[test]
    fn sign_is_unit() {
        let mut c = Chance::from_seed(7);
        assert_eq!(c.sign().abs(), 1.0);
    }
}
