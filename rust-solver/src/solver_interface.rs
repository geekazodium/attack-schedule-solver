use crate::attack::Attack;
use crate::enemy_track::EnemyTrack;
use crate::enemy_track::future_move_commit::FutureMoveCommit;
use crate::solver::Solver;
use crate::solver_interface::extern_enemy_track::ExternEnemyTrack;
use crate::solver_interface::godot_random::GodotRandom;
use godot::classes::INode;
use godot::classes::Node;
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
    solver: Solver,
}

#[godot_api]
impl INode for SolverInterface {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            solver: Solver::new(),
        }
    }
    fn physics_process(&mut self, _delta: f64) {
        let mut random = GodotRandom {};

        // godot_print!("creating req...");
        self.solver.try_create_new_request();
        // godot_print!("running solver...");
        if self.solver.solve(&mut random).is_some() {
        } else {
            godot_warn!("no attack in lead track, failed to create request");
        }
        self.solver.update_latest_nonpast();
        self.solver.tick();
    }
}

#[godot_api]
impl SolverInterface {
    #[func]
    fn add_track(&mut self, mut extern_track: Gd<ExternEnemyTrack>) {
        extern_track.bind_mut().parent_to_solver(self.to_gd());
        let attacks = extern_track
            .bind()
            .get_attacks()
            .iter_shared()
            .map(Attack::from)
            .collect();
        let track = EnemyTrack::new(attacks);
        let index = extern_track.bind().get_id();
        self.solver.add_track(index, track);
        godot_print!("added track: {}", index);
    }
    #[func]
    fn remove_track(&mut self, mut extern_track: Gd<ExternEnemyTrack>) {
        extern_track.bind_mut().unparent_from_solver();
        let index = extern_track.bind().get_id();
        self.solver.remove_track(index);
        godot_print!("removed track: {}", index);
    }
}

impl SolverInterface {
    pub fn time_now(&self) -> u64 {
        self.solver.current_tick()
    }
    pub fn commit_move_now(&mut self, id: NonZeroI64, index: usize) {
        let time_now = self.time_now();
        if !self.solver.all_tracks_actionable(time_now) {
            return;
        }
        if self
            .solver
            .get_track_mut(id)
            .commit_by_index(index, time_now)
        {
            self.solver.change_lead(id);
            godot_print!("sucess committed move");
        } else {
            // godot_warn!("failed to commit move");
        }
    }
    pub fn get_latest_nonpast_commit(&self, id: NonZeroI64) -> Option<&FutureMoveCommit> {
        self.solver.get_track(id).latest_nonpast_commit()
    }
    pub fn get_commit_on_this_frame(&self, id: NonZeroI64) -> Option<&FutureMoveCommit> {
        // subtract one because the end of the processing cycle for the solver increments and so checking
        // when you are outside of that cycle, you will need to effectively undo that.
        let time_now = self.time_now() - 1;
        self.get_latest_nonpast_commit(id)
            .filter(|v| v.get_start_frame() == time_now)
    }
}
