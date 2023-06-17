use crate::combat::health::Health;
use crate::file_system_interaction::asset_loading::{AnimationAssets, SceneAssets};
use crate::level_instantiation::spawning::GameObject;
use crate::movement::general_movement::{CharacterAnimations, CharacterControllerBundle, Model};
use crate::movement::navigation::Follower;
use bevy::prelude::*;
use std::f32::consts::TAU;
use serde::{Deserialize, Serialize};


pub(crate) const HEIGHT: f32 = 0.4;
pub(crate) const RADIUS: f32 = 0.4;

pub(crate) fn spawn(
    In(transform): In<Transform>,
    mut commands: Commands,
    animations: Res<AnimationAssets>,
    scene_handles: Res<SceneAssets>,
) {
    let entity = commands
        .spawn((
            PbrBundle {
                transform,
                ..default()
            },
            Name::new("Enemy"),
            CharacterControllerBundle::capsule(HEIGHT, RADIUS),
            Follower,
            CharacterAnimations {
                idle: animations.character_idle.clone(),
                walk: animations.character_walking.clone(),
                aerial: animations.character_running.clone(),
            },
            Health {
                hit_points: 100.0,
                max_hit_points: 100.0,
            },
            GameObject::Enemy,
            EnemyTag::default(),
        ))
        .id();

    commands
        .spawn((
            Model { target: entity },
            SpatialBundle::default(),
            Name::new("Enemy Model Parent"),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneBundle {
                    scene: scene_handles.character.clone(),
                    transform: Transform {
                        translation: Vec3::new(0., -HEIGHT / 2. - RADIUS, 0.),
                        scale: Vec3::splat(0.012),
                        rotation: Quat::from_rotation_y(TAU / 2.),
                    },
                    ..default()
                },
                Name::new("Enemy Model"),
            ));
        });
}

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct EnemyTag {}

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct EndlessEnemySpawner {
    pub(crate) max_enemies: usize,
}

impl Default for EndlessEnemySpawner {
    fn default() -> Self {
        Self { max_enemies: 0 }
    }
}

pub(crate) fn spawn_enemies_on_endless_spawner(
    enemies_query: Query<&EnemyTag>,
    enemy_spawners_query: Query<(&Transform, &EndlessEnemySpawner)>,
    mut commands: Commands,
    animations: Res<AnimationAssets>,
    scene_handles: Res<SceneAssets>,
) {
    let enemy_count = enemies_query.iter().count();
    for (transform, enemy_spawner) in &enemy_spawners_query {
        if enemy_count < enemy_spawner.max_enemies {
            let entity = commands
                .spawn((
                    PbrBundle {
                        transform: *transform,
                        ..default()
                    },
                    Name::new("Enemy"),
                    CharacterControllerBundle::capsule(HEIGHT, RADIUS),
                    Follower,
                    CharacterAnimations {
                        idle: animations.character_idle.clone(),
                        walk: animations.character_walking.clone(),
                        aerial: animations.character_running.clone(),
                    },
                    Health {
                        hit_points: 100.0,
                        max_hit_points: 100.0,
                    },
                    GameObject::Enemy,
                    EnemyTag::default(),
                ))
                .id();

            commands
                .spawn((
                    Model { target: entity },
                    SpatialBundle::default(),
                    Name::new("Enemy Model Parent"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SceneBundle {
                            scene: scene_handles.character.clone(),
                            transform: Transform {
                                translation: Vec3::new(0., -HEIGHT / 2. - RADIUS, 0.),
                                scale: Vec3::splat(0.012),
                                rotation: Quat::from_rotation_y(TAU / 2.),
                            },
                            ..default()
                        },
                        Name::new("Enemy Model"),
                    ));
                });
        }
    }
}
