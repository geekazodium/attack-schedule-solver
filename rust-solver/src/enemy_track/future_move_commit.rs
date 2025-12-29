use super::EnemyTrack;
use crate::enemy_track::complement_attack_request::ComplementAttackRequest;

#[derive(Debug)]
pub struct FutureMoveCommit {
    start_frame: u64,
    move_index: usize,
}

impl FutureMoveCommit {
    pub unsafe fn new_unchecked(attack_index: usize, start_frame: u64) -> Self {
        Self {
            start_frame,
            move_index: attack_index,
        }
    }
    pub fn try_create(
        attack_index: usize,
        start_frame: u64,
        first_actionable: u64,
    ) -> Option<Self> {
        if start_frame < first_actionable {
            return None;
        }
        Some(unsafe { Self::new_unchecked(attack_index, start_frame) })
    }
    // pub fn try_create(
    //     track: &EnemyTrack,
    //     attack_index: usize,
    //     request_frame: u64,
    //     first_actionable: u64
    // ) -> Option<Self> {
    //     let attack = track.get_attack(attack_index);
    //     let attack_index = attack.get_index();
    //     attack
    //         .get_attack()
    //         .get_start_frame(request_frame, first_actionable)
    //         .map(|start_frame| unsafe { Self::new_unchecked(attack_index, start_frame) })
    // }
    pub fn get_start_frame(&self) -> u64 {
        // godot_print!("frames after now: {}", self.frames_after_now);
        self.start_frame
    }
    pub fn get_active_frames<'a>(
        &self,
        parent_track: &'a EnemyTrack,
    ) -> impl 'a + Iterator<Item = u64> {
        parent_track
            .get_attack(self.get_index())
            .get_attack()
            .get_active_frames(self.start_frame)
    }
    fn get_full_duration(&self, parent_track: &EnemyTrack) -> u64 {
        parent_track
            .get_attack(self.get_index())
            .get_attack()
            .get_full_duration()
    }
    pub fn get_end_frame(&self, parent_track: &EnemyTrack) -> u64 {
        self.get_full_duration(parent_track) + self.get_start_frame()
    }
    pub fn get_index(&self) -> usize {
        self.move_index
    }
    pub(super) fn active_frame_times(
        &self,
        parent_track: &EnemyTrack,
    ) -> impl Iterator<Item = u64> {
        parent_track
            .get_attack(self.get_index())
            .get_attack()
            .get_active_frames(self.start_frame)
    }
    pub(super) fn can_meet_request_followup(
        &self,
        parent_track: &EnemyTrack,
        request: &ComplementAttackRequest,
    ) -> bool {
        let mut request_iter = request.iter_skip_start();
        let active_frames_iter = self.active_frame_times(parent_track);
        for active in active_frames_iter.skip(1) {
            //if outside of current attack's claim, definitely done.
            if active >= request.claim_end_time() {
                return true;
            }
            let mut matched = false;
            for next_request_frame in request_iter.by_ref() {
                if next_request_frame > active {
                    break;
                }
                if next_request_frame == active {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return false;
            }
        }
        true
    }
}
