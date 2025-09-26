use avian3d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use super::{components::*, resources::*};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>()
            .register_type::<CrafterCache>()
            .register_type::<CrafterInput>();

        app.add_observer(on_item_added);
        app.add_observer(on_crafter_added);

        app.add_systems(Update, update_crafter_cache);
        app.add_systems(Update, handle_crater_input);
    }
}

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
        ));
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
