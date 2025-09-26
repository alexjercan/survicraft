use bevy::prelude::*;
use super::resources::*;

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
pub(super) struct CrafterCache(pub Vec<Entity>);
