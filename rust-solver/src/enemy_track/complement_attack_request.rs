use self::request_restore_point::RequestRestorePoint;
use crate::enemy_track::EnemyTrack;
use crate::enemy_track::future_move_commit::FutureMoveCommit;
mod request_restore_point;

#[derive(Debug)]
pub struct ComplementAttackRequest {
    request_frames: Vec<u64>,
    taken_requests: Vec<bool>,
    request_offset: usize,
    request_source_claim_end: u64,
}

impl ComplementAttackRequest {
    pub fn new(vec: Vec<u64>, request_source_claim_end: u64, start_frame: u64) -> Option<Self> {
        if vec.is_empty() {
            None
        } else {
            Some(Self {
                request_offset: 0,
                taken_requests: vec.iter().map(|_| false).collect(),
                request_frames: vec.iter().map(|x| x + start_frame).collect(),
                request_source_claim_end: request_source_claim_end + start_frame,
            })
        }
    }
    pub(crate) fn first_req_frame(&self) -> Option<u64> {
        if self.taken_requests[self.request_offset] {
            return None;
        }
        self.request_frames.get(self.request_offset).copied()
    }
    pub(crate) fn iter_skip_start(&'_ self) -> impl Iterator<Item = u64> {
        self.request_frames
            .iter()
            .zip(&self.taken_requests)
            .skip(self.request_offset + 1)
            .filter_map(|(req, taken)| if *taken { None } else { Some(req) })
            .copied()
    }
    pub(crate) fn claim_end_time(&self) -> u64 {
        self.request_source_claim_end
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
    pub(super) fn apply_commit_claim(
        &mut self,
        track: &EnemyTrack,
        commit: &FutureMoveCommit,
        undo: bool,
    ) {
        let mut index = if undo { 0 } else { self.request_offset };
        for active in commit.get_active_frames(track) {
            if active >= self.claim_end_time() {
                break;
            }
            while index < self.request_frames.len() {
                if self.request_frames[index] == active {
                    self.taken_requests[index] = !undo;
                    break;
                }
                index += 1;
            }
        }
        let mut index = self.request_offset;
        while self
            .request_frames
            .get(index)
            .is_some_and(|v| commit.get_start_frame().gt(v))
        {
            index += 1;
        }
        let get_end_frame = commit.get_end_frame(track);
        for other_request_frame in commit.get_request_frames(track) {
            if other_request_frame >= self.claim_end_time() {
                self.request_frames.push(other_request_frame);
                self.taken_requests.push(false);
                continue;
            }

            while index < self.request_frames.len() {
                if self.request_frames[index] >= get_end_frame {
                    break;
                }
                if self.request_frames[index] == other_request_frame {
                    break;
                }
                self.taken_requests[index] = true;
                index += 1;
            }
        }
        while index < self.request_frames.len() {
            if self.request_frames[index] >= get_end_frame {
                break;
            }
            self.taken_requests[index] = true;
            index += 1;
        }
        self.request_source_claim_end = u64::max(get_end_frame, self.request_source_claim_end);
    }
}

#[cfg(test)]
mod complement_attack_request_tests {
    use super::*;

    #[test]
    fn test_filter() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40], 100, 0).unwrap();
        req.taken_requests[2] = true;

        assert_eq!(req.iter_skip_start().collect::<Vec<u64>>(), vec![32]);
    }

    #[test]
    fn test_filter_first() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40, 90], 100, 0).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        assert_eq!(req.iter_skip_start().collect::<Vec<u64>>(), vec![32, 90]);

        req.taken_requests[2] = false;

        assert_eq!(
            req.iter_skip_start().collect::<Vec<u64>>(),
            vec![32, 40, 90]
        );
    }

    #[test]
    fn test_skip() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40, 90], 100, 0).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        req.skip();
        assert_eq!(req.iter_skip_start().collect::<Vec<u64>>(), vec![90]);

        req.taken_requests[2] = false;

        assert_eq!(req.iter_skip_start().collect::<Vec<u64>>(), vec![40, 90]);
    }

    #[test]
    fn test_restore() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40, 90], 100, 0).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        let restore = req.get_restore_point();
        assert!(req.skip());
        assert_eq!(req.iter_skip_start().collect::<Vec<u64>>(), vec![90]);

        req.taken_requests[2] = false;

        assert_eq!(req.iter_skip_start().collect::<Vec<u64>>(), vec![40, 90]);
        req.restore(&restore);

        assert_eq!(
            req.iter_skip_start().collect::<Vec<u64>>(),
            vec![32, 40, 90]
        );
    }

    #[test]
    fn test_offset() {
        let mut req = ComplementAttackRequest::new(vec![20, 32, 40, 90], 100, 20).unwrap();
        req.taken_requests[2] = true;

        assert_eq!(req.iter_skip_start().collect::<Vec<u64>>(), vec![52, 110]);

        assert_eq!(req.first_req_frame(), Some(40));

        req.skip();

        assert_eq!(req.first_req_frame(), Some(52));
    }
}
