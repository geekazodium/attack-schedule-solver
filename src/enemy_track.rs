use std::ops::RangeFrom;

use self::attack_future_instance::AttackFutureInstance;
use self::complement_attack_request::ComplementAttackRequest;
use self::enemy_track_attack_wrapper::EnemyTrackAttack;
use crate::attack::Attack;

mod attack_future_instance;
mod complement_attack_request;
mod enemy_track_attack_wrapper;

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
    fn get_attack<'a>(&self, parent_track: &'a EnemyTrack) -> &'a EnemyTrackAttack {
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
    fn get_full_duration(&self, parent_track: &EnemyTrack) -> u64 {
        self.get_attack(parent_track)
            .get_attack()
            .get_full_duration()
    }
}

impl EnemyTrack {
    pub fn new(attacks: Vec<Attack>) -> Self {
        let attacks = attacks
            .into_iter()
            .zip(RangeFrom { start: 0 })
            .map(|(a, index)| EnemyTrackAttack::new(a, index))
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
            Some(commit) => commit.get_full_duration(self),
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
    fn commit(&mut self, request: &mut ComplementAttackRequest, attack_index: usize) -> bool {
        if let Some(attack_frame) = self.get_attack_frame(attack_index, request.start_frame()) {
            let commit = FutureMoveCommit::new(attack_index, attack_frame);

            request.apply_commit_claim(&self, &commit, false);

            self.future_stack.push(commit);

            return true;
        }
        false
    }
    //uncommit move
    fn uncommit(&mut self, request: &mut ComplementAttackRequest) -> bool {
        if let Some(commit) = self.future_stack.pop() {
            request.apply_commit_claim(&self, &commit, true);

            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod enemy_track_tests {
    use super::*;

    #[test]
    fn can_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(20, vec![8, 10, 16], vec![4]),
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![12], vec![4]),
        ]);
        assert_eq!(
            mock_track
                .can_meet_request(&Attack::new(30, vec![], vec![20, 28]).into())
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
        let mock_lead_action = Attack::new(30, vec![], vec![16, 24]);
        let mut request = mock_lead_action.into();
        let request_result = mock_track.can_meet_request(&request);
        assert!(!request_result.is_empty());

        assert!(mock_track.commit(&mut request, request_result[0].get_index()));
        assert!(mock_track.uncommit(&mut request));
    }

    #[test]
    fn test_catch_invalid_commit() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![8, 10, 16], vec![4]),
        ]);
        let mock_lead_action = Attack::new(30, vec![], vec![16, 24]);

        let request_result = mock_track.can_meet_request(&mock_lead_action.into());
        assert!(!request_result.is_empty());

        assert!(!mock_track.commit(
            &mut Attack::new(10, vec![], vec![3]).into(),
            request_result[0].get_index()
        ));
    }

    #[test]
    fn fail_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![12, 20], vec![4]),
        ]);

        let mock_lead_action = Attack::new(25, vec![10, 16], vec![13, 29]);
        assert!(
            mock_track
                .can_meet_request(&mock_lead_action.into())
                .is_empty()
        );
    }

    #[test]
    fn can_meet_multuple_request() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![1, 9], vec![2]),
            Attack::new(13, vec![12], vec![4]),
        ]);

        let src = Attack::new(30, vec![], vec![20, 28]);

        let mut src_request: ComplementAttackRequest = src.into();
        let can_meet = mock_track.can_meet_request(&src_request);

        assert_eq!(can_meet.len(), 2);

        let second_opt: &EnemyTrackAttack = can_meet[1];
        dbg!(&second_opt);
        mock_track.commit(&mut src_request, second_opt.get_index());
        assert!(src_request.next_unclaimed());

        let new_can_meet = mock_track.can_meet_request(&src_request);

        assert_eq!(new_can_meet.len(), 2);
        dbg!(&new_can_meet);
        mock_track.commit(&mut src_request, new_can_meet[0].get_index());
        assert!(!src_request.next_unclaimed());
    }


    #[test]
    fn can_deny_inapplicable_request() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![1, 9], vec![2]),
            Attack::new(6, vec![5], vec![4]),
            Attack::new(13, vec![2, 4, 6], vec![4]),
        ]);

        let src = Attack::new(25, vec![], vec![10, 18]);

        let mut src_request: ComplementAttackRequest = src.into();
        let can_meet = mock_track.can_meet_request(&src_request);

        assert_eq!(can_meet.len(), 2);

        let second_opt: &EnemyTrackAttack = can_meet[1];
        dbg!(&second_opt);
        mock_track.commit(&mut src_request, second_opt.get_index());
        assert!(src_request.next_unclaimed());

        let new_can_meet = mock_track.can_meet_request(&src_request);

        assert_eq!(new_can_meet.len(), 2);
        dbg!(&new_can_meet);
        mock_track.commit(&mut src_request, new_can_meet[0].get_index());
        assert!(!src_request.next_unclaimed());
    }
}
