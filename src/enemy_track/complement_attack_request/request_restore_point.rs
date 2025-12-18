pub struct RequestRestorePoint {
    point: usize,
}

impl RequestRestorePoint {
    pub(super) fn new(point: usize) -> Self {
        Self { point }
    }
    pub fn get(&self) -> usize {
        self.point
    }
}
