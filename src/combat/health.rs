use std::f32::consts::E;

use crate::level_instantiation::spawning::objects::util::MeshAssetsExt;
use crate::level_instantiation::spawning::objects::GameCollisionGroup;
use crate::particles::{ParticleEffects, TimedParticle};
use crate::player_control::{camera::IngameCamera, player_embodiment::Player};
use crate::shader::Materials;
use crate::GameState;
use anyhow::{Context, Result};
use bevy::ecs::query;
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_hanabi::{ParticleEffect, ParticleEffectBundle};
use bevy_mod_sysfail::sysfail;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) fn health_plugin(app: &mut App) {
    app.register_type::<Health>()
        .add_system(apply_death);//.in_set(HealthSystemSet)
        //.in_set(OnUpdate(GameState::Playing));
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub(crate) struct HealthSystemSet;

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct Health {
    pub(crate) hit_points: f32,
    pub(crate) max_hit_points: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            hit_points: 0.0,
            max_hit_points: 0.0,
        }
    }
}

fn apply_death(query_health: Query<(Entity, &Health)>, mut commands: Commands) {
    for (entity, health) in &query_health {
        if health.hit_points <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
