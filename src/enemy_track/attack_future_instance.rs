use super::complement_attack_request::ComplementAttackRequest;

use crate::enemy_track::enemy_track_attack_wrapper::EnemyTrackAttack;

#[derive(Debug)]
pub(super) struct AttackFutureInstance<'a> {
    attack: &'a EnemyTrackAttack,
    frames_into_future: u64,
}

impl<'a> AttackFutureInstance<'a> {
    pub(super) fn try_create(
        attack: &'a EnemyTrackAttack,
        request_frame: u64,
        first_actionable: u64,
    ) -> Option<Self> {
        attack
            .get_attack()
            .get_start_frame(request_frame, first_actionable)
            .map(|start_frame| Self {
                attack,
                frames_into_future: start_frame,
            })
    }
    pub(super) fn active_frame_times(&self) -> impl Iterator<Item = u64> {
        self.attack
            .get_attack()
            .get_active_frames(self.frames_into_future)
    }
    pub(super) fn can_meet_request_followup(&self, request: &ComplementAttackRequest) -> bool {
        let mut request_iter = request.iter_skip_start();
        let active_frames_iter = self.active_frame_times();
        for active in active_frames_iter.skip(1) {
            dbg!(active);
            //if outside of current attack's claim, definitely done.
            if active >= request.claim_end_time() {
                return true;
            }
            let mut matched = false;
            for next_request_frame in request_iter.by_ref() {
                if *next_request_frame > active {
                    break;
                }
                if *next_request_frame == active {
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
    pub(super) fn to_attack(self) -> &'a EnemyTrackAttack {
        self.attack
    }
}
