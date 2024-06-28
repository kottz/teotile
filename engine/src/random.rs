use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

pub struct CustomRng {
    rng: ChaCha8Rng,
}

impl CustomRng {
    pub fn seed_from_u64(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    pub fn gen_range(&mut self, low: u32, high: u32) -> u32 {
        assert!(low < high, "low must be less than high");
        low + (self.next_u32() % (high - low))
    }

    pub fn gen_range_f32(&mut self, low: f32, high: f32) -> f32 {
        assert!(low < high, "low must be less than high");
        low + (self.next_u32() as f32 / u32::MAX as f32) * (high - low)
    }

    pub fn gen_range_f64(&mut self, low: f64, high: f64) -> f64 {
        assert!(low < high, "low must be less than high");
        low + (self.next_u64() as f64 / u64::MAX as f64) * (high - low)
    }

    pub fn gen_bool(&mut self, probability: f64) -> bool {
        assert!(
            (0.0..=1.0).contains(&probability),
            "probability must be between 0 and 1"
        );
        (self.next_u64() as f64 / u64::MAX as f64) < probability
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            slice.swap(i, j);
        }
    }
}

impl RngCore for CustomRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

impl SeedableRng for CustomRng {
    type Seed = <ChaCha8Rng as SeedableRng>::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            rng: ChaCha8Rng::from_seed(seed),
        }
    }
}
