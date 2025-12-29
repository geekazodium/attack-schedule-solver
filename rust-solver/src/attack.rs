use crate::enemy_track::complement_attack_request::ComplementAttackRequest;

#[derive(Debug)]
pub struct Attack {
    duration: u64,
    active: Vec<u64>,
    active_request_frames: Vec<u64>,
}

impl Attack {
    #[cfg(test)]
    #[must_use]
    pub fn new_expect(duration: u64, active: Vec<u64>, request_frames: Vec<u64>) -> Self {
        Self::new(duration, active, request_frames).expect("invalid instance parameters")
    }
    #[must_use]
    pub fn new(duration: u64, active: Vec<u64>, request_frames: Vec<u64>) -> Option<Self> {
        if !active.is_sorted() {
            return None;
        }
        if !request_frames.is_sorted() {
            return None;
        }
        if active.last().is_some_and(|v| *v >= duration) {
            return None;
        }
        if request_frames.last().is_some_and(|v| *v >= duration) {
            return None;
        }

        unsafe { Some(Self::new_unchecked(duration, active, request_frames)) }
    }
    #[must_use]
    pub unsafe fn new_unchecked(duration: u64, active: Vec<u64>, request_frames: Vec<u64>) -> Self {
        Self {
            duration,
            active,
            active_request_frames: request_frames,
        }
    }
    #[must_use]
    pub fn get_full_duration(&self) -> u64 {
        self.duration
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
    pub fn to_request(&self, start: u64) -> Option<ComplementAttackRequest> {
        ComplementAttackRequest::new(
            self.active_request_frames().clone(),
            self.get_full_duration(),
            start,
        )
    }
}

#[cfg(test)]
mod attack_tests {
    use super::*;

    #[test]
    fn start_frame_valid() {
        let a = Attack::new_expect(10, vec![8], vec![4]);
        assert_eq!(a.get_start_frame(15, 0), Some(7));
    }

    #[test]
    fn start_frame_invalid_due_to_actionable() {
        let a = Attack::new_expect(10, vec![8], vec![4]);
        assert_eq!(a.get_start_frame(15, 8), None);
    }
    #[test]
    fn start_frame_invalid_due_to_move_length() {
        let a = Attack::new(10, vec![16], vec![4]);
        assert!(a.is_none());
    }

    #[test]
    fn test_offsetting() {
        let a = Attack::new_expect(30, vec![8, 10, 24], vec![4]);
        let mut expected = vec![28, 30, 44];
        for n in a.get_active_frames(20) {
            assert_eq!(expected.remove(0), n);
        }
    }
}
