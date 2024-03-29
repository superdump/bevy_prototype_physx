use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_physx::*;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysXPlugin)
        .add_startup_system(spawn_scene.system())
        .add_system(exit_on_esc_system.system())
        .add_system_to_stage(bevy::app::CoreStage::Update, physx_control_character.system())
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

    let ground_plane = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.5,
        subdivisions: 5,
    }));
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.6 }));

    commands
        .spawn_bundle(PbrBundle {
            material: grey,
            mesh: ground_plane,
            transform: Transform::from_scale(Vec3::new(10.0, 1.0, 10.0)),
            ..Default::default()
        })
        .insert_bundle((
            PhysXMaterialDesc {
                static_friction: 0.5,
                dynamic_friction: 0.5,
                restitution: 0.6,
            },
            PhysXColliderDesc::Box(5.0, 0.5, 5.0),
            PhysXRigidBodyDesc::Static,
        ));
    commands
        .spawn_bundle(PbrBundle {
            material: purple,
            mesh: cube,
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::new(1.0, 1.75, 1.0),
                Quat::IDENTITY,
                Vec3::new(0.0, 0.6 * 1.75, 0.0),
            )),
            ..Default::default()
        })
        .insert_bundle((
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
        ));
    commands
        .spawn_bundle(LightBundle {
            transform: Transform::from_translation(Vec3::new(-5.0, 15.0, -5.0)),
            ..Default::default()
        });
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(-10.0, 0.5, -10.0),
                Vec3::new(5.0, 0.0, 5.0),
                Vec3::Y,
            )),
            ..Default::default()
        });

    for y in 0..5 {
        for z in 0..5 {
            for x in 0..5 {
                commands
                    .spawn_bundle(PbrBundle {
                        material: teal.clone(),
                        mesh: sphere.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            -2.5 + x as f32,
                            7.5 + y as f32,
                            -2.5 + z as f32,
                        )),
                        ..Default::default()
                    })
                    .insert_bundle((
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
    mut query: Query<(&mut PhysXController, &mut Transform)>,
) {
    for (mut controller, mut transform) in query.iter_mut() {
        let mut translation = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::W) {
            translation += Vec3::Z;
        }
        if keyboard_input.pressed(KeyCode::S) {
            translation -= Vec3::Z;
        }
        if keyboard_input.pressed(KeyCode::A) {
            translation += Vec3::X;
        }
        if keyboard_input.pressed(KeyCode::D) {
            translation -= Vec3::X;
        }
        if translation.length_squared() > 1e-5 {
            let translation = translation.normalize() * 5.0 * time.delta_seconds();
            let position = controller.get_position();
            let new_position = position + translation;
            controller.set_position(new_position);
            transform.translation = translation;
        }
    }
}
