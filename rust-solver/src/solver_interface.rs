use crate::attack::Attack;
use crate::enemy_track::EnemyTrack;
use crate::solver::Solver;
use crate::solver_interface::extern_enemy_track::ExternEnemyTrack;
use crate::solver_interface::godot_random::GodotRandom;
use godot::classes::INode;
use godot::classes::Node;
use godot::classes::class_macros::private::virtuals::Os::Array;
use godot::global::godot_print;
use godot::global::godot_warn;
use godot::obj::Gd;
use godot::obj::WithBaseField;
use godot::prelude::Base;
use godot::prelude::GodotClass;
use godot::prelude::godot_api;
use std::num::NonZeroI64;

mod extern_enemy_attack;
mod extern_enemy_track;
mod godot_random;

#[derive(GodotClass)]
#[class(base=Node)]
struct SolverInterface {
    base: Base<Node>,
    #[export]
    tracks: Array<Gd<ExternEnemyTrack>>,
    solver: Solver,
}

#[godot_api]
impl INode for SolverInterface {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            tracks: Array::new(),
            solver: Solver::new(),
        }
    }
    fn ready(&mut self) {
        for mut t in self.tracks.clone().iter_shared() {
            t.bind_mut().parent_to_solver(self.to_gd());
            self.add_track(t);
        }

        if !self.tracks.is_empty() {
            self.solver
                .change_lead(self.tracks.get(0).unwrap().bind_mut().get_id());
        }
    }
    fn physics_process(&mut self, _delta: f64) {
        let mut random = GodotRandom {};

        godot_print!("creating req...");
        self.solver.try_create_new_request();
        godot_print!("running solver...");
        if self.solver.solve(&mut random).is_some() {
        } else {
            godot_warn!("no attack in lead track, failed to create request");
        }
        self.solver.tick();
    }
}

#[godot_api]
impl SolverInterface {
    #[func]
    fn add_track(&mut self, extern_track: Gd<ExternEnemyTrack>) {
        let attacks = extern_track
            .bind()
            .get_attacks()
            .iter_shared()
            .map(|attack| {
                Attack::new(
                    u64::from(attack.bind().get_duration()),
                    attack
                        .bind()
                        .get_frames()
                        .iter_shared()
                        .map(|v| u64::from(v))
                        .collect(),
                    attack
                        .bind()
                        .get_requests()
                        .iter_shared()
                        .map(|v| u64::from(v))
                        .collect(),
                )
            })
            .collect();
        let track = EnemyTrack::new(attacks);
        self.solver.add_track(extern_track.bind().get_id(), track);
    }
    #[func]
    fn remove_track(&mut self, track: Gd<ExternEnemyTrack>) {
        self.solver.remove_track(track.bind().get_id());
    }
}

impl SolverInterface {
    pub fn commit_move_now(&mut self, id: NonZeroI64, index: usize) {
        let time_now = self.solver.current_tick();
        godot_print!("current time: {}", time_now);
        if self
            .solver
            .get_track_mut(id)
            .commit_by_index(index, time_now)
        {
            self.solver.change_lead(id);
            godot_print!("sucess committed move");
        } else {
            godot_warn!("failed to commit move");
        }
    }
}
