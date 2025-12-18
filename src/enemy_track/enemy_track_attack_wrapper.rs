use super::Attack;

#[derive(Debug)]
pub struct EnemyTrackAttack {
    index: usize,
    attack: Attack,
}

impl EnemyTrackAttack {
    pub fn new(attack: Attack, index: usize) -> Self {
        dbg!(index);
        Self { attack, index }
    }
    pub fn get_attack(&self) -> &Attack {
        &self.attack
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
}
