use std::hash::DefaultHasher;
use std::hash::Hasher;

use crate::solver::SolverRandomState;

pub struct HashRandom {
    seed: u64,
    hasher: DefaultHasher,
}

impl HashRandom {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            hasher: DefaultHasher::new(),
        }
    }
}

impl SolverRandomState for HashRandom {
    #[allow(clippy::cast_possible_truncation)]
    fn next_in_range(&mut self, max: usize) -> usize {
        self.hasher.write_u64(self.seed ^ self.hasher.finish());
        let v = u128::from(self.hasher.finish()) * max as u128;
        (v >> 64) as usize
    }
}
