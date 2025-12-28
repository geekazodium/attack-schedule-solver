use crate::attack::Attack;
use godot::classes::Resource;
use godot::classes::class_macros::private::virtuals::Os::Array;
use godot::obj::Gd;
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

impl ExternEnemyAttack {
    pub fn get_frames_as_vec(&self) -> Vec<u64> {
        self.frames.iter_shared().map(u64::from).collect()
    }
    pub fn get_requests_as_vec(&self) -> Vec<u64> {
        self.requests.iter_shared().map(u64::from).collect()
    }
}

impl From<Gd<ExternEnemyAttack>> for Attack {
    fn from(attack: Gd<ExternEnemyAttack>) -> Self {
        Self::new(
            u64::from(attack.bind().get_duration()),
            attack.bind().get_frames_as_vec(),
            attack.bind().get_requests_as_vec(),
        )
    }
}
