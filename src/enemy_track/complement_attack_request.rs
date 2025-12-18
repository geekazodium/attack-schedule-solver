use crate::attack::Attack;
use crate::enemy_track::EnemyTrack;
use crate::enemy_track::FutureMoveCommit;

#[allow(unused)]
#[derive(Debug)]
pub struct ComplementAttackRequest {
    request_frames: Vec<u64>,
    taken_requests: Vec<bool>,
    request_offset: usize,
    request_source_claim_end: u64,
}

impl From<&Attack> for Option<ComplementAttackRequest> {
    fn from(attack: &Attack) -> Self {
        ComplementAttackRequest::new(
            attack.active_request_frames().clone(),
            0,
            attack.get_full_duration(),
        )
    }
}

impl From<&Attack> for ComplementAttackRequest {
    fn from(value: &Attack) -> Self {
        let maybe: Option<ComplementAttackRequest> = value.into();
        maybe.expect("attack instance has no valid requests.")
    }
}

impl From<Attack> for ComplementAttackRequest {
    fn from(value: Attack) -> Self {
        let maybe: Option<ComplementAttackRequest> = (&value).into();
        maybe.expect("attack instance has no valid requests.")
    }
}

impl ComplementAttackRequest {
    fn new(vec: Vec<u64>, offset: usize, request_source_claim_end: u64) -> Option<Self> {
        if vec.len() <= offset {
            return None;
        }
        Some(Self {
            request_offset: offset,
            taken_requests: vec.iter().map(|_| false).collect(),
            request_frames: vec,
            request_source_claim_end,
        })
    }
    pub(crate) fn start_frame(&self) -> u64 {
        self.request_frames
            .get(self.request_offset)
            .copied()
            .expect("impossible instance of request was created")
    }
    pub(crate) fn iter_skip_start(&'_ self) -> impl Iterator<Item = &u64> {
        self.request_frames
            .iter()
            .zip(&self.taken_requests)
            .skip(self.request_offset + 1)
            .filter_map(|pair| if *pair.1 { None } else { Some(pair.0) })
    }
    pub(crate) fn claim_end_time(&self) -> u64 {
        self.request_source_claim_end
    }
    //attempts to go to the next unclaimed item,
    //returns false if there isn't one.
    pub(super) fn next_unclaimed(&mut self) -> bool {
        while self.request_offset < self.request_frames.len() {
            if !self.taken_requests[self.request_offset] {
                return true;
            }
            self.request_offset += 1;
        }
        self.request_offset -= 1;
        false
    }
    //inverse of the go to next unclaimed, should undo any result of previous if
    //run after changes are uncommitted.
    pub(super) fn prev_unclaimed(&mut self) {
        while self.request_offset > 0 {
            self.request_offset -= 1;
            if self.taken_requests[self.request_offset] {
                self.request_offset += 1;
                break;
            }
        }
    }
    pub(super) fn apply_commit_claim(
        &mut self,
        track: &EnemyTrack,
        commit: &FutureMoveCommit,
        undo: bool,
    ) {
        let mut index = if undo { 0 } else { self.request_offset };
        for active in commit.get_active_frames(track) {
            dbg!(&self);
            if active >= self.claim_end_time() {
                return;
            }
            while index < self.request_frames.len() {
                if self.request_frames[index] == active {
                    self.taken_requests[index] = !undo;
                    break;
                }
                index += 1;
            }
        }
    }
}

#[cfg(test)]
mod complement_attack_request_tests {
    use super::*;

    #[test]
    fn test_filter() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40], 0, 100).unwrap();
        req.taken_requests[2] = true;

        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![32]
        );
    }

    #[test]
    fn test_filter_first() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40, 90], 0, 100).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![32, 90]
        );

        req.taken_requests[2] = false;

        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![32, 40, 90]
        );
    }
}
