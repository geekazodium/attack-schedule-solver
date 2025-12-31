pub struct RequestOffset {
    request_offset: usize,
}

impl RequestOffset {
    pub fn new_default() -> Self {
        Self::new(0)
    }
    pub fn new(point: usize) -> Self {
        Self {
            request_offset: point,
        }
    }
    pub fn get(&self) -> usize {
        self.request_offset
    }
    // unsafe as this preincrements the value but DOES NOT DO ANY CHECK
    // to verify that this results in a valid state afterwards.
    pub unsafe fn increment(&mut self) -> usize {
        self.request_offset += 1;
        self.request_offset
    }
}
