mod helpers;

use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use helpers::{controller::PlayerControllerPlugin, PlayerController};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-character")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft character controller", long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    dynamic: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExampleIsDynamic(bool);

fn main() {
    let cli = Cli::parse();

    let mut app = new_gui_app();

    app.add_plugins(PhysicsPlugins::default().build());

    app.add_systems(Startup, setup);
    app.add_plugins(PlayerControllerPlugin {
        dynamic: cli.dynamic,
        render: true,
    });
    app.insert_resource(ExampleIsDynamic(cli.dynamic));

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    is_dynamic: Res<ExampleIsDynamic>,
) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));

    if is_dynamic.0 {
        info!("Using dynamic physics character controller");
        commands.spawn((
            PlayerController,
            Transform::from_xyz(0.0, 3.0, 0.0),
            Name::new("Dynamic Player Character"),
            Position(Vec3::new(0.0, 3.0, 0.0)),
            Rotation::default(),
            PhysicsCharacterBundle::default(),
            CharacterInput::default(),
        ));
    } else {
        info!("Using kinematic physics character controller");
        commands.spawn((
            PlayerController,
            Transform::from_xyz(0.0, 3.0, 0.0),
            Name::new("Kinematic Player Character"),
            Position(Vec3::new(0.0, 3.0, 0.0)),
            Rotation::default(),
            KinematicCharacterBundle::default(),
            CharacterInput::default(),
        ));
    }

    const FLOOR_WIDTH: f32 = 20.0;
    const FLOOR_HEIGHT: f32 = 1.0;

    commands.spawn((
        Name::new("Floor"),
        Collider::cuboid(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH),
        RigidBody::Static,
        Position::new(Vec3::ZERO),
        Mesh3d(meshes.add(Cuboid::new(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
    ));

    commands.spawn((
        Name::new("Ramp"),
        Collider::cuboid(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH),
        RigidBody::Static,
        Position::new(Vec3::new(
            -5.0,
            FLOOR_HEIGHT * std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        )),
        Rotation::from(Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.0,
            0.0,
            -std::f32::consts::FRAC_PI_4,
        ))),
        Mesh3d(meshes.add(Cuboid::new(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
    ));

    for i in 0..5 {
        commands.spawn((
            Name::new(format!("Cube {i}")),
            Collider::cuboid(0.5, 0.5, 0.5),
            RigidBody::Dynamic,
            Position::new(Vec3::new(i as f32 - 2.0, 5.0 + i as f32, 0.0)),
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        ));
    }

    for i in 0..5 {
        commands.spawn((
            Name::new(format!("Sphere {i}")),
            Collider::sphere(0.5),
            RigidBody::Dynamic,
            Position::new(Vec3::new(i as f32 - 2.0, 5.0 + i as f32, 2.0)),
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.8))),
        ));
    }

    let wall_thickness = 1.0;
    let wall_height = 10.0;
    let wall_length = FLOOR_WIDTH * 2.0 + wall_thickness * 2.0;
    let wall_color = Color::srgb(0.4, 0.4, 0.4);
    let wall_material = materials.add(StandardMaterial {
        base_color: wall_color,
        ..default()
    });
    commands.spawn((
        Name::new("Wall +X"),
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            FLOOR_WIDTH / 2.0 + wall_thickness,
            wall_height / 2.0,
            0.0,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, wall_length))),
        MeshMaterial3d(wall_material.clone()),
    ));
    commands.spawn((
        Name::new("Wall -X"),
        Collider::cuboid(wall_thickness, wall_height, wall_length),
        RigidBody::Static,
        Position::new(Vec3::new(
            -FLOOR_WIDTH / 2.0 - wall_thickness,
            wall_height / 2.0,
            0.0,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, wall_length))),
        MeshMaterial3d(wall_material.clone()),
    ));
    commands.spawn((
        Name::new("Wall +Z"),
        Collider::cuboid(wall_length, wall_height, wall_thickness),
        RigidBody::Static,
        Position::new(Vec3::new(
            0.0,
            wall_height / 2.0,
            FLOOR_WIDTH / 2.0 + wall_thickness,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_length, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material.clone()),
    ));
    commands.spawn((
        Name::new("Wall -Z"),
        Collider::cuboid(wall_length, wall_height, wall_thickness),
        RigidBody::Static,
        Position::new(Vec3::new(
            0.0,
            wall_height / 2.0,
            -FLOOR_WIDTH / 2.0 - wall_thickness,
        )),
        Mesh3d(meshes.add(Cuboid::new(wall_length, wall_height, wall_thickness))),
        MeshMaterial3d(wall_material),
    ));
}
