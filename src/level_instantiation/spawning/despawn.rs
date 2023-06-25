use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Component, Clone, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct Despawn {
    pub(crate) recursive: bool,
}

pub(crate) fn despawn(mut commands: Commands, despawn_query: Query<(Entity, &Despawn, &Children)>) {
    for (entity, despawn, children) in despawn_query.iter() {
        if despawn.recursive {
            if let Some(entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn_recursive();
            }
        } else {
            for child in children.iter() {
                if let Some(mut entity_commands) = commands.get_entity(*child) {
                    entity_commands.remove_parent();
                }
            }
        }
    }
}
