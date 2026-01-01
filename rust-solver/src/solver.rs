use crate::enemy_track::EnemyTrack;
use crate::enemy_track::complement_attack_request::ComplementAttackRequest;
use crate::enemy_track::complement_attack_request::request_offset::RequestOffset;
use std::collections::HashMap;
use std::num::NonZeroI64;

#[derive(Debug)]
pub struct Solver {
    lead_track_id: Option<NonZeroI64>,
    tracks: HashMap<NonZeroI64, EnemyTrack>,
    lead_request: Option<ComplementAttackRequest>,
    time_now_frames: u64,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            lead_track_id: None,
            tracks: HashMap::new(),
            lead_request: None,
            time_now_frames: 0,
        }
    }
    //swaps in the track at index with lead track, then returns the new index of the track that was swapped
    pub fn change_lead(&mut self, track_id: NonZeroI64) {
        self.lead_track_id = Some(track_id);
    }
    pub fn get_lead(&self) -> Option<NonZeroI64> {
        self.lead_track_id
    }
    fn clear_lead(&mut self) {
        self.lead_track_id = None;
        self.lead_request = None;
    }
    pub fn add_track(&mut self, index: NonZeroI64, track: EnemyTrack) {
        self.tracks.insert(index, track);
    }
    pub fn remove_track(&mut self, index: NonZeroI64) {
        self.tracks.remove(&index);
        if self.lead_track_id == Some(index) {
            self.clear_lead();
        }
    }
    pub fn get_track_mut(&mut self, index: NonZeroI64) -> &mut EnemyTrack {
        self.tracks.get_mut(&index).unwrap()
    }
    pub fn get_track(&self, index: NonZeroI64) -> &EnemyTrack {
        self.tracks.get(&index).unwrap()
    }
    pub fn all_tracks_actionable(&self, start_time: u64) -> bool {
        !self
            .tracks
            .iter()
            .any(|(_, value)| !value.is_actionable_now(start_time, self.time_now_frames()))
    }
    pub fn get_non_actionable_tracks(&self, start_time: u64) -> Vec<&NonZeroI64> {
        self.tracks
            .iter()
            .filter(|(_, value)| !value.is_actionable_now(start_time, self.time_now_frames()))
            .map(|(index, _)| index)
            .collect::<Vec<&NonZeroI64>>()
    }
    pub fn tick(&mut self) {
        self.time_now_frames += 1;
    }
    pub fn time_now_frames(&self) -> u64 {
        self.time_now_frames
    }
    //returns true if the lead request is cleared or if there was no lead request
    fn try_clear_lead_request(&mut self) -> bool {
        if let Some(req) = &self.lead_request {
            // godot_print!("current request claim ends at: {}", req.claim_end_time());
            if req.claim_end_time() > self.time_now_frames {
                return false;
            }
            self.lead_request = None;
            dbg!("lead request cleared!");
        }
        true
    }
    fn update_current_request(&mut self, random: &mut impl SolverRandomState) {
        if !self.try_clear_lead_request() {
            return;
        }
        if self.is_valid_lead() {
            let mut arr = self.get_non_actionable_tracks(self.time_now_frames());
            if arr.is_empty() {
                return;
            }
            let index = random.next_in_range(arr.len());
            let key = arr.swap_remove(index);
            self.change_lead(*key);
        }
        self.lead_request = self
            .get_lead_track()
            .and_then(EnemyTrack::last_queued_attack_as_request);
    }
    pub fn update_latest_nonpast(&mut self) {
        let curr_tick = self.time_now_frames();
        for value in self.tracks.values_mut() {
            value.update_latest_nonpast(curr_tick);
        }
    }
    fn get_lead_track(&self) -> Option<&EnemyTrack> {
        self.lead_track_id.as_ref().and_then(|v| self.tracks.get(v))
    }
    fn is_valid_lead(&self) -> bool {
        self.get_lead_track()
            .is_none_or(|v| v.is_actionable_now(self.time_now_frames(), self.time_now_frames()))
    }
    fn solve_request(
        &mut self,
        mut request: ComplementAttackRequest,
        random: &mut impl SolverRandomState,
    ) -> ComplementAttackRequest {
        let mut request_state = RequestOffset::new_default();
        while let Some(new_offset) = request.next_unclaimed(request_state) {
            request_state = new_offset;

            let mut possible_commits = self
                .tracks
                .iter()
                .map(|(index, track)| {
                    (
                        index,
                        track.possible_future_commits(&request, self.time_now_frames()),
                    )
                })
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
        }
        request
    }
    pub fn solve(&mut self, random: &mut impl SolverRandomState) {
        self.update_current_request(random);
        if let Some(request) = self.lead_request.take() {
            self.lead_request = Some(self.solve_request(request, random));
        } else {
            println!("no last queued attack, can not create request and solve");
        }
    }
    pub fn reset_non_current(&mut self) {
        let now = self.time_now_frames();
        for track in self.tracks.values_mut() {
            track.reset_non_current(now);
        }
        let mut req = None;
        for (track, commit) in self
            .tracks
            .values()
            .filter_map(|track| track.latest_nonpast_commit().map(|commit| (track, commit)))
            .filter(|(track, commit)| track.commit_valid(commit))
        {
            req = match req {
                None => track.get_commit_as_request(commit),
                Some(mut req) => {
                    req.apply_commit_claim(track, commit);
                    Some(req)
                }
            }
        }
        self.lead_request = req;
    }
}

pub trait SolverRandomState {
    fn next_in_range(&mut self, max: usize) -> usize;
}
