use self::request_offset::RequestOffset;
use crate::enemy_track::EnemyTrack;
use crate::enemy_track::future_move_commit::FutureMoveCommit;

pub mod request_offset;

#[derive(Debug)]
pub struct ComplementAttackRequest {
    request_frames: Vec<u64>,
    taken_requests: Vec<bool>,
    request_source_claim_end: u64,
}

impl ComplementAttackRequest {
    pub fn new(vec: &[u64], request_source_claim_end: u64, start_frame: u64) -> Option<Self> {
        if vec.is_empty() {
            None
        } else {
            Some(Self {
                taken_requests: vec.iter().map(|_| false).collect(),
                request_frames: vec.iter().map(|x| x + start_frame).collect(),
                request_source_claim_end: request_source_claim_end + start_frame,
            })
        }
    }
    pub(crate) fn first_req_frame(&self, request_state: &RequestOffset) -> Option<u64> {
        if self.taken_requests[request_state.get()] {
            return None;
        }
        self.request_frames.get(request_state.get()).copied()
    }
    pub(crate) fn iter_skip_start(
        &'_ self,
        request_state: &RequestOffset,
    ) -> impl Iterator<Item = u64> {
        self.request_frames
            .iter()
            .zip(&self.taken_requests)
            .skip(request_state.get() + 1)
            .filter_map(|(req, taken)| if *taken { None } else { Some(req) })
            .copied()
    }
    pub(crate) fn claim_end_time(&self) -> u64 {
        self.request_source_claim_end
    }
    pub fn skip(&self, mut request_state: RequestOffset) -> Option<RequestOffset> {
        unsafe { request_state.increment() };
        self.next_unclaimed(request_state)
    }
    //attempts to go to the next unclaimed item,
    //returns false if there isn't one.
    pub fn next_unclaimed(&self, mut request_state: RequestOffset) -> Option<RequestOffset> {
        while request_state.get() < self.request_frames.len() {
            if !self.taken_requests[request_state.get()] {
                return Some(request_state);
            }
            unsafe { request_state.increment() };
        }
        None
    }
    pub(super) fn apply_commit_claim(&mut self, track: &EnemyTrack, commit: &FutureMoveCommit) {
        let mut index = 0;
        for active in commit.get_active_frames(track) {
            if active >= self.claim_end_time() {
                break;
            }
            while index < self.request_frames.len() {
                if self.request_frames[index] == active {
                    self.taken_requests[index] = true;
                    break;
                }
                index += 1;
            }
        }
        let mut index = 0;
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
        let mut req = ComplementAttackRequest::new(&vec![20, 32, 40], 100, 0).unwrap();
        req.taken_requests[2] = true;

        assert_eq!(
            req.iter_skip_start(&RequestOffset::new_default())
                .collect::<Vec<u64>>(),
            vec![32]
        );
    }

    #[test]
    fn test_filter_first() {
        let offset = RequestOffset::new_default();
        let mut req = ComplementAttackRequest::new(&vec![20, 32, 40, 90], 100, 0).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        assert_eq!(
            req.iter_skip_start(&offset).collect::<Vec<u64>>(),
            vec![32, 90]
        );

        req.taken_requests[2] = false;

        assert_eq!(
            req.iter_skip_start(&offset).collect::<Vec<u64>>(),
            vec![32, 40, 90]
        );
    }

    #[test]
    fn test_skip() {
        let mut offset = RequestOffset::new_default();
        let mut req = ComplementAttackRequest::new(&vec![20, 32, 40, 90], 100, 0).unwrap();
        req.taken_requests[2] = true;
        req.taken_requests[0] = true;

        offset = req.skip(offset).unwrap();
        assert_eq!(req.iter_skip_start(&offset).collect::<Vec<u64>>(), vec![90]);

        req.taken_requests[2] = false;

        assert_eq!(
            req.iter_skip_start(&offset).collect::<Vec<u64>>(),
            vec![40, 90]
        );
    }

    #[test]
    fn test_offset() {
        let mut offset = RequestOffset::new_default();
        let mut req = ComplementAttackRequest::new(&vec![20, 32, 40, 90], 100, 20).unwrap();
        req.taken_requests[2] = true;

        assert_eq!(
            req.iter_skip_start(&offset).collect::<Vec<u64>>(),
            vec![52, 110]
        );

        assert_eq!(req.first_req_frame(&offset), Some(40));

        offset = req.skip(offset).unwrap();

        assert_eq!(req.first_req_frame(&offset), Some(52));
    }
}
