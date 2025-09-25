mod helpers;

use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use helpers::controller::{PlayerController, PlayerControllerPlugin};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-terrain")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft character controller on terrain", long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    dynamic: bool,
}

const DISCOVER_RADIUS: u32 = 1;

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

    app.add_plugins(TerrainPlugin::default());
    app.add_plugins(TerrainRenderPlugin::default());
    app.add_systems(
        Update,
        (spawn_player_on_space, discover_terrain_at_player_position),
    );

    app.run();
}

fn setup(mut ev_discover: EventWriter<TileDiscoverEvent>, mut commands: Commands) {
    commands.insert_resource(TerrainAssets::new(vec![
        TileAsset {
            id: "deep_water".to_string(),
            name: "Deep Water".to_string(),
            generation: TileGeneration {
                elevation_min: None,
                elevation_max: Some(0.25),
            },
        },
        TileAsset {
            id: "shallow_water".to_string(),
            name: "Shallow Water".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.25),
                elevation_max: Some(0.5),
            },
        },
        TileAsset {
            id: "sand".to_string(),
            name: "Sand".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.5),
                elevation_max: Some(0.55),
            },
        },
        TileAsset {
            id: "grass".to_string(),
            name: "Grass".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.55),
                elevation_max: Some(0.75),
            },
        },
        TileAsset {
            id: "hills".to_string(),
            name: "Hills".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.75),
                elevation_max: Some(0.9),
            },
        },
        TileAsset {
            id: "mountain".to_string(),
            name: "Mountain".to_string(),
            generation: TileGeneration {
                elevation_min: Some(0.9),
                elevation_max: None,
            },
        },
    ]));

    ev_discover.write(TileDiscoverEvent::new(Vec2::ZERO, DISCOVER_RADIUS));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));
}

fn spawn_player_on_space(
    mut commands: Commands,
    is_dynamic: Res<ExampleIsDynamic>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyP) {
        return;
    }

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
}

fn discover_terrain_at_player_position(
    q_player: Query<&Position, With<PlayerController>>,
    mut ev_discover: EventWriter<TileDiscoverEvent>,
) {
    for player_pos in q_player.iter() {
        let tile_pos = Vec2::new(player_pos.x, player_pos.z);
        ev_discover.write(TileDiscoverEvent::new(tile_pos, DISCOVER_RADIUS));
    }
}
