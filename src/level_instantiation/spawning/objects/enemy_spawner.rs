use crate::file_system_interaction::asset_loading::{AnimationAssets, SceneAssets};
use crate::level_instantiation::spawning::GameObject;
use bevy::prelude::*;

use super::enemy::{EndlessEnemySpawner, EnemyTag};

pub(crate) fn spawn(In(transform): In<Transform>, mut commands: Commands) {
    let _entity = commands
        .spawn((
            Transform::from(transform),
            EndlessEnemySpawner { max_enemies: 10 },
            GameObject::EnemySpawner,
            EnemyTag::default(),
        ))
        .id();
}
