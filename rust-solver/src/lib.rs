use std::collections::HashMap;

use godot::classes::INode;
use godot::classes::Node;
use godot::classes::Resource;
use godot::classes::class_macros::private::virtuals::Os::Array;
use godot::global::godot_warn;
use godot::obj::Gd;
use godot::obj::WithBaseField;
use godot::prelude::Base;
use godot::prelude::godot_api;

use godot::init::ExtensionLibrary;
use godot::init::gdextension;
use godot::prelude::GodotClass;

use crate::enemy_track::EnemyTrack;
use crate::solver::Solver;
use crate::solver::SolverRandomState;

pub(crate) mod attack;
mod default_hasher_random;
pub(crate) mod enemy_track;
mod solver;

struct AttackSchedulerExtension;

#[gdextension]
unsafe impl ExtensionLibrary for AttackSchedulerExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct SolverInterface {
    base: Base<Node>,
    #[export]
    tracks: Array<Gd<ExternEnemyTrack>>,
    solver: Solver,
    solver_indexes: HashMap<i64, usize>,
}

#[godot_api]
impl INode for SolverInterface {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            tracks: Array::new(),
            solver: Solver::new(EnemyTrack::new(vec![])),
            solver_indexes: HashMap::new(),
        }
    }
    fn process(&mut self, _delta: f64) {
        if self.solver.solve(&mut GodotRandom {}).is_some() {
        } else {
            godot_warn!("no attack in lead track, failed to create request");
        }
    }
}

struct GodotRandom {}

impl SolverRandomState for GodotRandom {
    fn next_in_range(&mut self, max: usize) -> usize {
        godot::global::randi_range(0, max as i64) as usize
    }
}

#[derive(GodotClass)]
#[class(base=Resource, init)]
struct ExternEnemyTrack {
    base: Base<Resource>,
    #[export]
    tracks: i64,
}

impl ExternEnemyTrack {
    fn get_id(&self) -> i64 {
        self.base().instance_id().to_i64()
    }
}

#[cfg(test)]
mod tests {
    use crate::attack::Attack;
    use crate::default_hasher_random::HashRandom;
    use crate::enemy_track::EnemyTrack;
    use crate::solver::Solver;
    use crate::solver::SolverRandomState;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn test_hasher_works() {
        let mut rand1 = HashRandom::new(0);
        let mut rand2 = HashRandom::new(0);
        for _ in 0..10000 {
            let random_v = rand1.next_in_range(3);
            assert!(random_v < 3);
            assert_eq!(random_v, rand2.next_in_range(3));
        }
    }

    #[test]
    fn solve_select_move_success() {
        let mut random = HashRandom::new(120);
        let mut lead_track = EnemyTrack::new(vec![
            Attack::new(30, vec![15, 25], vec![20]),
            Attack::new(40, vec![38], vec![20, 30]),
            Attack::new(80, vec![38], vec![20, 30, 60]),
        ]);

        assert!(lead_track.commit_by_index(2));

        let mut solver = Solver::new(lead_track);

        for _ in 0..2 {
            solver.add_track(EnemyTrack::new(vec![
                Attack::new(30, vec![15, 25], vec![20]),
                Attack::new(40, vec![10], vec![20, 30]),
                Attack::new(40, vec![20], vec![30]),
                Attack::new(40, vec![30], vec![20]),
            ]));
        }

        let now = Instant::now();
        let request = solver.solve(&mut random);
        let elapsed = Instant::now() - now;
        dbg!(elapsed);
        //cursed performance target check
        assert!(elapsed < Duration::from_millis(8));
        // dbg!(&solver);
        dbg!(request);
        dbg!(solver);
    }
}
