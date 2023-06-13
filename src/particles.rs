use crate::file_system_interaction::config::GameConfig;
use crate::level_instantiation::spawning::objects::player;
use crate::movement::general_movement::Grounded;
use crate::particles::init::init_effects;
use crate::util::trait_extension::{F32Ext, Vec3Ext};
use crate::GameState;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;

mod init;

/// Handles particle effects instantiation and playing.
pub(crate) fn particle_plugin(app: &mut App) {
    app.register_type::<SprintingParticle>()
        .add_plugin(HanabiPlugin)
        .add_system(init_effects.in_schedule(OnExit(GameState::Loading)))
        .add_system(play_sprinting_effect.in_set(OnUpdate(GameState::Playing)))
        .add_system(handle_timed_particles.in_set(OnUpdate(GameState::Playing)))
        .insert_resource(ParticleEffects::default());
}

#[derive(Debug, Clone, PartialEq, Resource, Reflect, Default)]
#[reflect(Resource)]
pub(crate) struct ParticleEffects {
    pub(crate) firework: Option<Handle<EffectAsset>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
struct SprintingParticle;

fn play_sprinting_effect(
    with_player: Query<(&Transform, &Grounded, &Velocity), Without<SprintingParticle>>,
    mut with_particle: Query<(&mut Transform, &mut ParticleEffect), With<SprintingParticle>>,
    config: Res<GameConfig>,
) {
    for (player_transform, grounded, velocity) in with_player.iter() {
        let horizontal_speed_squared = velocity
            .linvel
            .split(player_transform.up())
            .horizontal
            .length_squared();
        for (mut particle_transform, mut effect) in with_particle.iter_mut() {
            let threshold = config.player.sprint_effect_speed_threshold;
            if grounded.0 && horizontal_speed_squared > threshold.squared() {
                let translation = player_transform.translation
                    - player_transform.up() * (player::HEIGHT / 2. + player::RADIUS);
                *particle_transform = player_transform.with_translation(translation);
                effect.maybe_spawner().unwrap().set_active(true);
            } else {
                effect.maybe_spawner().unwrap().set_active(false);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct TimedParticle {
    pub(crate) length: f32,
    pub(crate) time_played: f32,
    pub(crate) destroy_on_completion: bool,
}

fn handle_timed_particles(
    mut timed_particles: Query<(Entity, &mut ParticleEffect, &mut TimedParticle)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut particle_effect, mut timed_particle) in &mut timed_particles {
        timed_particle.time_played += time.delta_seconds();
        if timed_particle.time_played >= timed_particle.length {
            if let Some(spawner) = particle_effect.maybe_spawner() {
                spawner.set_active(false);
                if timed_particle.destroy_on_completion {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
