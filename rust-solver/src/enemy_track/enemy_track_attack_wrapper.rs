use super::Attack;

#[derive(Debug)]
pub struct EnemyTrackAttack {
    index: usize,
    attack: Attack,
}

impl EnemyTrackAttack {
    pub fn new(attack: Attack, index: usize) -> Self {
        Self { index, attack }
    }
    pub fn get_attack(&self) -> &Attack {
        &self.attack
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
    pub fn start_frame_and_index(
        &self,
        request_frame: u64,
        first_actionable: u64,
    ) -> Option<(usize, u64)> {
        self.get_attack()
            .get_start_frame(request_frame, first_actionable)
            .map(|v| (self.get_index(), v))
    }
}
