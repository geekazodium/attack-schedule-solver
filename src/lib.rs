use std::hash::{DefaultHasher, Hasher, RandomState};
use std::ops::RangeFrom;

use crate::enemy_track::EnemyTrack;
use crate::enemy_track::complement_attack_request::ComplementAttackRequest;

pub mod attack;
pub mod enemy_track;

#[derive(Debug)]
struct Solver {
    lead_track: EnemyTrack,
    other_tracks: Vec<EnemyTrack>,
}

impl Solver {
    fn new(lead: EnemyTrack) -> Self {
        Self {
            lead_track: lead,
            other_tracks: vec![],
        }
    }
    fn add_track(&mut self, track: EnemyTrack) {
        self.other_tracks.push(track);
    }
    fn solve(&mut self, hasher: &mut impl MoveRandom) {
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
                .filter(|(_, b)| b.len() > 0)
                .collect::<Vec<_>>();
            if possible_commits.len() > 0 {
                let index = hasher.next_in_range(possible_commits.len());
                let (track_index, mut options) = possible_commits.swap_remove(index);
                let index = hasher.next_in_range(options.len());
                self.other_tracks[track_index].commit(&mut request, options.swap_remove(index));
            }
            if !request.skip() {
                break;
            }
        }
    }
}

trait MoveRandom {
    fn next_in_range(&mut self, max: usize) -> usize;
}

struct HashRandom {
    seed: u64,
    hasher: DefaultHasher,
}

impl MoveRandom for HashRandom {
    fn next_in_range(&mut self, max: usize) -> usize {
        self.hasher.write_u64(self.seed ^ self.hasher.finish());
        let v = (self.hasher.finish() as u128) * max as u128;
        (v >> 64) as usize
    }
}

#[cfg(test)]
mod tests {
    use std::hash::DefaultHasher;

    use crate::HashRandom;
    use crate::MoveRandom;
    use crate::Solver;
    use crate::attack::Attack;
    use crate::enemy_track::EnemyTrack;

    #[test]
    fn it_works() {
        let mut rand1 = HashRandom {
            hasher: DefaultHasher::new(),
            seed: 0,
        };
        let mut rand2 = HashRandom {
            hasher: DefaultHasher::new(),
            seed: 0,
        };
        for _ in 0..10000 {
            let random_v = rand1.next_in_range(3);
            assert!(random_v < 3);
            assert_eq!(random_v, rand2.next_in_range(3));
        }
    }

    #[test]
    fn solve_select_move_success() {
        let mut random = HashRandom {
            hasher: DefaultHasher::new(),
            seed: 100,
        };

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
