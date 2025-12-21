pub mod attack;
mod default_hasher_random;
pub mod enemy_track;
mod solver;

#[cfg(test)]
mod tests {
    use crate::default_hasher_random::HashRandom;
    use crate::solver::SolverRandomState;

    #[test]
    fn test_hasher_works() {
        let mut rand1 = HashRandom::new(0);
        let mut rand2 = HashRandom::new(0);
        for _ in 0..10000 {
            let random_v = rand1.next_in_range(3);
            assert!(random_v < 3);
            assert_eq!(random_v, rand2.next_in_range(3));
        }
    }
}
