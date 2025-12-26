use self::request_restore_point::RequestRestorePoint;
use crate::attack::Attack;
use crate::enemy_track::EnemyTrack;
use crate::enemy_track::future_move_commit::FutureMoveCommit;
mod request_restore_point;

#[derive(Debug)]
pub struct ComplementAttackRequest {
    request_frames: Vec<u64>,
    taken_requests: Vec<bool>,
    request_offset: usize,
    request_source_claim_end: u64,
    request_start_frame_offset: u64,
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
            request_start_frame_offset: 0,
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
        self.request_source_claim_end + self.request_start_frame_offset
    }
    #[must_use]
    pub fn get_restore_point(&self) -> RequestRestorePoint {
        RequestRestorePoint::new(self.request_offset)
    }
    pub fn restore(&mut self, position: &RequestRestorePoint) {
        self.request_offset = position.get();
    }
    pub fn skip(&mut self) -> bool {
        let start = self.request_offset;
        if self.request_offset + 1 < self.taken_requests.len() {
            self.request_offset += 1;
            if self.next_unclaimed() {
                return true;
            }
            self.request_offset = start;
        }
        false
    }
    //attempts to go to the next unclaimed item,
    //returns false if there isn't one.
    pub fn next_unclaimed(&mut self) -> bool {
        while self.request_offset < self.request_frames.len() {
            if !self.taken_requests[self.request_offset] {
                return true;
            }
            self.request_offset += 1;
        }
        self.request_offset -= 1;
        false
    }
    // do I delete this?
    // //inverse of the go to next unclaimed, should undo any result of previous if
    // //run after changes are uncommitted.
    // pub(super) fn prev_unclaimed(&mut self) {
    //     while self.request_offset > 0 {
    //         self.request_offset -= 1;
    //         if self.taken_requests[self.request_offset] {
    //             self.request_offset += 1;
    //             break;
    //         }
    //     }
    // }
    pub(super) fn apply_commit_claim(
        &mut self,
        track: &EnemyTrack,
        commit: &FutureMoveCommit,
        undo: bool,
    ) {
        let mut index = if undo { 0 } else { self.request_offset };
        for active in commit.get_active_frames(track) {
            // dbg!(&self);
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

    #[test]
    fn test_skip() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40, 90], 0, 100).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        req.skip();
        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![90]
        );

        req.taken_requests[2] = false;

        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![40, 90]
        );
    }

    #[test]
    fn test_restore() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40, 90], 0, 100).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        let restore = req.get_restore_point();
        assert!(req.skip());
        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![90]
        );

        req.taken_requests[2] = false;

        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![40, 90]
        );
        req.restore(&restore);

        assert_eq!(
            req.iter_skip_start().map(|x| *x).collect::<Vec<u64>>(),
            vec![32, 40, 90]
        );
    }
}
