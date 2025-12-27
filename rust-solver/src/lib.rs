use godot::init::ExtensionLibrary;
use godot::init::gdextension;

pub(crate) mod attack;
mod default_hasher_random;
pub(crate) mod enemy_track;
mod solver;
mod solver_interface;

struct AttackSchedulerExtension;

#[gdextension]
unsafe impl ExtensionLibrary for AttackSchedulerExtension {}

#[cfg(test)]
mod tests {
    use crate::attack::Attack;
    use crate::default_hasher_random::HashRandom;
    use crate::enemy_track::EnemyTrack;
    use crate::solver::Solver;
    use crate::solver::SolverRandomState;
    use std::i64;
    use std::num::NonZeroI64;
    use std::time::Duration;
    use std::time::Instant;

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
        let mut random = HashRandom::new(120);
        let mut lead_track = EnemyTrack::new(vec![
            Attack::new(30, vec![15, 25], vec![20]),
            Attack::new(40, vec![38], vec![20, 30]),
            Attack::new(80, vec![38], vec![20, 30, 60]),
        ]);

        assert!(lead_track.commit_by_index(2, 2));

        let mut solver = Solver::new();
        let lead_key = NonZeroI64::new(i64::MAX).unwrap();
        solver.add_track(lead_key, lead_track);
        solver.change_lead(lead_key);

        for count in 0..2 {
            solver.add_track(
                NonZeroI64::new(count + 1).unwrap(),
                EnemyTrack::new(vec![
                    Attack::new(30, vec![15, 25], vec![20]),
                    Attack::new(40, vec![10], vec![20, 30]),
                    Attack::new(40, vec![20], vec![30]),
                    Attack::new(40, vec![30], vec![20]),
                ]),
            );
        }

        let now = Instant::now();
        solver.try_create_new_request();
        let request = solver.solve(&mut random);
        let elapsed = Instant::now() - now;
        dbg!(elapsed);
        //cursed performance target check
        assert!(elapsed < Duration::from_millis(8));
        // dbg!(&solver);
        dbg!(request);
        dbg!(solver);
    }

    #[test]
    fn test_select_move_and_tick() {
        let mut random = HashRandom::new(120);
        let mut lead_track = EnemyTrack::new(vec![
            Attack::new(30, vec![15, 25], vec![20]),
            Attack::new(40, vec![38], vec![20, 30]),
            Attack::new(80, vec![38], vec![20, 30, 60]),
        ]);

        assert!(lead_track.commit_by_index(2, 2));

        let mut solver = Solver::new();
        let lead_key = NonZeroI64::new(i64::MAX).unwrap();
        solver.add_track(lead_key, lead_track);
        solver.change_lead(lead_key);

        for count in 0..2 {
            solver.add_track(
                NonZeroI64::new(count + 1).unwrap(),
                EnemyTrack::new(vec![
                    Attack::new(30, vec![15, 25], vec![20]),
                    Attack::new(40, vec![10], vec![20, 30]),
                    Attack::new(40, vec![20], vec![30]),
                    Attack::new(40, vec![30], vec![20]),
                ]),
            );
        }

        for now in 0..120 {
            solver.get_track_mut(lead_key).commit_by_index(2, now);
            solver.try_create_new_request();
            let request = solver.solve(&mut random);
            solver.tick();
            // dbg!(request.unwrap().claim_end_time());
        }
    }
}
