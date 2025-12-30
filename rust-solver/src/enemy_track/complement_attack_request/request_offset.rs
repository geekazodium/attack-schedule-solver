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
    pub fn increment(&mut self) -> usize {
        self.request_offset += 1;
        self.request_offset
    }
}
