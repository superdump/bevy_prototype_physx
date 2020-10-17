use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_physx::*;

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_plugin(PhysXPlugin)
        .add_startup_system(spawn_scene.system())
        .add_system(exit_on_esc_system.system())
        .add_system_to_stage_front(bevy::app::stage::UPDATE, physx_control_character.system())
        .run();
}

fn spawn_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let grey = materials.add(Color::hex("808080").unwrap().into());
    let teal = materials.add(Color::hex("008080").unwrap().into());
    let purple = materials.add(Color::hex("800080").unwrap().into());

    let ground_plane = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    let sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.5,
        subdivisions: 5,
    }));
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.3 }));

    commands
        .spawn(PbrComponents {
            material: grey,
            mesh: ground_plane,
            transform: Transform::from_non_uniform_scale(Vec3::new(10.0, 1.0, 10.0)),
            ..Default::default()
        })
        .with_bundle((
            PhysXMaterialDesc {
                static_friction: 0.5,
                dynamic_friction: 0.5,
                restitution: 0.6,
            },
            PhysXColliderDesc::Box(5.0, 0.5, 5.0),
            PhysXRigidBodyDesc::Static,
        ))
        .spawn(PbrComponents {
            material: purple,
            mesh: cube,
            transform: Transform::new(Mat4::from_scale_rotation_translation(
                Vec3::new(1.0, 1.75, 1.0),
                Quat::identity(),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with_bundle((
            PhysXMaterialDesc {
                static_friction: 0.5,
                dynamic_friction: 0.5,
                restitution: 0.6,
            },
            PhysXCapsuleControllerDesc {
                height: 1.75,
                radius: 0.3,
                step_offset: 0.5,
            },
        ))
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(-5.0, 15.0, -5.0)),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: Transform::new(Mat4::face_toward(
                Vec3::new(-10.0, 10.0, -10.0),
                Vec3::new(5.0, 0.0, 5.0),
                Vec3::unit_y(),
            )),
            ..Default::default()
        });

    for y in 0..5 {
        for z in 0..5 {
            for x in 0..5 {
                commands
                    .spawn(PbrComponents {
                        material: teal,
                        mesh: sphere,
                        transform: Transform::from_translation(Vec3::new(
                            -2.5 + x as f32,
                            7.5 + y as f32,
                            -2.5 + z as f32,
                        )),
                        ..Default::default()
                    })
                    .with_bundle((
                        PhysXMaterialDesc {
                            static_friction: 0.5,
                            dynamic_friction: 0.5,
                            restitution: 0.6,
                        },
                        PhysXColliderDesc::Sphere(0.5),
                        PhysXRigidBodyDesc::Dynamic {
                            density: 10.0,
                            angular_damping: 0.5,
                        },
                    ));
            }
        }
    }
}

fn physx_control_character(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut _physx: ResMut<PhysX>, // For synchronization
    mut controller: Mut<PhysXController>,
    mut transform: Mut<Transform>,
) {
    let mut translation = Vec3::zero();
    if keyboard_input.pressed(KeyCode::W) {
        translation += Vec3::unit_z();
    }
    if keyboard_input.pressed(KeyCode::S) {
        translation -= Vec3::unit_z();
    }
    if keyboard_input.pressed(KeyCode::A) {
        translation += Vec3::unit_x();
    }
    if keyboard_input.pressed(KeyCode::D) {
        translation -= Vec3::unit_x();
    }
    if translation.length_squared() > 1e-5 {
        let translation = translation.normalize() * 5.0 * time.delta_seconds;
        let position = controller.get_position();
        let new_position = position + translation;
        controller.set_position(new_position);
        transform.translate(translation);
    }
}
