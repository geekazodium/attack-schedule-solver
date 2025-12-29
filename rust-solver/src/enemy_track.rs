use self::attack_future_instance::AttackFutureInstance;
use self::complement_attack_request::ComplementAttackRequest;
use self::enemy_track_attack_wrapper::EnemyTrackAttack;
use crate::attack::Attack;
use crate::enemy_track::future_move_commit::FutureMoveCommit;
use std::ops::RangeFrom;

mod attack_future_instance;
pub mod complement_attack_request;
mod enemy_track_attack_wrapper;
pub mod future_move_commit;

#[derive(Debug)]
pub struct EnemyTrack {
    attacks: Vec<EnemyTrackAttack>,
    attacks_validitiy: Vec<bool>,
    future_stack: Vec<FutureMoveCommit>,
}

impl EnemyTrack {
    #[must_use]
    pub fn new(attacks: Vec<Attack>) -> Self {
        let attacks: Vec<EnemyTrackAttack> = attacks
            .into_iter()
            .zip(RangeFrom { start: 0 })
            .map(|(a, index)| EnemyTrackAttack::new(a, index))
            .collect();
        let attacks_validitiy = (0..attacks.len()).map(|_| true).collect();
        Self {
            attacks,
            attacks_validitiy,
            future_stack: vec![],
        }
    }
    pub fn possible_now_moves_iter(
        &self,
        request: &ComplementAttackRequest,
        request_frame: u64,
    ) -> impl Iterator<Item = &EnemyTrackAttack> {
        self.attacks
            .iter()
            .zip(&self.attacks_validitiy)
            .filter_map(move |(attack, valid)| if *valid { Some(attack) } else { None })
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
    pub fn latest_nonpast_commit(&self) -> Option<&FutureMoveCommit> {
        self.future_stack.first()
    }
    pub fn update_latest_nonpast(&mut self, time: u64) {
        let mut i = 0;
        while let Some(zeroth) = self.future_stack.get(i) {
            if zeroth.get_end_frame(self) > time {
                break;
            }
            i += 1;
        }

        self.future_stack.rotate_left(i);
        for _ in 0..i {
            self.future_stack.pop();
        }
    }
    #[must_use]
    pub fn possible_now_commits(&self, request: &ComplementAttackRequest) -> Vec<FutureMoveCommit> {
        if let Some(request_frame) = request.first_req_frame() {
            self.possible_now_moves_iter(request, request_frame)
                .filter_map(|attack| self.create_future_move(request_frame, attack))
                .collect()
        } else {
            vec![]
        }
    }
    //FIXME: bad mutability practice, request should never be changed here, so it should be immutable
    //however, need to move it forwards and then revert to avoid issues.
    pub fn possible_future_commits(
        &self,
        request: &mut ComplementAttackRequest,
    ) -> Vec<FutureMoveCommit> {
        let mut collection = self.possible_now_commits(request);
        let restore = request.get_restore_point();
        while request.skip() {
            let mut skipped_add = self.possible_now_commits(request);
            collection.append(&mut skipped_add);
        }
        request.restore(&restore);
        collection
    }
    fn last_future_stack_item(&self) -> Option<&FutureMoveCommit> {
        self.future_stack.last()
    }
    pub fn last_queued_attack_as_request(&self) -> Option<ComplementAttackRequest> {
        self.last_future_stack_item().map(|commit| {
            commit
                .get_attack(self)
                .get_attack()
                .to_request(commit.get_start_frame())
        }).flatten()
    }
    #[must_use]
    pub fn first_actionable_frame(&self) -> u64 {
        match self.last_future_stack_item() {
            Some(commit) => commit.get_end_frame(self),
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
    //FIXME: in future, make sure commits are checked to be valid before allowing
    pub fn commit(&mut self, request: &mut ComplementAttackRequest, commit: FutureMoveCommit) {
        request.apply_commit_claim(self, &commit, false);
        self.future_stack.push(commit);
    }
    pub fn commit_by_index(&mut self, attack_index: usize, start_time: u64) -> bool {
        if !self.is_actionable_now(start_time) {
            return false;
        }
        let commit = FutureMoveCommit::new(attack_index, start_time);
        self.future_stack.push(commit);
        true
    }
    pub fn is_actionable_now(&self, start_time: u64) -> bool {
        self.first_actionable_frame() <= start_time
    }
}

#[cfg(test)]
mod enemy_track_tests {
    use super::*;

    impl From<&Attack> for ComplementAttackRequest{
        fn from(value: &Attack) -> Self {
            value.to_request(0).unwrap()
        }
    }
    impl From<Attack> for ComplementAttackRequest{
        fn from(value: Attack) -> Self {
            (&value).into()
        }
    }

    impl EnemyTrack {
        #[must_use]
        pub fn possible_now_moves(
            &self,
            request: &ComplementAttackRequest,
        ) -> Vec<&EnemyTrackAttack> {
            if let Some(request_frame) = request.first_req_frame() {
                self.possible_now_moves_iter(request, request_frame)
                    .collect()
            } else {
                vec![]
            }
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

    fn commit_and_assert(
        request: &mut ComplementAttackRequest,
        track: &mut EnemyTrack,
        take_option_index: usize,
        expected_len: usize,
        expected_next_unclaimed: bool,
    ) {
        let mut commits = track.possible_now_commits(&request);
        assert_eq!(commits.len(), expected_len);
        track.commit(request, commits.swap_remove(take_option_index));
        assert_eq!(request.next_unclaimed(), expected_next_unclaimed);
    }

    fn assert_commits_length(
        request: &ComplementAttackRequest,
        track: &EnemyTrack,
        expected_len: usize,
    ) {
        let commits = track.possible_now_commits(request);
        assert_eq!(commits.len(), expected_len);
    }

    #[test]
    fn can_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(20, vec![8, 10, 16], vec![]),
            Attack::new(10, vec![8, 16], vec![]),
            Attack::new(20, vec![12], vec![]),
        ]);
        let mock_request: ComplementAttackRequest =
            Attack::new(30, vec![], vec![20, 28]).into();
        assert_commits_length(&mock_request, &mock_track, 2);
    }

    #[test]
    fn commit_and_uncommit_test() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![8, 10, 16], vec![4]),
        ]);
        let mock_lead_action = Attack::new(30, vec![], vec![16, 24]);
        let mut mock_request = mock_lead_action.into();

        commit_and_assert(&mut mock_request, &mut mock_track, 0, 1, false);

        assert!(mock_track.uncommit(&mut mock_request));
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
                .possible_now_moves(&mock_lead_action.into())
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
        let mut mock_request: ComplementAttackRequest = src.into();

        commit_and_assert(&mut mock_request, &mut mock_track, 1, 2, true);
        commit_and_assert(&mut mock_request, &mut mock_track, 0, 1, false);
    }

    #[test]
    fn can_deny_inapplicable_request() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![1, 9], vec![]),
            Attack::new(6, vec![5], vec![]),
            Attack::new(13, vec![2, 4, 6], vec![]),
        ]);

        let src = Attack::new(25, vec![], vec![10, 18]);
        let mut mock_request: ComplementAttackRequest = src.into();

        let restore_point_a = mock_request.get_restore_point();
        commit_and_assert(&mut mock_request, &mut mock_track, 1, 2, true);

        let restore_point_b = mock_request.get_restore_point();

        commit_and_assert(&mut mock_request, &mut mock_track, 0, 2, false);

        mock_track.uncommit(&mut mock_request);
        mock_request.restore(&restore_point_b);

        commit_and_assert(&mut mock_request, &mut mock_track, 0, 2, false);

        assert!(mock_track.uncommit(&mut mock_request));
        mock_request.restore(&restore_point_b);
        assert!(mock_track.uncommit(&mut mock_request));
        mock_request.restore(&restore_point_a);
        dbg!(&mock_request);
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
