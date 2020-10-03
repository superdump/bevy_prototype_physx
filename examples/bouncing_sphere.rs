use bevy::{
    input::system::exit_on_esc_system,
    prelude::*,
};

fn main() {
    App::build()
        .add_default_plugins()
        .add_startup_system(spawn_scene.system())
        .add_system(exit_on_esc_system.system())
        .run();
}

fn spawn_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let grey = materials.add(Color::hex("808080").unwrap().into());
    let teal = materials.add(Color::hex("008080").unwrap().into());

    let ground_plane = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    let sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.5,
        subdivisions: 5,
    }));

    commands.spawn(PbrComponents {
        material: grey,
        mesh: ground_plane,
        transform: Transform::from_non_uniform_scale(Vec3::new(10.0, 1.0, 10.0)),
        ..Default::default()
    })
    .spawn(PbrComponents {
        material: teal,
        mesh: sphere,
        transform: Transform::from_translation(Vec3::new(0.0, 7.5, 0.0)),
        ..Default::default()
    })
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
}
