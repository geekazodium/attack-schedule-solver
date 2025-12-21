use std::ops::RangeFrom;

use crate::enemy_track::EnemyTrack;
use crate::enemy_track::complement_attack_request::ComplementAttackRequest;

#[derive(Debug)]
pub struct Solver {
    lead_track: EnemyTrack,
    other_tracks: Vec<EnemyTrack>,
}

#[allow(unused)]
impl Solver {
    pub fn new(lead: EnemyTrack) -> Self {
        Self {
            lead_track: lead,
            other_tracks: vec![],
        }
    }
    pub fn add_track(&mut self, track: EnemyTrack) {
        self.other_tracks.push(track);
    }
    pub fn solve(&mut self, hasher: &mut impl SolverRandomState) -> ComplementAttackRequest {
        let mut request: ComplementAttackRequest = self
            .lead_track
            .last_queued_attack_as_request()
            .expect("this should never be emptyyyy");
        loop {
            let mut possible_commits = self
                .other_tracks
                .iter()
                .zip(RangeFrom { start: 0_usize })
                .map(|(track, index)| (index, track.possible_future_commits(&mut request)))
                .filter(|(_, b)| !b.is_empty())
                .collect::<Vec<_>>();

            if possible_commits.is_empty() {
                break;
            }
            let index = hasher.next_in_range(possible_commits.len());
            let (track_index, mut options) = possible_commits.swap_remove(index);
            let index = hasher.next_in_range(options.len());
            self.other_tracks[track_index].commit(&mut request, options.swap_remove(index));
            if !request.next_unclaimed() {
                break;
            }
        }
        request
    }
}

pub trait SolverRandomState {
    fn next_in_range(&mut self, max: usize) -> usize;
}

#[cfg(test)]
mod solver_test {
    use std::time::Duration;
    use std::time::Instant;

    use crate::attack::Attack;
    use crate::default_hasher_random::HashRandom;
    use crate::enemy_track::EnemyTrack;
    use crate::solver::Solver;

    #[test]
    fn solve_select_move_success() {
        let mut random = HashRandom::new(120);
        let mut lead_track = EnemyTrack::new(vec![
            Attack::new(30, vec![15, 25], vec![20]),
            Attack::new(40, vec![38], vec![20, 30]),
            Attack::new(80, vec![38], vec![20, 30, 60]),
        ]);

        assert!(lead_track.commit_by_index(2));

        let mut solver = Solver::new(lead_track);

        for _ in 0..2 {
            solver.add_track(EnemyTrack::new(vec![
                Attack::new(30, vec![15, 25], vec![20]),
                Attack::new(40, vec![10], vec![20, 30]),
                Attack::new(40, vec![20], vec![30]),
                Attack::new(40, vec![30], vec![20]),
            ]));
        }

        let now = Instant::now();
        let request = solver.solve(&mut random);
        let elapsed = Instant::now() - now;
        dbg!(elapsed);
        //cursed performance target check
        assert!(elapsed < Duration::from_millis(8));
        // dbg!(&solver);
        dbg!(request);
        dbg!(solver);
    }
}
