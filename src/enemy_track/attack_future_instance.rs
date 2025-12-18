use super::complement_attack_request::ComplementAttackRequest;

use crate::attack::Attack;

#[derive(Debug)]
pub(super) struct AttackFutureInstance<'a> {
    attack: &'a Attack,
    frames_into_future: u64,
}

impl<'a> AttackFutureInstance<'a> {
    pub(super) fn try_create(attack: &'a Attack, request_frame: u64) -> Option<Self> {
        attack
            .get_start_frame(request_frame)
            .map(|start_frame| Self {
                attack,
                frames_into_future: start_frame,
            })
    }
    pub(super) fn active_frame_times(&self) -> impl Iterator<Item = u64> {
        self.attack.get_active_frames(self.frames_into_future)
    }
    pub(super) fn can_meet_request_followup(&self, request: &ComplementAttackRequest) -> bool {
        let mut request_iter = request.iter_skip_start();
        let active_frames_iter = self.active_frame_times();
        for active in active_frames_iter.skip(1) {
            dbg!(active);
            if active >= request.claim_end_time() {
                return true;
            }
            while let Some(next_request_frame) = request_iter.next() {
                if *next_request_frame > active {
                    return false;
                }
                if *next_request_frame == active {
                    break;
                }
            }
        }
        return true;
    }
    pub(super) fn to_attack(self) -> &'a Attack {
        self.attack
    }
}
