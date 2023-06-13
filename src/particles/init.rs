use crate::level_instantiation::spawning::objects::player;
use crate::particles::SprintingParticle;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use super::ParticleEffects;

pub(crate) fn init_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut particle_effects: ResMut<ParticleEffects>,
) {
    let sprinting = create_sprinting_effect(&mut effects);
    commands.spawn((
        Name::new("Sprinting particle"),
        SprintingParticle,
        ParticleEffectBundle {
            effect: sprinting,
            ..default()
        },
        NotShadowReceiver,
    ));

    let firework_handle = create_firework_effect(&mut effects);
    particle_effects.firework = Some(firework_handle);
}

fn create_sprinting_effect(effects: &mut Assets<EffectAsset>) -> ParticleEffect {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(1.2, 1.0, 1.0, 0.6));
    color_gradient.add_key(0.1, Vec4::new(1.2, 1.0, 1.0, 0.4));
    color_gradient.add_key(0.6, Vec4::new(1.2, 1.0, 1.0, 0.2));
    color_gradient.add_key(1.0, Vec4::new(1.2, 1.0, 1.0, 0.0));

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec2::splat(0.1));
    size_gradient.add_key(0.3, Vec2::splat(0.12));
    size_gradient.add_key(0.6, Vec2::splat(0.15));
    size_gradient.add_key(1.0, Vec2::splat(0.2));

    ParticleEffect::new(
        effects.add(
            EffectAsset {
                name: "Sprint".to_string(),
                capacity: 100,
                spawner: Spawner::rate(10.0.into()).with_active(false),
                ..Default::default()
            }
            .init(InitPositionCircleModifier {
                dimension: ShapeDimension::Volume,
                radius: player::RADIUS * 0.5,
                center: Vec3::ZERO,
                axis: Vec3::Y,
            })
            .init(InitVelocitySphereModifier {
                speed: 1_f32.into(),
                center: Vec3::ZERO,
            })
            .init(InitLifetimeModifier {
                lifetime: 0.8.into(),
            })
            .update(LinearDragModifier { drag: 5. })
            .render(BillboardModifier {})
            .update(AccelModifier::constant(Vec3::new(0., 1., 0.)))
            .render(ColorOverLifetimeModifier {
                gradient: color_gradient,
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
            }),
        ),
    )
}

fn create_firework_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(0.3, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let firework = effects.add(
        EffectAsset {
            name: "firework".to_string(),
            capacity: 32768,
            spawner: Spawner::once(2500.0.into(), true),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 0.00001,
            dimension: ShapeDimension::Volume,
        })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            // Give a bit of variation by randomizing the initial speed
            speed: Value::Uniform((15., 25.)),
        })
        .init(InitLifetimeModifier {
            // Give a bit of variation by randomizing the lifetime per particle
            lifetime: Value::Uniform((0.8, 1.2)),
        })
        .init(InitAgeModifier {
            // Give a bit of variation by randomizing the age per particle. This will control the
            // starting color and starting size of particles.
            age: Value::Uniform((0.0, 0.2)),
        })
        .update(LinearDragModifier { drag: 5. })
        .update(AccelModifier::constant(Vec3::new(0., -8., 0.)))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
        }),
    );
    firework
}
