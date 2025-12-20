use super::EnemyTrackAttack;

use super::EnemyTrack;

#[derive(Debug)]
pub struct FutureMoveCommit {
    pub(crate) frames_after_now: u64,
    pub(crate) move_index: usize,
}

impl FutureMoveCommit {
    pub(crate) fn new(attack_index: usize, frames_after_now: u64) -> Self {
        Self {
            frames_after_now,
            move_index: attack_index,
        }
    }
    pub(crate) fn get_attack<'a>(&self, parent_track: &'a EnemyTrack) -> &'a EnemyTrackAttack {
        &parent_track.attacks[self.move_index]
    }
    pub fn get_active_frames<'a>(
        &self,
        parent_track: &'a EnemyTrack,
    ) -> impl 'a + Iterator<Item = u64> {
        self.get_attack(parent_track)
            .get_attack()
            .get_active_frames(self.frames_after_now)
    }
    pub(crate) fn get_full_duration(&self, parent_track: &EnemyTrack) -> u64 {
        self.get_attack(parent_track)
            .get_attack()
            .get_full_duration()
    }
}
