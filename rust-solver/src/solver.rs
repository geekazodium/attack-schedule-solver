use crate::enemy_track::EnemyTrack;
use crate::enemy_track::complement_attack_request::ComplementAttackRequest;
use std::collections::HashMap;
use std::num::NonZeroI64;

#[derive(Debug)]
pub struct Solver {
    lead_track_id: Option<NonZeroI64>,
    tracks: HashMap<NonZeroI64, EnemyTrack>,
    lead_request: Option<ComplementAttackRequest>,
    time_frames: u64,
}

// #[allow(unused)]
impl Solver {
    pub fn new() -> Self {
        Self {
            lead_track_id: None,
            tracks: HashMap::new(),
            lead_request: None,
            time_frames: 0,
        }
    }
    //swaps in the track at index with lead track, then returns the new index of the track that was swapped
    pub fn change_lead(&mut self, track_id: NonZeroI64) {
        self.lead_track_id = Some(track_id);
    }
    pub fn add_track(&mut self, index: NonZeroI64, track: EnemyTrack) {
        self.tracks.insert(index, track);
    }
    pub fn remove_track(&mut self, index: NonZeroI64) {
        self.tracks.remove(&index);
    }
    pub fn get_track_mut(&mut self, index: NonZeroI64) -> &mut EnemyTrack {
        self.tracks.get_mut(&index).unwrap()
    }
    //returns true if the lead request is cleared or if there was no lead request
    fn try_clear_lead_request(&mut self) -> bool {
        if let Some(req) = &self.lead_request {
            // godot_print!("current request claim ends at: {}", req.claim_end_time());
            if req.claim_end_time() >= self.time_frames {
                return false;
            }
            self.lead_request = None;
            dbg!("lead request cleared!");
        }
        true
    }
    pub fn tick(&mut self) {
        self.time_frames += 1;
    }
    pub fn current_tick(&self) -> u64 {
        self.time_frames
    }
    pub fn try_create_new_request(&mut self) -> Option<&ComplementAttackRequest> {
        if !self.try_clear_lead_request() {
            return None;
        }
        self.lead_request = self.get_lead_track()?.last_queued_attack_as_request();
        self.lead_request.as_ref()
    }
    pub fn solve(
        &mut self,
        random: &mut impl SolverRandomState,
    ) -> Option<&ComplementAttackRequest> {
        if let Some(request) = self.lead_request.take() {
            self.lead_request = Some(self.solve_request(request, random));
            self.lead_request.as_ref()
        } else {
            println!("no last queued attack, can not create request and solve");
            None
        }
    }
    fn get_lead_track(&mut self) -> Option<&mut EnemyTrack> {
        self.lead_track_id
            .as_ref()
            .and_then(|v| self.tracks.get_mut(v))
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
                .map(|(index, track)| (index, track.possible_future_commits(&mut request)))
                .filter(|(_, b)| !b.is_empty())
                .collect::<Vec<_>>();

            if possible_commits.is_empty() {
                break;
            }
            let index = random.next_in_range(possible_commits.len());
            let (track_index, mut options) = possible_commits.swap_remove(index);
            let index = random.next_in_range(options.len());
            if let Some(track) = self.tracks.get_mut(&track_index.clone()) {
                let commit = options.swap_remove(index);
                track.commit(&mut request, commit);
            }
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
