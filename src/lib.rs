use bevy::prelude::*;
use physx::traits::Releasable;
use std::sync::{Arc, RwLock};

pub struct PhysXPlugin;

impl Plugin for PhysXPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PhysX>()
            .add_system_to_stage_front(
                bevy::app::stage::PRE_UPDATE,
                physx_create_body_material_collider.system(),
            )
            .add_system_to_stage(bevy::app::stage::UPDATE, physx_step_simulation.system())
            .add_system_to_stage(
                bevy::app::stage::POST_UPDATE,
                physx_sync_transforms.system(),
            );
    }
}

const PX_PHYSICS_VERSION: u32 = physx::version(4, 1, 1);
pub struct PhysX {
    foundation: Arc<RwLock<physx::prelude::Foundation>>,
    pub physics: Option<Arc<RwLock<physx::prelude::Physics>>>,
    pub scene: Box<physx::prelude::Scene>,
}

impl Default for PhysX {
    fn default() -> Self {
        let foundation = Arc::new(RwLock::new(physx::prelude::Foundation::new(
            PX_PHYSICS_VERSION,
        )));

        let physics = Some(Arc::new(RwLock::new(
            physx::prelude::PhysicsBuilder::default()
                .load_extensions(false)
                .build(&mut *foundation.write().unwrap()),
        )));

        let scene = if let Some(ref physics) = physics {
            physics.write().unwrap().create_scene(
                physx::prelude::SceneBuilder::default()
                    .set_gravity(Vec3::new(0.0, -9.81, 0.0))
                    .set_simulation_threading(physx::prelude::SimulationThreadType::Dedicated(1)),
            )
        } else {
            panic!("FAIL");
        };
        PhysX {
            foundation,
            physics,
            scene,
        }
    }
}

impl Drop for PhysX {
    fn drop(&mut self) {
        unsafe {
            self.scene.release();
        }

        {
            let physics = self.physics.take();
            drop(physics);
        }

        self.foundation.write().unwrap().release();
    }
}

pub struct PhysXMaterialDesc {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
}

pub type PhysXColliderDesc = physx::prelude::ColliderDesc;

pub enum PhysXRigidBodyDesc {
    Static,
    Dynamic { density: f32, angular_damping: f32 },
}

pub struct PhysXStaticRigidBodyHandle(pub physx::prelude::BodyHandle);
pub struct PhysXDynamicRigidBodyHandle(pub physx::prelude::BodyHandle);

fn physx_create_body_material_collider(
    mut commands: Commands,
    mut physx: ResMut<PhysX>,
    mut query: Query<(
        Entity,
        &PhysXMaterialDesc,
        &PhysXColliderDesc,
        &PhysXRigidBodyDesc,
        &Transform,
    )>,
) {
    let mut physics = physx.physics.as_mut().expect("Failed to get Physics").clone();
    let mut physics_write = physics.write().expect("Failed to get Physics write lock");
    for (entity, material_desc, collider_desc, body_desc, transform) in &mut query.iter() {
        let material = physics_write.create_material(
            material_desc.static_friction,
            material_desc.dynamic_friction,
            material_desc.restitution,
        );
        let geometry = physx::prelude::PhysicsGeometry::from(collider_desc);
        match body_desc {
            PhysXRigidBodyDesc::Static => {
                let (scale, rotation, translation) =
                    transform.value().to_scale_rotation_translation();
                // FIXME - are non-uniform scales disallowed???
                println!(
                    "scale: {:#?}, rotation: {:#?}, translation: {:#?}",
                    scale, rotation, translation
                );
                println!(
                    "rotation translation: {:#?}",
                    Mat4::from_rotation_translation(rotation, translation)
                );
                println!("scale: {:#?}", Mat4::from_scale(scale));
                let actor = unsafe {
                    physics_write.create_static(
                        Mat4::identity(),  //Mat4::from_rotation_translation(rotation, translation),
                        geometry.as_raw(), // todo: this should take the PhysicsGeometry straight.
                        material,
                        Mat4::identity(), //Mat4::from_scale(scale),
                    )
                };
                commands.insert_one(
                    entity,
                    PhysXStaticRigidBodyHandle(physx.scene.add_actor(actor)),
                );
            }
            PhysXRigidBodyDesc::Dynamic {
                density,
                angular_damping,
            } => {
                let (scale, rotation, translation) =
                    transform.value().to_scale_rotation_translation();
                let mut actor = unsafe {
                    physics_write.create_dynamic(
                        Mat4::from_rotation_translation(rotation, translation),
                        geometry.as_raw(), // todo: this should take the PhysicsGeometry straight.
                        material,
                        *density,
                        Mat4::from_scale(scale),
                    )
                };
                actor.set_angular_damping(*angular_damping);
                commands.insert_one(
                    entity,
                    PhysXDynamicRigidBodyHandle(physx.scene.add_dynamic(actor)),
                );
            }
        }
        commands.remove_one::<PhysXMaterialDesc>(entity);
        commands.remove_one::<PhysXColliderDesc>(entity);
        commands.remove_one::<PhysXRigidBodyDesc>(entity);
    }
}

fn physx_step_simulation(time: Res<Time>, mut physx: ResMut<PhysX>) {
    if time.delta_seconds > 0.0 {
        physx.scene.simulate(time.delta_seconds);
        physx
            .scene
            .fetch_results(true)
            .expect("PhysX simulation failed");
    }
}

fn physx_sync_transforms(
    physx: Res<PhysX>,
    body_handle: &PhysXDynamicRigidBodyHandle,
    mut transform: Mut<Transform>,
) {
    *transform = Transform::new(
        unsafe { physx.scene.get_rigid_actor_unchecked(&body_handle.0) }.get_global_pose(),
    );
}
