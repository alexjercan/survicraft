mod helpers;

use avian3d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use clap::Parser;
use helpers::{
    wasd::{WASDCameraControllerBundle, WASDCameraControllerPlugin},
    PlayerController,
};
use survicraft::prelude::*;

#[derive(Parser)]
#[command(name = "survicraft-crafting")]
#[command(version = "0.1")]
#[command(about = "Example for the survicraft crafting", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(PhysicsDebugPlugin::default());

    app.add_systems(Startup, setup);
    app.add_plugins(WASDCameraControllerPlugin);

    app.add_systems(Startup, setup_item_assets);
    app.add_systems(Startup, setup_recipe_assets);

    app.add_systems(Update, testing_spawn_item);
    app.add_systems(Update, test_f_input);

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
        Name::new("Player Character"),
        Camera3d::default(),
        PlayerController,
        WASDCameraControllerBundle::default(),
        Crafter,
        CrafterInput::default(),
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
            offset: Some(Vec3::new(0.0, -0.1, 0.0)),
            scale: Some(Vec3::splat(1.0)),
            collider: Some(Collider::cuboid(0.75, 0.2, 0.4)),
            modifiers: vec![],
        },
        ItemAsset {
            id: "wood_box".to_string(),
            name: "Wood Box".to_string(),
            description: "A storage box made of wood.".to_string(),
            weight: 50.0,
            scene: asset_server.load("gltf/decoration/props/crate_A_big.gltf#Scene0"),
            offset: Some(Vec3::new(0.0, -0.2, 0.0)),
            scale: Some(Vec3::splat(2.0)),
            collider: Some(Collider::cuboid(0.5, 0.5, 0.5)),
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
                Name::new("Item"),
                Item(wood_item.id.clone()),
                Transform::from_translation(transform.translation + Vec3::new(0.0, 1.0, 0.0)),
            ));
        }
    }
}

// testing system:
// press F will set the craft input to true for 1 frame
fn test_f_input(mut query: Query<&mut CrafterInput>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for mut input in &mut query {
        input.craft = keyboard_input.just_pressed(KeyCode::KeyF);
    }
}

pub type ItemID = String;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct ItemAsset {
    pub id: ItemID,
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub scene: Handle<Scene>,
    pub offset: Option<Vec3>,
    pub scale: Option<Vec3>,
    pub collider: Option<Collider>,
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

impl RecipeAsset {
    pub fn can_craft(&self, available: &HashMap<ItemID, u32>) -> bool {
        for item in &self.input {
            let count = available.get(&item.item_id).cloned().unwrap_or(0);
            if count < item.count {
                return false;
            }
        }
        true
    }
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

/// Item component to mark an entity as an item. This will add a RigidBody and a Collider to the
/// entity, as well as a Scene to represent the item visually.
///
/// This component should be added to entities that represent items dropped in the world and can be
/// picked up by players, or used in crafting recipes.
#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct Item(pub ItemID);

/// Marker component to mark an entity as a crafter. This can be used to identify entities that can
/// craft items. The entity will cast a ray to find the items in front of it, and will use the
/// recipes to craft new items.
#[derive(Component, Debug, Clone)]
#[require(Transform)]
pub struct Crafter;

/// A component that holds the input state for a crafter controller.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct CrafterInput {
    pub craft: bool,
}

/// Component used to store the items that are in front of the crafter, and can be used for
/// crafting.
#[derive(Component, Debug, Clone, Default, Deref, DerefMut, Reflect)]
pub struct CrafterCache(pub Vec<Entity>);

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>()
            .register_type::<CrafterCache>()
            .register_type::<CrafterInput>();

        app.add_observer(on_item_added);
        app.add_observer(on_crafter_added);

        app.add_systems(Update, update_crafter_cache);
        app.add_systems(Update, handle_crater_input);

        // Debug stuff
        app.add_systems(Update, gizmos_crafter_cache);
    }
}

// if we have an Item component, we need to attach some more stuff to it:
// - RigidBody::Dynamic (I want the item to be affected by physics : more fun)
// - Collider (not sure what shape though, we need to fit the entire item)
fn on_item_added(
    trigger: Trigger<OnAdd, Item>,
    mut commands: Commands,
    q_items: Query<&Item>,
    items: Res<ItemAssets>,
) {
    let entity = trigger.target();
    let Ok(Item(item)) = q_items.get(entity) else {
        error!("No Item component found on entity {:?}", trigger.target());
        return;
    };

    let Some(item) = items.get_item(item) else {
        error!("No item asset found for id {}", item);
        return;
    };

    commands
        .entity(entity)
        .insert((
            Name::new(format!("Item: {}", item.name)),
            RigidBody::Dynamic,
            item.collider
                .clone()
                .unwrap_or(Collider::cuboid(0.5, 0.5, 0.5)),
            Friction::new(0.7),
            Restitution::new(0.3),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new(format!("Item Scene: {}", item.name)),
                SceneRoot(item.scene.clone()),
                Transform::from_translation(item.offset.unwrap_or_default())
                    .with_scale(item.scale.unwrap_or(Vec3::ONE)),
            ));
        });
}

fn on_crafter_added(trigger: Trigger<OnAdd, Crafter>, mut commands: Commands) {
    let entity = trigger.target();

    commands.entity(entity).insert(CrafterCache::default());
}

fn update_crafter_cache(
    query: SpatialQuery,
    mut q_crafter: Query<(&GlobalTransform, &mut CrafterCache), With<Crafter>>,
    q_item: Query<&Item>,
) {
    for (transform, mut cache) in &mut q_crafter {
        let shape = Collider::sphere(1.0);
        let origin = transform.translation();
        let direction = transform.forward();
        let config = ShapeCastConfig::from_max_distance(10.0);
        let filter = SpatialQueryFilter { ..default() };

        let mut hits = vec![];
        query.shape_hits_callback(
            &shape,
            origin,
            Quat::default(),
            direction,
            &config,
            &filter,
            |hit| {
                if q_item.contains(hit.entity) {
                    hits.push(hit);
                }
                true
            },
        );

        cache.clear();
        cache.extend(hits.iter().map(|hit| hit.entity));
    }
}

fn gizmos_crafter_cache(
    mut gizmos: Gizmos,
    q_crafter: Query<&CrafterCache, With<Crafter>>,
    q_items: Query<&GlobalTransform, With<Item>>,
) {
    for cache in &q_crafter {
        for entity in cache.iter() {
            let Ok(item_transform) = q_items.get(*entity) else {
                continue;
            };
            gizmos.sphere(
                Isometry3d::from_translation(item_transform.translation()),
                1.0,
                Color::srgb(0.0, 1.0, 0.0),
            );
        }
    }
}

fn handle_crater_input(
    mut commands: Commands,
    q_crafter: Query<(&CrafterInput, &CrafterCache, &GlobalTransform), With<Crafter>>,
    q_item: Query<(Entity, &Item)>,
    recipe: Res<RecipeAssets>,
) {
    // XXX: We need a lock on the items, e.g two players craft at the same time, that would lead
    // to race conditions where the item gets crafted by both players with the same items.
    for (input, cache, transform) in &q_crafter {
        apply_crafter_action(
            &mut commands,
            input,
            cache,
            &q_item,
            &recipe,
            transform.translation(),
        );
    }
}

fn apply_crafter_action(
    commands: &mut Commands,
    input: &CrafterInput,
    cache: &CrafterCache,
    q_item: &Query<(Entity, &Item)>,
    recipe: &Res<RecipeAssets>,
    origin: Vec3,
) {
    if input.craft {
        let items = cache
            .iter()
            .filter_map(|entity| q_item.get(*entity).ok())
            .collect::<Vec<(Entity, &Item)>>();
        let available: HashMap<ItemID, u32> =
            items.iter().fold(HashMap::new(), |mut acc, (_, item)| {
                *acc.entry(item.0.clone()).or_insert(0) += 1;
                acc
            });

        for recipe in &recipe.recipes {
            if recipe.can_craft(&available) {
                trace!("Crafting recipe: {:?}", recipe);

                // remove the input items
                for input in &recipe.input {
                    let mut to_remove = input.count;
                    for (entity, item) in &items {
                        if item.0 == input.item_id && to_remove > 0 {
                            commands.entity(*entity).despawn();
                            to_remove -= 1;
                        }
                    }
                }

                // spawn the output items at the origin
                for output in &recipe.output {
                    for _ in 0..output.count {
                        commands.spawn((
                            Name::new("Crafted Item"),
                            Item(output.item_id.clone()),
                            Transform::from_translation(origin),
                        ));
                    }
                }
            } else {
                trace!("Cannot craft recipe: {:?}", recipe);
            }
        }
    }
}
