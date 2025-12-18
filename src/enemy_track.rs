use self::attack_future_instance::AttackFutureInstance;
use self::complement_attack_request::ComplementAttackRequest;
use crate::attack::Attack;

mod attack_future_instance;
mod complement_attack_request;

#[derive(Debug)]
struct EnemyTrack {
    attacks: Vec<Attack>,
    future_stack: Vec<u64>,
}

impl EnemyTrack {
    pub fn new(attacks: Vec<Attack>) -> Self {
        Self {
            attacks,
            future_stack: vec![],
        }
    }
    #[allow(unused)]
    pub fn can_meet_request(&self, request: &ComplementAttackRequest) -> Vec<&Attack> {
        match request.start_frame() {
            Some(request_frame) => self
                .attacks
                .iter()
                .filter_map(|attack| AttackFutureInstance::try_create(attack, request_frame))
                .filter(|future_instance| future_instance.can_meet_request_followup(request))
                .map(AttackFutureInstance::to_attack)
                .collect(),
            None => vec![],
        }
    }
    //commit move
    //uncommit move
}

#[cfg(test)]
mod enemy_track_tests {
    use super::*;

    #[test]
    fn can_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![12], vec![4]),
            Attack::new(20, vec![8,10,16], vec![4]),
        ]);
        assert_eq!(
            mock_track
                .can_meet_request(&ComplementAttackRequest::new(&vec![16, 24], 0, 100))
                .len(),
            2
        );
    }

    #[test]
    fn fail_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![12, 20], vec![4]),
        ]);

        assert!(
            mock_track
                .can_meet_request(&ComplementAttackRequest::new(&vec![19, 28], 0, 100))
                .is_empty()
        );
    }
}
