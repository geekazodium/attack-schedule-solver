use crate::attack::Attack;

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
    pub fn can_meet_request(
        &self,
        request_vec: &Vec<u64>,
        request_offset: usize,
        claim_end: u64,
    ) -> Vec<&Attack> {
        if let Some(request_frame) = request_vec.get(request_offset).map(|u| *u) {
            self.attacks
                .iter()
                .filter_map(|attack| {
                    attack
                        .get_start_frame(request_frame)
                        .map(|start_frame| (attack, start_frame))
                })
                .filter(|pair| {
                    dbg!(&pair.0);
                    let mut tmp_iter = request_vec[(request_offset + 1)..].iter();
                    dbg!(&tmp_iter);
                    let active_frames_iter = pair.0.get_active_frames(pair.1);
                    for active in active_frames_iter.skip(1) {
                        dbg!(active);
                        if active >= claim_end {
                            return true;
                        }
                        while let Some(next_request_frame) = tmp_iter.next() {
                            if *next_request_frame > active {
                                return false;
                            }
                            if *next_request_frame == active {
                                break;
                            }
                        }
                    }
                    return true;
                })
                .map(|x| x.0)
                .collect()
        } else {
            vec![]
        }
    }
    pub fn commit() {}
}

#[cfg(test)]
mod enemy_track_tests {
    use super::*;

    #[test]
    fn can_meet_request_test() {
        let mock_track = EnemyTrack::new(vec![
            Attack::new(10, vec![8, 16], vec![2]),
            Attack::new(20, vec![12], vec![4]),
        ]);
        assert_eq!(
            mock_track.can_meet_request(&mut vec![20, 28], 0, 100).len(),
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
                .can_meet_request(&mut vec![19, 28], 0, 100)
                .is_empty()
        );
    }
}
