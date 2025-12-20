use std::ops::RangeFrom;

use self::attack_future_instance::AttackFutureInstance;
use self::complement_attack_request::ComplementAttackRequest;
use self::enemy_track_attack_wrapper::EnemyTrackAttack;
use crate::attack::Attack;
use crate::enemy_track::future_move_commit::FutureMoveCommit;

mod attack_future_instance;
pub mod complement_attack_request;
mod enemy_track_attack_wrapper;
pub mod future_move_commit;

#[derive(Debug)]
pub struct EnemyTrack {
    attacks: Vec<EnemyTrackAttack>,
    future_stack: Vec<FutureMoveCommit>,
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
    pub fn can_meet_request_iter(
        &self,
        request: &ComplementAttackRequest,
    ) -> impl Iterator<Item = &EnemyTrackAttack> {
        let request_frame = request.start_frame();
        self.attacks
            .iter()
            .filter_map(move |attack| {
                AttackFutureInstance::try_create(
                    attack,
                    request_frame,
                    self.first_actionable_frame(),
                )
            })
            .filter(|future_instance| future_instance.can_meet_request_followup(request))
            .map(AttackFutureInstance::unwrap)
    }
    pub fn can_meet_request(&self, request: &ComplementAttackRequest) -> Vec<&EnemyTrackAttack> {
        self.can_meet_request_iter(request).collect()
    }
    pub fn can_meet_request_commits(
        &self,
        request: &ComplementAttackRequest,
    ) -> Vec<future_move_commit::FutureMoveCommit> {
        self.can_meet_request_iter(request)
            .map(|attack| {
                self.create_future_move(request.start_frame(), attack)
                    .unwrap()
            })
            .collect()
    }
    //FIXME: bad mutability practice, request should never be changed here, so it should be immutable
    //however, need to move it forwards and then revert to avoid issues.
    pub fn possible_future_commits(
        &self,
        request: &mut ComplementAttackRequest,
    ) -> Vec<FutureMoveCommit> {
        let mut collection = self.can_meet_request_commits(request);
        let restore = request.get_restore_point();
        while request.skip() {
            let mut skipped_add = self.can_meet_request_commits(request);
            collection.append(&mut skipped_add);
        }
        request.restore(restore);
        collection
    }
    fn last_future_stack_item(&self) -> Option<&FutureMoveCommit> {
        self.future_stack.last()
    }
    pub fn last_queued_attack(&self) -> Option<&EnemyTrackAttack> {
        self.last_future_stack_item()
            .map(|commit| commit.get_attack(self))
    }
    pub fn last_queued_attack_as_request(&self) -> Option<ComplementAttackRequest> {
        self.last_queued_attack()
            .map(EnemyTrackAttack::get_attack)
            .map(|attack| attack.into())
    }
    pub fn first_actionable_frame(&self) -> u64 {
        match self.last_future_stack_item() {
            Some(commit) => commit.get_full_duration(self),
            None => 0,
        }
    }
    fn get_attack_frame(&self, attack_index: usize, request_frame: u64) -> Option<u64> {
        let first_actionable = self.first_actionable_frame();
        self.attacks[attack_index]
            .get_attack()
            .get_start_frame(request_frame, first_actionable)
    }
    fn create_future_move(
        &self,
        start_frame: u64,
        attack: &EnemyTrackAttack,
    ) -> Option<FutureMoveCommit> {
        let attack_index = attack.get_index();
        if let Some(attack_frame) = self.get_attack_frame(attack_index, start_frame) {
            let commit = FutureMoveCommit::new(attack_index, attack_frame);
            return Some(commit);
        }
        None
    }
    //commit possible future move
    fn commit_request_and_index(
        &mut self,
        request: &mut ComplementAttackRequest,
        attack_index: usize,
    ) -> bool {
        if let Some(commit) =
            self.create_future_move(request.start_frame(), &self.attacks[attack_index])
        {
            self.commit(request, commit);
            return true;
        }
        false
    }
    pub fn commit(&mut self, request: &mut ComplementAttackRequest, commit: FutureMoveCommit) {
        request.apply_commit_claim(self, &commit, false);
        self.future_stack.push(commit);
    }
    pub fn commit_by_index(&mut self, attack_index: usize) -> bool {
        let commit = FutureMoveCommit::new(attack_index, 0);
        self.future_stack.push(commit);
        return true;
    }
    //uncommit move
    pub fn uncommit(&mut self, request: &mut ComplementAttackRequest) -> bool {
        if let Some(commit) = self.future_stack.pop() {
            request.apply_commit_claim(self, &commit, true);

            return true;
        }
        false
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

        assert!(mock_track.commit_request_and_index(&mut request, request_result[0].get_index()));
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

        assert!(!mock_track.commit_request_and_index(
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
        mock_track.commit_request_and_index(&mut src_request, second_opt.get_index());
        assert!(src_request.next_unclaimed());

        let new_can_meet = mock_track.can_meet_request(&src_request);

        assert_eq!(new_can_meet.len(), 2);
        dbg!(&new_can_meet);
        mock_track.commit_request_and_index(&mut src_request, new_can_meet[0].get_index());
        assert!(!src_request.next_unclaimed());
    }

    #[test]
    fn can_deny_inapplicable_request() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![1, 9], vec![]),
            Attack::new(6, vec![5], vec![]),
            Attack::new(13, vec![2, 4, 6], vec![]),
        ]);

        let src = Attack::new(25, vec![], vec![10, 18]);
        let mut src_request: ComplementAttackRequest = src.into();

        let can_meet = mock_track.can_meet_request(&src_request);
        assert_eq!(can_meet.len(), 2);
        mock_track.commit_request_and_index(&mut src_request, can_meet[1].get_index());
        assert!(src_request.next_unclaimed());

        let can_meet = mock_track.can_meet_request(&src_request);
        assert_eq!(can_meet.len(), 2);
        mock_track.commit_request_and_index(&mut src_request, can_meet[0].get_index());
        assert!(!src_request.next_unclaimed());

        mock_track.uncommit(&mut src_request);
        src_request.prev_unclaimed();

        let can_meet = mock_track.can_meet_request(&src_request);
        assert_eq!(can_meet.len(), 2);
        mock_track.commit_request_and_index(&mut src_request, can_meet[0].get_index());
        assert!(!src_request.next_unclaimed());

        assert!(mock_track.uncommit(&mut src_request));
        src_request.prev_unclaimed();
        assert!(mock_track.uncommit(&mut src_request));
        src_request.prev_unclaimed();
        dbg!(&src_request);
    }

    #[test]
    fn can_match_all_futures() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![1, 10], vec![]),
            Attack::new(6, vec![5], vec![]),
            Attack::new(13, vec![2, 4, 6], vec![]),
        ]);

        let mock_lead_track = Attack::new(25, vec![], vec![10, 18]);
        let mut mock_request: ComplementAttackRequest = mock_lead_track.into();

        let can_meet = mock_track.possible_future_commits(&mut mock_request);
        dbg!(&can_meet);
        assert_eq!(can_meet.len(), 3);
    }
}
