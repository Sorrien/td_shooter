use crate::file_system_interaction::asset_loading::AudioAssets;
use crate::level_instantiation::spawning::objects::util::MeshAssetsExt;
use crate::level_instantiation::spawning::objects::GameCollisionGroup;
use crate::particles::{ParticleEffects, TimedParticle};
use crate::player_control::{camera::IngameCamera, player_embodiment::Player};
use crate::shader::Materials;
use crate::spatial_audio::{AudioEmitterHandle, CustomAudioEmitter, DisposableAudioEmitter};
use crate::GameState;
use anyhow::Result;
use bevy::render::render_resource::encase::rts_array::Length;
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_hanabi::{ParticleEffect, ParticleEffectBundle};
use bevy_kira_audio::{Audio, AudioControl, AudioInstance};
use bevy_mod_sysfail::sysfail;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::health::Health;

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct Shooting {
    /// Was shoot requested?
    pub(crate) requested: bool,
    pub(crate) shoot_delay_enabled: bool,
    pub(crate) shoot_delay_length: f32,
    pub(crate) shoot_delay_time: f32,
}

impl Default for Shooting {
    fn default() -> Self {
        Self {
            requested: false,
            shoot_delay_enabled: false,
            shoot_delay_length: 0.0,
            shoot_delay_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct PhysicsProjectile {}

impl Default for PhysicsProjectile {
    fn default() -> Self {
        Self {}
    }
}

pub(crate) fn shooting_plugin(app: &mut App) {
    app.register_type::<Shooting>()
        .add_systems(
            (apply_shooting, apply_projectile_impact)
                .chain()
                .in_set(ShootingSystemSet)
                .in_set(OnUpdate(GameState::Playing)),
        )
        .add_system(handle_tracing_projectile_movement);
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub(crate) struct ShootingSystemSet;

#[sysfail(log(level = "error"))]
fn apply_shooting(
    //mut player_query: Query<(&mut Shooting, &Transform, &mut CustomAudioEmitter), With<Player>>,
    mut player_query: Query<(&mut Shooting, &Transform), With<Player>>,
    camera_query: Query<(&IngameCamera, &Transform), Without<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<Materials>,
    time: Res<Time>,
    //mut audio_instances: ResMut<Assets<AudioInstance>>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) -> Result<()> {
    #[cfg(feature = "tracing")]
    let _span = info_span!("handle_horizontal_movement").entered();
    let Some((_camera, camera_transform)) = camera_query.iter().next() else {
        return Ok(());
    };
    let dt = time.delta_seconds();

    //for (mut shooting, player_transform, mut emitter) in &mut player_query {
    for (mut shooting, player_transform) in &mut player_query {
        if shooting.shoot_delay_enabled {
            if shooting.shoot_delay_time >= shooting.shoot_delay_length {
                shooting.shoot_delay_enabled = false;
                shooting.shoot_delay_time = 0.0;
            } else {
                shooting.shoot_delay_time += dt;
            }
        }

        if shooting.requested && !shooting.shoot_delay_enabled {
            let forward = camera_transform.forward();
            const SPAWN_FORWARD_ADJUST: f32 = 2.0;
            let projectile_starting_vel = 10.0;

            let mesh_handle = get_or_add_mesh_handle(&mut meshes);

            let projectile_transform = camera_transform
                .with_translation((forward * SPAWN_FORWARD_ADJUST) + camera_transform.translation);

            let is_physics_projectile = false;

            //spawn projectile
            let _projectile = if is_physics_projectile {
                commands
                    .spawn((
                        MaterialMeshBundle {
                            mesh: mesh_handle,
                            material: materials.glowy.clone(),
                            transform: projectile_transform,
                            ..default()
                        },
                        Name::new("Projectile"),
                        PhysicsProjectileBundle::ball(0.1, forward * projectile_starting_vel),
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::DYNAMIC_DYNAMIC,
                        CollisionGroups::new(
                            GameCollisionGroup::OTHER.into(),
                            GameCollisionGroup::OTHER.into(),
                        ),
                        Ccd::enabled(),
                    ))
                    .id()
            } else {
                commands
                    .spawn((
                        MaterialMeshBundle {
                            mesh: mesh_handle,
                            material: materials.glowy.clone(),
                            transform: projectile_transform,
                            ..default()
                        },
                        Name::new("Projectile"),
                        TracingProjectile {
                            velocity: forward * projectile_starting_vel,
                        },
                    ))
                    .id()
            };

            //play firing audio

            /*let rifle_shot_1_handle = audio
                .play(audio_assets.rifle_shot_1.clone())
                .with_volume(1.0)
                .handle();
            emitter.instances.push(rifle_shot_1_handle);

            let instance_count = emitter.instances.length();
            for i in 0..instance_count {
                let instance_handle = &emitter.instances[i];
                if let Some(audio_instance) = audio_instances.get_mut(instance_handle) {
                    //error!("found audio instance");
                } else {
                    emitter.instances.remove(i);

                    //error!("removing old instance handle");
                    //emitter.instances.push(rifle_shot_1_handle);
                }
            }*/

            //pick one of the shot sounds at random to provide some variety
            let rand_shot_index = rand::thread_rng().gen_range(0..4);
            let rifle_shot_handle = match rand_shot_index {
                1 => &audio_assets.rifle_shot_2,
                2 => &audio_assets.rifle_shot_4,
                3 => &audio_assets.rifle_shot_5,
                _ => &audio_assets.rifle_shot_1,
            };

            let rifle_shot_1_handle = audio
                .play(rifle_shot_handle.clone())
                .with_volume(0.6) //at least for now I'll relinquish realism along with the tinnitus
                .handle();

            commands.spawn((
                TransformBundle::from_transform(projectile_transform),
                AudioEmitterHandle {
                    instance: Some(rifle_shot_1_handle),
                },
                DisposableAudioEmitter {},
            ));

            shooting.requested = false;
            shooting.shoot_delay_enabled = true;
        } else if shooting.requested {
            shooting.requested = false;
        }
    }
    Ok(())
}

fn apply_projectile_impact(
    mut collision_events: EventReader<CollisionEvent>,
    projectile_query: Query<Entity, With<PhysicsProjectile>>,
    parent_query: Query<&Parent>,
    mut commands: Commands,
) {
    for event in collision_events.iter() {
        let (entity_a, entity_b, ongoing) = unpack_collision_event(event);

        let (projectile_entity, _target_entity) = match determine_projectile_and_target(
            &projectile_query,
            &parent_query,
            entity_a,
            entity_b,
        ) {
            Some((projectile, target)) => (projectile, target),
            None => continue,
        };
        if ongoing {
        } else {
        }
        commands.entity(projectile_entity).despawn();
    }
}

fn unpack_collision_event(event: &CollisionEvent) -> (Entity, Entity, bool) {
    match event {
        CollisionEvent::Started(entity_a, entity_b, _kind) => (*entity_a, *entity_b, true),
        CollisionEvent::Stopped(entity_a, entity_b, _kind) => (*entity_a, *entity_b, false),
    }
}

fn determine_projectile_and_target(
    projectile_query: &Query<Entity, With<PhysicsProjectile>>,
    parent_query: &Query<&Parent>,
    entity_a: Entity,
    entity_b: Entity,
) -> Option<(Entity, Entity)> {
    if projectile_query.get(entity_a).is_ok() {
        let projectile_entity = entity_a;
        let target_entity = parent_query
            .get(entity_b)
            .map(|parent| parent.get())
            .unwrap_or(entity_b);
        Some((projectile_entity, target_entity))
    } else if projectile_query.get(entity_b).is_ok() {
        let projectile_entity = entity_b;
        let target_entity = parent_query
            .get(entity_a)
            .map(|parent| parent.get())
            .unwrap_or(entity_a);
        Some((projectile_entity, target_entity))
    } else {
        None
    }
}

fn get_or_add_mesh_handle(mesh_assets: &mut Assets<Mesh>) -> Handle<Mesh> {
    const MESH_HANDLE: HandleUntyped =
        HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 0x1f40128bad02a9b);
    mesh_assets.get_or_add(MESH_HANDLE, || {
        Mesh::from(shape::UVSphere {
            radius: 0.01,
            ..default()
        })
    })
}

#[derive(Debug, Clone, Bundle)]
pub(crate) struct PhysicsProjectileBundle {
    pub(crate) projectile: PhysicsProjectile,
    pub(crate) gravity_scale: GravityScale,
    pub(crate) mass: ColliderMassProperties,
    pub(crate) read_mass: ReadMassProperties,
    pub(crate) damping: Damping,
    pub(crate) rigid_body: RigidBody,
    pub(crate) collider: Collider,
    pub(crate) force: ExternalForce,
    pub(crate) impulse: ExternalImpulse,
    pub(crate) velocity: Velocity,
    pub(crate) dominance: Dominance,
}

impl Default for PhysicsProjectileBundle {
    fn default() -> Self {
        Self {
            projectile: PhysicsProjectile {},
            read_mass: default(),
            gravity_scale: GravityScale(1.0),
            force: default(),
            mass: ColliderMassProperties::Mass(1.0),
            damping: Damping {
                linear_damping: 1.0,
                ..default()
            },
            collider: default(),
            rigid_body: RigidBody::Dynamic,
            impulse: default(),
            velocity: default(),
            dominance: default(),
        }
    }
}

impl PhysicsProjectileBundle {
    pub(crate) fn capsule(height: f32, radius: f32, lin_vel: Vec3) -> Self {
        Self {
            collider: Collider::capsule_y(height / 2., radius),
            velocity: Velocity {
                linvel: lin_vel,
                angvel: default(),
            },
            ..default()
        }
    }

    pub(crate) fn ball(radius: f32, lin_vel: Vec3) -> Self {
        Self {
            collider: Collider::ball(radius),
            velocity: Velocity {
                linvel: lin_vel,
                angvel: default(),
            },
            ..default()
        }
    }
}

#[derive(Debug, Clone, Bundle)]
pub(crate) struct TracingProjectileBundle {
    pub(crate) projectile: TracingProjectile,
}

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct TracingProjectile {
    velocity: Vec3,
}

impl Default for TracingProjectile {
    fn default() -> Self {
        Self {
            velocity: Vec3::default(),
        }
    }
}

fn handle_tracing_projectile_movement(
    mut tracing_projectiles: Query<(Entity, &mut TracingProjectile, &mut Transform)>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
    query_name: Query<&Name>,
    mut query_health: Query<&mut Health>,
    mut commands: Commands,
    particle_effects: Res<ParticleEffects>,
) {
    //error!("starting projectile system");
    for (projectile_entity, mut projectile, mut transform) in tracing_projectiles.iter_mut() {
        //error!("processing projectile {:?}", projectile_entity);
        if projectile.velocity.cmpgt(Vec3::ZERO).any() {
            //error!("projectile {:?} has velocity greater than 0", projectile_entity);
            let dt = time.delta_seconds();
            let ray_start = transform.translation;
            let travel_distance = projectile.velocity * dt;
            let gravity = Vec3::Y * -1.0; //ideally we'd pull the gravity from rapier
            let ray_end = ray_start + travel_distance + (gravity * dt); //things are dropping too quickly at the moment.

            let mut filter = QueryFilter::new();
            filter.flags |= QueryFilterFlags::EXCLUDE_SENSORS;

            let max_toi = ray_end.length();
            let ray_direction = ray_end - ray_start;
            /*             error!(
                "entity: {:?} ray dir: {:?} max_toi: {:?}",
                projectile_entity, ray_direction, max_toi
            ); */
            //let hit = rapier_context.cast_ray(ray_start, ray_direction, max_toi, true, filter);

            let hit = rapier_context.cast_ray_and_get_normal(
                ray_start,
                ray_direction,
                max_toi,
                true,
                filter,
            );
            if let Some((entity, ray_intersection)) = hit {
                //let entity_name = query_name.get(entity).unwrap();
                //error!("hit entity: {:?} {:?}", entity, entity_name);
                transform.translation = ray_intersection.point;
                if let Some(firework) = particle_effects.firework.clone() {
                    commands.spawn((
                        Name::new("Firework particle"),
                        ParticleEffectBundle {
                            effect: ParticleEffect::new(firework),
                            transform: transform.clone(),
                            ..default()
                        },
                        TimedParticle {
                            destroy_on_completion: true,
                            length: 3.0,
                            time_played: 0.0,
                        },
                    ));
                };

                if let Ok(mut health) = query_health.get_mut(entity) {
                    health.hit_points = 0.0; //all projectiles are instant kill for now
                }

                projectile.velocity = Vec3::ZERO;
                commands.entity(projectile_entity).despawn();
            } else {
                transform.translation = ray_end;
            }
        }
    }
}
