use bevy::prelude::*;
use physx::prelude::*;

const PX_PHYSICS_VERSION: u32 = physx::version(4, 1, 1);
pub struct PhysX {
    foundation: Foundation,
}

impl Default for PhysX {
    fn default() -> Self {
        let mut foundation = Foundation::new(PX_PHYSICS_VERSION);
        PhysX {
            foundation,
        }
    }
}

fn physx_init(
    mut physx: ResMut<PhysX>,
) {

}
