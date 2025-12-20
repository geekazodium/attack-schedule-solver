pub mod attack;
mod default_hasher_random;
pub mod enemy_track;
mod solver;

#[cfg(test)]
mod tests {
    use crate::attack::Attack;
    use crate::default_hasher_random::HashRandom;
    use crate::enemy_track::EnemyTrack;
    use crate::solver::Solver;
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

    #[test]
    fn solve_select_move_success() {
        let mut random = HashRandom::new(0);
        let mut lead_track = EnemyTrack::new(vec![
            Attack::new(30, vec![15, 25], vec![20]),
            Attack::new(40, vec![38], vec![20, 30]),
        ]);

        assert!(lead_track.commit_by_index(0));

        let mut solver = Solver::new(lead_track);

        solver.add_track(EnemyTrack::new(vec![
            Attack::new(30, vec![15, 25], vec![20]),
            Attack::new(40, vec![10], vec![20, 30]),
        ]));

        solver.solve(&mut random);
        dbg!(&solver);
    }
}
