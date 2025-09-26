use bevy::prelude::*;

#[cfg(feature = "debug")]
use self::debug::*;
use super::{components::*, resources::*};

pub struct InventoryRenderPlugin;

impl Plugin for InventoryRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_item_added);

        #[cfg(feature = "debug")]
        app.add_plugins(DebugPlugin);
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
        .insert(Visibility::Visible)
        .with_children(|parent| {
            parent.spawn((
                Name::new(format!("Item Scene: {}", item.name)),
                SceneRoot(item.scene.clone()),
                Transform::from_translation(item.offset.unwrap_or_default())
                    .with_scale(item.scale.unwrap_or(Vec3::ONE)),
            ));
        });
}

#[cfg(feature = "debug")]
mod debug {
    use super::*;

    #[derive(Debug, Resource, Default, Clone, Deref, DerefMut)]
    struct ShowGrid(pub bool);

    pub(super) struct DebugPlugin;

    impl Plugin for DebugPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(ShowGrid(true));
            app.add_systems(Update, (toggle, gizmos_crafter_cache));
        }
    }

    fn toggle(kbd: Res<ButtonInput<KeyCode>>, mut show_grid: ResMut<ShowGrid>) {
        if kbd.just_pressed(KeyCode::F11) {
            show_grid.0 = !show_grid.0;
        }
    }

    fn gizmos_crafter_cache(
        mut gizmos: Gizmos,
        show_grid: Res<ShowGrid>,
        q_crafter: Query<&CrafterCache, With<Crafter>>,
        q_items: Query<&GlobalTransform, With<Item>>,
    ) {
        if !**show_grid {
            return;
        }

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
}
