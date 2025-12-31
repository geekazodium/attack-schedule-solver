use super::extern_enemy_attack::ExternEnemyAttack;
use crate::solver_interface::SolverInterface;
use godot::classes::Resource;
use godot::classes::class_macros::private::virtuals::Os::Array;
use godot::global::godot_print;
use godot::obj::Gd;
use godot::obj::WithBaseField;
use godot::prelude::Base;
use godot::prelude::GodotClass;
use godot::prelude::godot_api;
use std::num::NonZeroI64;

#[derive(GodotClass)]
#[class(base=Resource, init)]
pub(super) struct ExternEnemyTrack {
    base: Base<Resource>,
    #[export]
    attacks: Array<Gd<ExternEnemyAttack>>,
    solver_parent: Option<Gd<SolverInterface>>,
}

impl ExternEnemyTrack {
    pub(super) fn get_id(&self) -> NonZeroI64 {
        NonZeroI64::new(self.base().instance_id().to_i64())
            .expect("instance ID was somehow 0, panicking")
    }
    fn get_solver_parent(&self) -> Gd<SolverInterface> {
        self.solver_parent
            .clone()
            .or_else(|| {
                panic!(
                    "this {} is not parented to a solver, can't commit move",
                    self.base().instance_id()
                )
            })
            .unwrap()
    }
    pub fn parent_to_solver(&mut self, solver: Gd<SolverInterface>) {
        godot_print!(
            "instance({}) parenting to solver: {}",
            self.base().instance_id(),
            solver
        );
        self.solver_parent = Some(solver);
    }
    pub fn unparent_from_solver(&mut self) {
        godot_print!("instance({}) unparenting...", self.base().instance_id());
        self.solver_parent = None;
    }
}

#[godot_api]
impl ExternEnemyTrack {
    #[func]
    fn commit_move_now(&mut self, index: i64) {
        self.get_solver_parent().bind_mut().commit_move_now(
            self.get_id(),
            usize::try_from(index).expect("index of move out of usize range"),
        );
    }
    #[func]
    fn attack_index_on_this_frame(&self) -> i64 {
        self.get_solver_parent()
            .bind()
            .get_commit_on_this_frame(self.get_id())
            .map_or(-1, |v| {
                i64::try_from(v.get_index()).expect("usize out of i64 range")
            })
    }
    #[func]
    fn is_current_lead(&self) -> bool {
        self.get_solver_parent()
            .bind()
            .get_current_lead()
            .is_some_and(|v| v.eq(&self.get_id()))
    }
    #[func]
    fn attack_index_active_now(&self) -> i64 {
        self.get_solver_parent()
            .bind()
            .get_active_commit(self.get_id())
            .map_or(-1, |v| {
                i64::try_from(v.get_index()).expect("usize out of i64 range")
            })
    }
    #[func]
    fn attack_frame_active_now(&self) -> i64 {
        let get_solver_parent = self.get_solver_parent();
        let bind = get_solver_parent.bind();
        bind.get_active_commit(self.get_id()).map_or(i64::MIN, |v| {
            bind.time_now() as i64 - 1 - v.get_start_frame() as i64
        })
    }
}
