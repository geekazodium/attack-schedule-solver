use std::ops::RangeFrom;

use crate::enemy_track::EnemyTrack;
use crate::enemy_track::complement_attack_request::ComplementAttackRequest;

#[derive(Debug)]
pub(crate) struct Solver {
    pub(crate) lead_track: EnemyTrack,
    pub(crate) other_tracks: Vec<EnemyTrack>,
}

#[allow(unused)]
impl Solver {
    pub(crate) fn new(lead: EnemyTrack) -> Self {
        Self {
            lead_track: lead,
            other_tracks: vec![],
        }
    }
    pub(crate) fn add_track(&mut self, track: EnemyTrack) {
        self.other_tracks.push(track);
    }
    pub(crate) fn solve(&mut self, hasher: &mut impl SolverRandomState) {
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
            if !possible_commits.is_empty() {
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

pub trait SolverRandomState {
    fn next_in_range(&mut self, max: usize) -> usize;
}
