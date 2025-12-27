use godot::classes::Resource;
use godot::classes::class_macros::private::virtuals::Os::Array;
use godot::prelude::Base;
use godot::prelude::GodotClass;

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
