use self::complement_attack_request::ComplementAttackRequest;
use self::enemy_track_attack_wrapper::EnemyTrackAttack;
use crate::attack::Attack;
use crate::enemy_track::complement_attack_request::request_offset::RequestOffset;
use crate::enemy_track::future_move_commit::FutureMoveCommit;
use std::ops::RangeFrom;

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
    pub fn set_validity(&mut self, index: usize, valid: bool) {
        self.attacks_validitiy[index] = valid;
    }
    pub fn reset_validity(&mut self, valid: bool) {
        self.attacks_validitiy.fill(valid);
    }
    pub fn commit_valid(&self, commit: &FutureMoveCommit) -> bool {
        self.attacks_validitiy
            .get(commit.get_index())
            .is_some_and(|v| *v)
    }
    fn valid_attacks(&self) -> impl Iterator<Item = &EnemyTrackAttack> {
        self.attacks
            .iter()
            .zip(&self.attacks_validitiy)
            .filter_map(|(attack, valid)| if *valid { Some(attack) } else { None })
    }
    fn possible_now_moves_iter(
        &self,
        request: &ComplementAttackRequest,
        offset: &RequestOffset,
        request_frame: u64,
        time_now: u64,
    ) -> impl Iterator<Item = FutureMoveCommit> {
        let first_actionable = self.first_actionable_frame(time_now);

        self.valid_attacks()
            .filter_map(move |attack| attack.start_frame_and_index(request_frame, first_actionable))
            .filter_map(move |(attack, start_frame)| {
                FutureMoveCommit::try_create(attack, start_frame, first_actionable)
            })
            .filter(|future_instance| {
                future_instance.can_meet_request_followup(self, request, offset)
            })
    }
    pub fn get_attack(&self, index: usize) -> &Attack {
        self.attacks[index].get_attack()
    }
    pub fn latest_nonpast_commit(&self) -> Option<&FutureMoveCommit> {
        self.future_stack.first()
    }
    pub fn update_latest_nonpast(&mut self, time: u64) {
        if let Some(zeroth) = self.future_stack.first() {
            if zeroth.get_end_frame(self) > time {
                return;
            }
        } else {
            return;
        }
        self.future_stack.remove(0);
    }
    #[must_use]
    pub fn possible_now_commits(
        &self,
        request: &ComplementAttackRequest,
        offset: &RequestOffset,
        time_now: u64,
    ) -> Vec<FutureMoveCommit> {
        if let Some(request_frame) = request.first_req_frame(offset) {
            self.possible_now_moves_iter(request, offset, request_frame, time_now)
                .collect()
        } else {
            vec![]
        }
    }
    pub fn possible_future_commits(
        &self,
        request: &ComplementAttackRequest,
        time_now: u64,
    ) -> Vec<FutureMoveCommit> {
        let mut offset = RequestOffset::new_default();
        let mut collection = self.possible_now_commits(request, &offset, time_now);
        while let Some(tmp) = request.skip(offset) {
            offset = tmp;
            let mut skipped_add = self.possible_now_commits(request, &offset, time_now);
            collection.append(&mut skipped_add);
        }
        collection
    }
    fn last_future_stack_item(&self) -> Option<&FutureMoveCommit> {
        self.future_stack.last()
    }
    pub fn get_commit_as_request(
        &self,
        commit: &FutureMoveCommit,
    ) -> Option<ComplementAttackRequest> {
        self.get_attack(commit.get_index())
            .to_request(commit.get_start_frame())
    }
    pub fn last_queued_attack_as_request(&self) -> Option<ComplementAttackRequest> {
        self.last_future_stack_item()
            .and_then(|commit| self.get_commit_as_request(commit))
    }
    #[must_use]
    pub fn first_actionable_frame(&self, time_now: u64) -> u64 {
        match self.last_future_stack_item() {
            Some(commit) => commit.get_end_frame(self),
            None => time_now,
        }
    }
    //FIXME: in future, make sure commits are checked to be valid before allowing
    pub fn commit(&mut self, request: &mut ComplementAttackRequest, commit: FutureMoveCommit) {
        request.apply_commit_claim(self, &commit);
        self.future_stack.push(commit);
    }
    pub fn is_actionable_now(&self, start_time: u64, time_now: u64) -> bool {
        self.first_actionable_frame(time_now) <= start_time
    }
    pub fn commit_by_index(&mut self, attack_index: usize, start_time: u64, time_now: u64) -> bool {
        if !self.attacks_validitiy.get(attack_index).is_some_and(|v| *v) {
            return false;
        }
        let maybe_commit = FutureMoveCommit::try_create(
            attack_index,
            start_time,
            self.first_actionable_frame(time_now),
        );

        if let Some(commit) = maybe_commit {
            self.future_stack.push(commit);
            return true;
        }
        false
    }
    // reset all non-current moves on the future stack.
    // DOES NOT RESET ANYTHING THAT IS CURRENT EVEN IF THE ACTION IS NO LONGER
    // VALID, AS THAT ACTION HAS ALREADY BEEN DONE AND CAN NOT BE BROKEN
    // HALFWAY.
    pub fn reset_non_current(&mut self, now: u64) {
        let now_commit = if self.future_stack.is_empty() {
            None
        } else {
            let n = self.future_stack.swap_remove(0);
            if n.get_start_frame().le(&now) {
                Some(n)
            } else {
                None
            }
        };
        self.future_stack.clear();
        if let Some(commit) = now_commit {
            self.future_stack.push(commit);
        }
    }
}

#[cfg(test)]
mod enemy_track_tests {
    use super::*;

    impl From<&Attack> for ComplementAttackRequest {
        fn from(value: &Attack) -> Self {
            value.to_request(0).unwrap()
        }
    }
    impl From<Attack> for ComplementAttackRequest {
        fn from(value: Attack) -> Self {
            (&value).into()
        }
    }

    impl EnemyTrack {
        #[must_use]
        pub fn possible_now_moves(
            &self,
            request: &ComplementAttackRequest,
            offset: &RequestOffset,
        ) -> Vec<&EnemyTrackAttack> {
            if let Some(request_frame) = request.first_req_frame(offset) {
                self.possible_now_moves_iter(request, offset, request_frame, 0)
                    .map(|commit| &self.attacks[commit.get_index()])
                    .collect()
            } else {
                vec![]
            }
        }
    }

    fn commit_and_assert(
        request: &mut ComplementAttackRequest,
        offset: RequestOffset,
        track: &mut EnemyTrack,
        take_option_index: usize,
        expected_len: usize,
        expected_next_unclaimed: bool,
    ) -> Option<RequestOffset> {
        let mut commits = track.possible_now_commits(&request, &offset, 0);
        assert_eq!(commits.len(), expected_len);
        track.commit(request, commits.swap_remove(take_option_index));
        let tmp = request.next_unclaimed(offset);
        assert_eq!(tmp.is_some(), expected_next_unclaimed);
        tmp
    }

    fn assert_commits_length(
        request: &ComplementAttackRequest,
        track: &EnemyTrack,
        expected_len: usize,
    ) {
        let commits = track.possible_now_commits(request, &RequestOffset::new_default(), 0);
        assert_eq!(commits.len(), expected_len);
    }

    #[test]
    fn can_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new_expect(20, vec![8, 10, 16], vec![]),
            Attack::new_expect(20, vec![8, 16], vec![]),
            Attack::new_expect(20, vec![12], vec![]),
        ]);
        let mock_request: ComplementAttackRequest =
            Attack::new_expect(30, vec![], vec![20, 28]).into();
        assert_commits_length(&mock_request, &mock_track, 2);
    }

    #[test]
    fn reject_due_to_invalid() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new_expect(20, vec![8, 16], vec![]),
            Attack::new_expect(20, vec![8, 10, 16], vec![]),
            Attack::new_expect(20, vec![12], vec![]),
        ]);
        let mock_request: ComplementAttackRequest =
            Attack::new_expect(30, vec![], vec![20, 28]).into();
        mock_track.set_validity(0, false);
        mock_track.set_validity(2, false);
        assert_commits_length(&mock_request, &mock_track, 0);
    }

    #[test]
    fn reset_to_valid() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new_expect(20, vec![8, 16], vec![]),
            Attack::new_expect(20, vec![8, 10, 16], vec![]),
            Attack::new_expect(20, vec![12], vec![]),
        ]);
        let mock_request: ComplementAttackRequest =
            Attack::new_expect(30, vec![], vec![20, 28]).into();
        mock_track.set_validity(0, false);
        mock_track.set_validity(2, false);
        mock_track.reset_validity(true);
        assert_commits_length(&mock_request, &mock_track, 2);
    }

    #[test]
    fn fail_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new_expect(18, vec![8, 16], vec![2]),
            Attack::new_expect(22, vec![12, 20], vec![4]),
        ]);

        let mock_lead_action = Attack::new_expect(40, vec![10, 16], vec![13, 29]);
        assert!(
            mock_track
                .possible_now_moves(&mock_lead_action.into(), &RequestOffset::new_default())
                .is_empty()
        );
    }

    #[test]
    fn can_meet_multuple_request() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new_expect(10, vec![1, 9], vec![2]),
            Attack::new_expect(13, vec![12], vec![4]),
        ]);

        let src = Attack::new_expect(30, vec![], vec![20, 28]);
        let mut mock_request: ComplementAttackRequest = src.into();

        let mut offset: RequestOffset = RequestOffset::new_default();
        offset = commit_and_assert(&mut mock_request, offset, &mut mock_track, 1, 2, true).unwrap();
        commit_and_assert(&mut mock_request, offset, &mut mock_track, 0, 1, false);
    }

    #[test]
    fn can_reset_future() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new_expect(10, vec![1, 9], vec![2]),
            Attack::new_expect(13, vec![12], vec![4]),
        ]);

        let src = Attack::new_expect(30, vec![], vec![20, 28]);
        let mut mock_request: ComplementAttackRequest = src.into();

        let mut offset: RequestOffset = RequestOffset::new_default();
        offset = commit_and_assert(&mut mock_request, offset, &mut mock_track, 1, 2, true).unwrap();
        commit_and_assert(&mut mock_request, offset, &mut mock_track, 0, 1, false);
        mock_track.update_latest_nonpast(5);
        mock_track.reset_non_current(5);
        assert!(mock_track.latest_nonpast_commit().is_none());
    }

    #[test]
    fn can_partial_reset() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new_expect(10, vec![1, 9], vec![2]),
            Attack::new_expect(13, vec![12], vec![4]),
        ]);

        let src = Attack::new_expect(30, vec![], vec![20, 28]);
        let mut mock_request: ComplementAttackRequest = src.into();

        let mut offset: RequestOffset = RequestOffset::new_default();
        offset = commit_and_assert(&mut mock_request, offset, &mut mock_track, 1, 2, true).unwrap();
        commit_and_assert(&mut mock_request, offset, &mut mock_track, 0, 1, false);
        mock_track.update_latest_nonpast(18);
        mock_track.reset_non_current(18);
        assert!(mock_track.latest_nonpast_commit().is_some()); 
        mock_track.update_latest_nonpast(21);
        mock_track.reset_non_current(21);
        assert!(mock_track.latest_nonpast_commit().is_none());
    }

    #[test]
    fn does_not_reset() {
        let mut mock_track = EnemyTrack::new(vec![
            Attack::new_expect(10, vec![1, 9], vec![2]),
            Attack::new_expect(13, vec![12], vec![4]),
        ]);

        let src = Attack::new_expect(30, vec![], vec![20, 28]);
        let mut mock_request: ComplementAttackRequest = src.into();

        let mut offset: RequestOffset = RequestOffset::new_default();
        offset = commit_and_assert(&mut mock_request, offset, &mut mock_track, 1, 2, true).unwrap();
        commit_and_assert(&mut mock_request, offset, &mut mock_track, 0, 1, false);
        mock_track.update_latest_nonpast(18);
        assert!(mock_track.latest_nonpast_commit().is_some()); 
        for i in 19..=28{
            mock_track.update_latest_nonpast(i);
        }
        mock_track.reset_non_current(28);
        assert!(mock_track.latest_nonpast_commit().is_some());
    }
    
    #[test]
    fn can_match_all_futures() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new_expect(11, vec![1, 10], vec![]),
            Attack::new_expect(6, vec![5], vec![]),
            Attack::new_expect(13, vec![2, 4, 6], vec![]),
        ]);

        let mock_lead_track = Attack::new_expect(25, vec![], vec![10, 18]);
        let mut mock_request: ComplementAttackRequest = mock_lead_track.into();

        let can_meet = mock_track.possible_future_commits(&mut mock_request, 0);
        dbg!(&can_meet);
        assert_eq!(can_meet.len(), 3);
    }
}
