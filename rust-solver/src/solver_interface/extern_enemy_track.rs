use crate::solver_interface::SolverInterface;

use super::extern_enemy_attack::ExternEnemyAttack;
use godot::classes::Resource;
use godot::classes::class_macros::private::virtuals::Os::Array;
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
            .expect("this is not parented to a solver, can't commit move")
    }
    pub fn parent_to_solver(&mut self, solver: Gd<SolverInterface>) {
        self.solver_parent = Some(solver);
    }
}

#[godot_api]
impl ExternEnemyTrack {
    #[func]
    fn commit_move_now(&mut self, index: i64) {
        self.get_solver_parent()
            .bind_mut()
            .commit_move_now(self.get_id(), index as usize);
    }
}
