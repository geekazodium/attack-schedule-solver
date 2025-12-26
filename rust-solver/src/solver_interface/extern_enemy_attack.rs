use godot::classes::Resource;
use godot::classes::class_macros::private::virtuals::Os::Array;
use godot::obj::WithBaseField;
use godot::prelude::Base;
use godot::prelude::GodotClass;
use std::num::NonZeroI64;

#[derive(GodotClass)]
#[class(base=Resource, init)]
pub(super) struct ExternEnemyAttack {
    base: Base<Resource>,
    #[export]
    frames: Array<u32>,
    #[export]
    requests: Array<u32>,
    #[export]
    duration: u32,
}

impl ExternEnemyAttack {
    pub(super) fn get_id(&self) -> NonZeroI64 {
        NonZeroI64::new(self.base().instance_id().to_i64())
            .expect("instance ID was somehow 0, panicking")
    }
}
