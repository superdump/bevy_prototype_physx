use bevy::prelude::*;
use physx::traits::Releasable;

pub struct PhysXPlugin;

impl Plugin for PhysXPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PhysX>()
            .add_system_to_stage(
                bevy::app::CoreStage::PreUpdate,
                physx_create_body_material_collider.system(),
            )
            // This is intentionally added after but to the front to come before
            // the general body/material/collider creation system
            .add_system_to_stage(
                bevy::app::CoreStage::PreUpdate,
                physx_create_character_controller.system(),
            )
            .add_system_to_stage(bevy::app::CoreStage::Update, physx_step_simulation.system())
            .add_system_to_stage(bevy::app::CoreStage::Update, physx_sync_transforms.system());
    }
}

const PX_PHYSICS_VERSION: u32 = physx::version(4, 1, 1);

pub struct PhysX {
    foundation: physx::prelude::Foundation,
    pub physics: std::mem::ManuallyDrop<physx::prelude::Physics>,
    pub scene: Box<physx::prelude::Scene>,
}

impl Default for PhysX {
    fn default() -> Self {
        let mut foundation = physx::prelude::Foundation::new(PX_PHYSICS_VERSION);

        let mut physics = std::mem::ManuallyDrop::new(
            physx::prelude::PhysicsBuilder::default()
                .load_extensions(false)
                .build(&mut foundation),
        );

        let scene = physics.create_scene(
            physx::prelude::SceneBuilder::default()
                .use_controller_manager(true, true)
                .set_simulation_threading(physx::prelude::SimulationThreadType::Dedicated(1)),
        );
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
            std::mem::ManuallyDrop::drop(&mut self.physics);
        }

        self.foundation.release();
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

pub struct PhysXCapsuleControllerDesc {
    pub height: f32,
    pub radius: f32,
    pub step_offset: f32,
}

impl Default for PhysXCapsuleControllerDesc {
    fn default() -> Self {
        PhysXCapsuleControllerDesc {
            height: 1.75,
            radius: 0.3,
            step_offset: 0.4,
        }
    }
}

pub type PhysXController = physx::prelude::Controller;

fn physx_create_character_controller(
    mut commands: Commands,
    mut physx: ResMut<PhysX>,
    query_controller: Query<(
        Entity,
        &PhysXCapsuleControllerDesc,
        &PhysXMaterialDesc,
        &Transform,
    )>,
) {
    for (entity, physx_capsule_controller_desc, material_desc, transform) in query_controller.iter()
    {
        let material = physx.physics.create_material(
            material_desc.static_friction,
            material_desc.dynamic_friction,
            material_desc.restitution,
        );
        let mut capsule_controller = physx
            .scene
            .add_capsule_controller(
                &physx::controller::CapsuleControllerDesc::new(
                    physx_capsule_controller_desc.height,
                    physx_capsule_controller_desc.radius,
                    physx_capsule_controller_desc.step_offset,
                    material,
                )
                .expect("Failed to create capsule controller"),
            )
            .expect("Failed to add capsule controller to scene");
        capsule_controller.set_position(transform.translation);
        commands
            .entity(entity)
            .insert(capsule_controller)
            .remove::<PhysXCapsuleControllerDesc>()
            .remove::<PhysXMaterialDesc>();
    }
}

fn physx_create_body_material_collider(
    mut commands: Commands,
    mut physx: ResMut<PhysX>,
    query: Query<(
        Entity,
        &PhysXMaterialDesc,
        &PhysXColliderDesc,
        &PhysXRigidBodyDesc,
        &Transform,
    )>,
) {
    for (entity, material_desc, collider_desc, body_desc, transform) in query.iter() {
        let material = physx.physics.create_material(
            material_desc.static_friction,
            material_desc.dynamic_friction,
            material_desc.restitution,
        );
        create_body_collider(
            entity,
            collider_desc,
            body_desc,
            material,
            transform,
            &mut physx,
            &mut commands,
        );
        commands
            .entity(entity)
            .remove::<PhysXMaterialDesc>()
            .remove::<PhysXColliderDesc>()
            .remove::<PhysXRigidBodyDesc>();
    }
}

fn create_body_collider(
    entity: Entity,
    collider_desc: &PhysXColliderDesc,
    body_desc: &PhysXRigidBodyDesc,
    material: *mut physx_sys::PxMaterial,
    transform: &Transform,
    physx: &mut PhysX,
    commands: &mut Commands,
) {
    let geometry = physx::prelude::PhysicsGeometry::from(collider_desc);
    match body_desc {
        PhysXRigidBodyDesc::Static => {
            // let (scale, rotation, translation) = transform.value().to_scale_rotation_translation();
            // FIXME - are non-uniform scales disallowed???
            let actor = unsafe {
                physx.physics.create_static(
                    Mat4::IDENTITY,  //Mat4::from_rotation_translation(rotation, translation),
                    geometry.as_raw(), // todo: this should take the PhysicsGeometry straight.
                    material,
                    Mat4::IDENTITY, //Mat4::from_scale(scale),
                )
            };
            commands
                .entity(entity)
                .insert(PhysXStaticRigidBodyHandle(physx.scene.add_actor(actor)));
        }
        PhysXRigidBodyDesc::Dynamic {
            density,
            angular_damping,
        } => {
            let mut actor = unsafe {
                physx.physics.create_dynamic(
                    Mat4::from_rotation_translation(transform.rotation, transform.translation),
                    geometry.as_raw(), // todo: this should take the PhysicsGeometry straight.
                    material,
                    *density,
                    Mat4::from_scale(transform.scale),
                )
            };
            actor.set_angular_damping(*angular_damping);
            commands
                .entity(entity)
                .insert(PhysXDynamicRigidBodyHandle(physx.scene.add_dynamic(actor)));
        }
    }
}

fn physx_step_simulation(time: Res<Time>, mut physx: ResMut<PhysX>) {
    if time.delta_seconds() > 0.0 {
        physx.scene.simulate(time.delta_seconds());
        physx
            .scene
            .fetch_results(true)
            .expect("PhysX simulation failed");
    }
}

fn physx_sync_transforms(
    physx: ResMut<PhysX>,
    mut query_transforms: Query<(&PhysXDynamicRigidBodyHandle, &mut Transform)>,
) {
    // FIXME - this only works for bodies on top-level entities
    for (body_handle, mut transform) in query_transforms.iter_mut() {
        *transform = Transform::from_matrix(
            unsafe { physx.scene.get_rigid_actor_unchecked(&body_handle.0) }.get_global_pose(),
        );
    }
}
