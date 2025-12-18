use std::iter::Skip;

#[allow(unused)]
pub(crate) struct ComplementAttackRequest<'a> {
    request_frames: &'a Vec<u64>,
    request_offset: usize,
    request_source_claim_end: u64,
}

impl<'a> ComplementAttackRequest<'a> {
    pub(crate) fn new(vec: &'a Vec<u64>, offset: usize, request_source_claim_end: u64) -> Self {
        Self {
            request_offset: offset,
            request_frames: vec,
            request_source_claim_end,
        }
    }
    pub(crate) fn start_frame(&self) -> Option<u64> {
        self.request_frames.get(self.request_offset).copied()
    }
    pub(crate) fn iter_skip_start(&self) -> Skip<std::slice::Iter<'a, u64>> {
        self.request_frames.iter().skip(self.request_offset + 1)
    }
    pub(crate) fn claim_end_time(&self) -> u64 {
        self.request_source_claim_end
    }
}
