use crate::level_instantiation::spawning::objects::util::MeshAssetsExt;
use crate::level_instantiation::spawning::objects::GameCollisionGroup;
use crate::player_control::{camera::IngameCamera, player_embodiment::Player};
use crate::shader::Materials;
use crate::GameState;
use anyhow::{Context, Result};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_mod_sysfail::sysfail;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct Shooting {
    /// Was shoot requested?
    pub(crate) requested: bool,
}

impl Default for Shooting {
    fn default() -> Self {
        Self { requested: false }
    }
}

#[derive(Debug, Clone, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(crate) struct Projectile {}

impl Default for Projectile {
    fn default() -> Self {
        Self {}
    }
}

pub(crate) fn shooting_plugin(app: &mut App) {
    app.register_type::<Shooting>().add_systems(
        (apply_shooting, apply_projectile_impact)
            .chain()
            .in_set(ShootingSystemSet)
            .in_set(OnUpdate(GameState::Playing)),
    );
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub(crate) struct ShootingSystemSet;

#[sysfail(log(level = "error"))]
fn apply_shooting(
    mut player_query: Query<(&mut Shooting, &Transform), With<Player>>,
    camera_query: Query<(&IngameCamera, &Transform), Without<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<Materials>,
) -> Result<()> {
    #[cfg(feature = "tracing")]
    let _span = info_span!("handle_horizontal_movement").entered();
    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return Ok(());
    };

    for (mut shooting, player_transform) in &mut player_query {
        if shooting.requested {
            let forward = camera_transform.forward();
            const SPAWN_FORWARD_ADJUST: f32 = 1.0;
            let projectile_starting_vel = 50.0;

            let mesh_handle = get_or_add_mesh_handle(&mut meshes);

            let projectile_transform = camera_transform
                .with_translation((forward * SPAWN_FORWARD_ADJUST) + camera_transform.translation);
            let projectile = commands
                .spawn((
                    MaterialMeshBundle {
                        mesh: mesh_handle,
                        material: materials.glowy.clone(),
                        transform: projectile_transform,
                        ..default()
                    },
                    Name::new("Projectile"),
                    ProjectileBundle::ball(0.1, forward * projectile_starting_vel),
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::DYNAMIC_DYNAMIC,
                    CollisionGroups::new(
                        GameCollisionGroup::OTHER.into(),
                        GameCollisionGroup::OTHER.into(),
                    ), 
                    Ccd::enabled(),
                ))
                .id(); 

            shooting.requested = false;
        }
    }
    Ok(())
}

fn apply_projectile_impact(
/*     mut collision_events: EventReader<CollisionEvent>,
    projectile_query: Query<Entity, With<Projectile>>,
    parent_query: Query<&Parent>, */
) {
/*     for event in collision_events.iter() {
        let (entity_a, entity_b, ongoing) = unpack_collision_event(event);

        let (_player_entity, target_entity) = match determine_projectile_and_target(
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
    } */
}

fn unpack_collision_event(event: &CollisionEvent) -> (Entity, Entity, bool) {
    match event {
        CollisionEvent::Started(entity_a, entity_b, _kind) => (*entity_a, *entity_b, true),
        CollisionEvent::Stopped(entity_a, entity_b, _kind) => (*entity_a, *entity_b, false),
    }
}

fn determine_projectile_and_target(
    player_query: &Query<Entity, With<Projectile>>,
    parent_query: &Query<&Parent>,
    entity_a: Entity,
    entity_b: Entity,
) -> Option<(Entity, Entity)> {
    if player_query.get(entity_a).is_ok() {
        let projectile_entity = entity_a;
        let target_entity = parent_query
            .get(entity_b)
            .map(|parent| parent.get())
            .unwrap_or(entity_b);
        Some((projectile_entity, target_entity))
    } else if player_query.get(entity_b).is_ok() {
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
            radius: 0.1,
            ..default()
        })
    })
}

#[derive(Debug, Clone, Bundle)]
pub(crate) struct ProjectileBundle {
    pub(crate) projectile: Projectile,
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

impl Default for ProjectileBundle {
    fn default() -> Self {
        Self {
            projectile: Projectile {},
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

impl ProjectileBundle {
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