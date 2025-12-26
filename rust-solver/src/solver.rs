use std::num::NonZeroUsize;
use std::ops::RangeFrom;

use crate::enemy_track::EnemyTrack;
use crate::enemy_track::complement_attack_request::ComplementAttackRequest;

#[derive(Debug)]
pub struct Solver {
    tracks: Vec<EnemyTrack>,
    lead_request: Option<ComplementAttackRequest>,
    time_frames: u64,
}

const LEAD_TRACK_INDEX: usize = 0;

#[allow(unused)]
impl Solver {
    pub fn new(lead: EnemyTrack) -> Self {
        Self { tracks: vec![lead], lead_request: None, time_frames: 0 }
    }
    //swaps in the track at index with lead track, then returns the new index of the track that was swapped
    pub fn change_lead(&mut self, track_index: NonZeroUsize) {
        self.tracks.swap(LEAD_TRACK_INDEX, track_index.into());
    }
    pub fn add_track(&mut self, track: EnemyTrack) {
        self.tracks.push(track);
    }
    //returns true if the lead request is cleared or if there was no lead request
    fn try_clear_lead_request(&mut self) -> bool{
        if let Some(req) = &self.lead_request{
            if(req.claim_end_time() >= self.time_frames){
                return false;
            }
            self.lead_request = None;
        }
        true
    }
    pub fn try_create_new_request(&mut self) -> Option<&ComplementAttackRequest>{
        if !self.try_clear_lead_request(){
            return None;
        }
        self.lead_request = self.get_lead_track().last_queued_attack_as_request();
        self.lead_request.as_ref()
    }
    pub fn solve(&mut self, random: &mut impl SolverRandomState) -> Option<&ComplementAttackRequest> {
        if let Some(mut request) = self.lead_request.take() {
            self.lead_request = Some(self.solve_request(request, random));
            self.lead_request.as_ref()
        } else {
            println!("no last queued attack, can not create request and solve");
            None
        }
    }
    fn get_lead_track(&mut self) -> &mut EnemyTrack {
        &mut self.tracks[LEAD_TRACK_INDEX]
    }
    fn solve_request(
        &mut self,
        mut request: ComplementAttackRequest,
        random: &mut impl SolverRandomState,
    ) -> ComplementAttackRequest {
        loop {
            let mut possible_commits = self
                .tracks
                .iter()
                .zip(RangeFrom { start: 0_usize })
                .map(|(track, index)| (index, track.possible_future_commits(&mut request)))
                .filter(|(_, b)| !b.is_empty())
                .collect::<Vec<_>>();

            if possible_commits.is_empty() {
                break;
            }
            let index = random.next_in_range(possible_commits.len());
            let (track_index, mut options) = possible_commits.swap_remove(index);
            let index = random.next_in_range(options.len());
            self.tracks[track_index].commit(&mut request, options.swap_remove(index));
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
