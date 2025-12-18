
use crate::attack::Attack;

#[allow(unused)]
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
    fn claim_request_frame(&mut self, index: usize) -> bool {
        self.taken_requests[index] = true;
        true
    }
}
