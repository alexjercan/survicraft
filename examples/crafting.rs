mod helpers;

use avian3d::prelude::*;
use bevy::prelude::*;
use clap::Parser;
use helpers::controller::{PlayerController, PlayerControllerPlugin};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-crafting")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft crafting", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    app.add_plugins(PhysicsPlugins::default().build());

    app.add_systems(Startup, setup);
    app.add_plugins(PlayerControllerPlugin { render: true });

    app.add_systems(Startup, setup_item_assets);
    app.add_systems(Startup, setup_recipe_assets);

    app.add_systems(Update, testing_spawn_item);

    app.add_plugins(ItemPlugin);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(60.0, 60.0, 00.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Directional Light"),
    ));

    commands.spawn((
        PlayerController,
        Transform::from_xyz(0.0, 3.0, 0.0),
        Name::new("Player Character"),
        Position(Vec3::new(0.0, 3.0, 0.0)),
        Rotation::default(),
        PhysicsCharacterBundle::default(),
        PhysicsCharacterInput::default(),
    ));

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

fn setup_item_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    debug!("Setting up item assets...");

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [item]
    //     id = "wood"
    //     name = "Wood"
    //     description = "A piece of wood."
    //     weight = 10.0
    //     scene="gltf/decoration/props/resource_lumber.gltf#Scene0"
    // [/item]
    // [item]
    //     id = "wood_box"
    //     name = "Wood Box"
    //     description = "A storage box made of wood."
    //     weight = 50.0
    //     scene="gltf/decoration/props/crate_A_big.gltf#Scene0"
    //     [modifier]
    //         type = storage
    //         capacity = 100
    //     [/modifier]
    // [/item]
    commands.insert_resource(ItemAssets::new(vec![
        ItemAsset {
            id: "wood".to_string(),
            name: "Wood".to_string(),
            description: "A piece of wood.".to_string(),
            weight: 10.0,
            scene: asset_server.load("gltf/decoration/props/resource_lumber.gltf#Scene0"),
            modifiers: vec![],
        },
        ItemAsset {
            id: "wood_box".to_string(),
            name: "Wood Box".to_string(),
            description: "A storage box made of wood.".to_string(),
            weight: 50.0,
            scene: asset_server.load("gltf/decoration/props/crate_A_big.gltf#Scene0"),
            modifiers: vec![ItemModifier::Storage { capacity: 100 }],
        },
    ]));
}

fn setup_recipe_assets(mut commands: Commands) {
    debug!("Setting up recipe assets...");

    // TODO: I want to load these from file, but for now, hardcode them
    // with some kind of cool syntax like:
    //
    // [recipe]
    //    [input]
    //        item = "wood"
    //        count = 5
    //    [/input]
    //    [output]
    //        item = "wood_box"
    //        count = 1
    //    [/output]
    // [/recipe]
    commands.insert_resource(RecipeAssets::new(vec![RecipeAsset {
        input: vec![RecipeItem {
            item_id: "wood".to_string(),
            count: 5,
        }],
        output: vec![RecipeItem {
            item_id: "wood_box".to_string(),
            count: 1,
        }],
    }]));
}

pub type ItemID = String;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct ItemAsset {
    pub id: ItemID,
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub scene: Handle<Scene>,
    pub modifiers: Vec<ItemModifier>,
}

#[derive(Debug, Clone)]
pub enum ItemModifier {
    Storage { capacity: u32 },
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ItemAssets {
    pub items: Vec<ItemAsset>,
}

impl ItemAssets {
    pub fn new(items: Vec<ItemAsset>) -> Self {
        Self { items }
    }

    pub fn get_item(&self, id: &ItemID) -> Option<&ItemAsset> {
        self.items.iter().find(|item| &item.id == id)
    }
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct RecipeAsset {
    pub input: Vec<RecipeItem>,
    pub output: Vec<RecipeItem>,
}

#[derive(Debug, Clone)]
pub struct RecipeItem {
    pub item_id: ItemID,
    pub count: u32,
}

#[derive(Resource, Clone, Default, Debug)]
pub struct RecipeAssets {
    pub recipes: Vec<RecipeAsset>,
}

impl RecipeAssets {
    pub fn new(recipes: Vec<RecipeAsset>) -> Self {
        Self { recipes }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Item;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_item_added);
    }
}

// if we have an Item component, we need to attach some more stuff to it:
// - RigidBody::Dynamic (I want the item to be affected by physics : more fun)
// - Collider (not sure what shape though, we need to fit the entire item)
fn on_item_added(trigger: Trigger<OnAdd, Item>, mut commands: Commands) {
    commands.entity(trigger.target()).insert((
        RigidBody::Dynamic,
        Collider::sphere(0.5), // TODO: change this to a better shape
        Friction::new(0.7),
        Restitution::new(0.3),
    ));
}

// testing system:
// - press Z will spawn 1 wood item at the player position
fn testing_spawn_item(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<PlayerController>>,
    item_assets: Res<ItemAssets>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        for transform in player_query.iter() {
            let Some(wood_item) = item_assets.get_item(&"wood".to_string()) else {
                error!("No wood item found");
                return;
            };

            commands.spawn((
                Name::new("Wood Item"),
                Item,
                Transform::from_translation(transform.translation + Vec3::new(0.0, 1.0, 0.0)),
                SceneRoot(wood_item.scene.clone()),
            ));
        }
    }
}
