use self::attack_future_instance::AttackFutureInstance;
use self::complement_attack_request::ComplementAttackRequest;
use self::enemy_track_attack_wrapper::EnemyTrackAttack;
use crate::attack::Attack;

mod attack_future_instance;
mod complement_attack_request;

#[derive(Debug)]
pub struct EnemyTrack {
    attacks: Vec<EnemyTrackAttack>,
    future_stack: Vec<FutureMoveCommit>,
}

#[derive(Debug)]
struct FutureMoveCommit {
    frames_after_now: u64,
    move_index: usize,
}

impl FutureMoveCommit {
    fn new(attack_index: usize, frames_after_now: u64) -> Self {
        Self {
            frames_after_now,
            move_index: attack_index,
        }
    }
    fn get_full_duration(&self, parent_track: &EnemyTrack) -> u64 {
        parent_track.attacks[self.move_index]
            .get_attack()
            .get_full_duration()
    }
}

impl EnemyTrack {
    pub fn new(attacks: Vec<Attack>) -> Self {
        let attacks = attacks
            .into_iter()
            .scan(0, |a: &mut usize, b: Attack| {
                *a += 1;
                Some(enemy_track_attack_wrapper::EnemyTrackAttack::new(b, *a))
            })
            .collect();
        Self {
            attacks,
            future_stack: vec![],
        }
    }
    pub fn can_meet_request(&self, request: &ComplementAttackRequest) -> Vec<&EnemyTrackAttack> {
        let request_frame = request.start_frame();
        self.attacks
            .iter()
            .filter_map(|attack| {
                AttackFutureInstance::try_create(attack, request_frame, self.first_valid_frame())
            })
            .filter(|future_instance| future_instance.can_meet_request_followup(request))
            .map(AttackFutureInstance::to_attack)
            .collect()
    }
    pub fn first_valid_frame(&self) -> u64 {
        match self.future_stack.last() {
            Some(commit) => commit.get_full_duration(&self),
            None => 0,
        }
    }
    fn get_attack_frame(&self, attack_index: usize, request_frame: u64) -> Option<u64> {
        let first_actionable = self.first_valid_frame();
        self.attacks[attack_index]
            .get_attack()
            .get_start_frame(request_frame, first_actionable)
    }
    //commit possible future move
    fn commit(&mut self, request_frame: u64, attack_index: usize) -> bool {
        if let Some(attack_frame) = self.get_attack_frame(attack_index, request_frame) {
            self.future_stack
                .push(FutureMoveCommit::new(attack_index, attack_frame));
            return true;
        }
        false
    }
    //uncommit move
    fn uncommit(&mut self) -> bool {
        self.future_stack.pop().is_some()
    }
}

mod enemy_track_attack_wrapper {
    use super::Attack;

    #[derive(Debug)]
    pub struct EnemyTrackAttack {
        index: usize,
        attack: Attack,
    }

    impl EnemyTrackAttack {
        pub fn new(attack: Attack, index: usize) -> Self {
            Self { attack, index }
        }
        pub fn get_attack(&self) -> &Attack {
            &self.attack
        }
        pub fn get_index(&self) -> usize {
            self.index
        }
    }
}

#[cfg(test)]
mod enemy_track_tests {
    use super::*;

    #[test]
    fn can_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![12], vec![4]),
            Attack::new(20, vec![8, 10, 16], vec![4]),
        ]);
        assert_eq!(
            mock_track
                .can_meet_request(&ComplementAttackRequest::new(&vec![16, 24], 0, 100).unwrap())
                .len(),
            2
        );
    }

    #[test]
    fn commit_and_uncommit_test() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![8, 10, 16], vec![4]),
        ]);
        let request_frames = vec![16, 24];
        let request = ComplementAttackRequest::new(&request_frames, 0, 100).unwrap();
        
        let request_result = mock_track.can_meet_request(&request);
        assert!(!request_result.is_empty());
        
        assert!(mock_track.commit(request.start_frame(), request_result[0].get_index()));
        assert!(mock_track.uncommit());
    }

    #[test]
    fn test_catch_invalid_commit() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![8, 10, 16], vec![4]),
        ]);
        let request_frames = vec![16, 24];
        let request = ComplementAttackRequest::new(&request_frames, 0, 100).unwrap();
        
        let request_result = mock_track.can_meet_request(&request);
        assert!(!request_result.is_empty());
        
        assert!(!mock_track.commit(7, request_result[0].get_index()));
    }

    #[test]
    fn fail_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![12, 20], vec![4]),
        ]);

        assert!(
            mock_track
                .can_meet_request(&ComplementAttackRequest::new(&vec![19, 28], 0, 100).unwrap())
                .is_empty()
        );
    }
}
