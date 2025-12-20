#[derive(Debug)]
pub struct Attack {
    duration: u64,
    active: Vec<u64>,
    active_request_frames: Vec<u64>,
}

impl Attack {
    #[must_use]
    pub fn new(duration: u64, active: Vec<u64>, request_frames: Vec<u64>) -> Self {
        Self {
            duration,
            active,
            active_request_frames: request_frames,
        }
    }
    fn get_last_active(&self) -> Option<u64> {
        self.active.last().map(|x| self.duration - x)
    }
    #[must_use]
    pub fn get_end_frame(&self) -> Option<u64> {
        self.get_last_active()
    }
    #[must_use]
    pub fn get_full_duration(&self) -> u64 {
        self.duration
    }
    #[must_use]
    pub fn get_cooldown(&self) -> u64 {
        self.get_last_active().unwrap_or(self.duration)
    }
    #[must_use]
    pub fn get_start_frame(&self, request_frame: u64, first_actionable: u64) -> Option<u64> {
        self.active
            .first()
            .filter(|first_active| (**first_active + first_actionable <= request_frame))
            .map(|x| request_frame - x)
    }
    #[must_use]
    pub fn active_request_frames(&self) -> &Vec<u64> {
        &self.active_request_frames
    }
    pub fn get_active_frames(&self, start: u64) -> impl Iterator<Item = u64> {
        self.active.iter().map(move |x| x + start)
    }
}

#[cfg(test)]
mod attack_tests {
    use super::*;

    #[test]
    fn cooldown_valid() {
        let a = Attack::new(10, vec![8], vec![4]);
        assert_eq!(a.get_cooldown(), 2);
    }

    #[test]
    fn start_frame_valid() {
        let a = Attack::new(10, vec![8], vec![4]);
        assert_eq!(a.get_start_frame(15, 0), Some(7));
    }

    #[test]
    fn start_frame_invalid_due_to_actionable() {
        let a = Attack::new(10, vec![8], vec![4]);
        assert_eq!(a.get_start_frame(15, 8), None);
    }
    #[test]
    fn start_frame_invalid_due_to_move_length() {
        let a = Attack::new(10, vec![16], vec![4]);
        assert_eq!(a.get_start_frame(15, 0), None);
    }

    #[test]
    fn test_offsetting() {
        let a = Attack::new(10, vec![8, 10, 24], vec![4]);
        let mut expected = vec![28, 30, 44];
        for n in a.get_active_frames(20) {
            assert_eq!(expected.remove(0), n);
        }
    }
}
