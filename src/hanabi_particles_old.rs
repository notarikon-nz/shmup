use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use rand::distributions::Uniform;
use crate::components::*;
use crate::resources::*;
use crate::events::*;

#[derive(Component)]
pub struct HanabiParticleEffect {
    pub effect_type: ParticleEffectType,
    pub is_active: bool,
}

#[derive(Clone)]
pub enum ParticleEffectType {
    BioluminescentTrail,
    ThermalVent,
    Explosion,
    ATP,
    Pheromone,
    EngineTrial,
    CellBurst,
    SporeCloud,
    Background,
}

pub struct HanabiParticlePlugin;

impl Plugin for HanabiParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(Startup, setup_particle_effects)
            .add_systems(Update, (
                update_bioluminescent_effects,
                update_thermal_effects,
                update_pheromone_effects,
                spawn_hanabi_particles_system,
            ));
    }
}

#[derive(Resource)]
pub struct ParticleEffects {
    pub bioluminescent_trail: Handle<EffectAsset>,
    pub thermal_vent: Handle<EffectAsset>,
    pub explosion: Handle<EffectAsset>,
    pub atp_collection: Handle<EffectAsset>,
    pub pheromone_trail: Handle<EffectAsset>,
    pub engine_trail: Handle<EffectAsset>,
    pub cell_burst: Handle<EffectAsset>,
    pub spore_cloud: Handle<EffectAsset>,
    pub background_plankton: Handle<EffectAsset>,
}

pub fn setup_particle_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // Bioluminescent trail effect for organic particles
    let bioluminescent_trail = effects.add(
        EffectAsset::new(
            32768,
            Spawner::rate(CpuValue::Single(500.0)).with_starts_active(false),
        )
        .with_name("bioluminescent_trail")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 2.0,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: (30.0..80.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(1.0, 3.0).into()))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(0.8, 2.0).into()))
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(0.4, 0.9, 0.7, 0.0)),
                GradientKey::new(0.1, Vec4::new(0.4, 0.9, 0.7, 1.0)),
                GradientKey::new(0.9, Vec4::new(0.6, 1.0, 0.8, 0.8)),
                GradientKey::new(1.0, Vec4::new(0.8, 1.0, 0.9, 0.0)),
            ]),
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, 0.0),
                GradientKey::new(0.1, 1.0),
                GradientKey::new(0.9, 1.0),
                GradientKey::new(1.0, 0.0),
            ]),
        }),
    );

    // Thermal vent effect with heat shimmer
    let thermal_vent = effects.add(
        EffectAsset::new(
            1024,
            Spawner::rate(CpuValue::Single(50.0)).with_starts_active(true),
        )
        .with_name("thermal_vent")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 8.0,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::new(0.0, 1.0, 0.0),
            speed: (10.0..40.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(2.0, 4.0).into()))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(3.0, 8.0).into()))
        .update(AccelModifier::new(Vec3::new(0.0, 25.0, 0.0))) // Rising heat
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(1.0, 0.6, 0.2, 0.8)),
                GradientKey::new(0.5, Vec4::new(1.0, 0.8, 0.4, 0.6)),
                GradientKey::new(1.0, Vec4::new(1.0, 1.0, 0.8, 0.0)),
            ]),
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, 0.5),
                GradientKey::new(0.3, 1.0),
                GradientKey::new(1.0, 1.5),
            ]),
        }),
    );

    // Explosion effect with multiple layers
    let explosion = effects.add(
        EffectAsset::new(
            2048,
            Spawner::burst(CpuValue::Single(200.0), CpuValue::Single(0.1)).with_starts_active(false),
        )
        .with_name("explosion")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 5.0,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: (50.0..200.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(0.5, 2.0).into()))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(2.0, 8.0).into()))
        .update(AccelModifier::new(Vec3::new(0.0, -50.0, 0.0))) // Gravity
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                GradientKey::new(0.1, Vec4::new(1.0, 0.8, 0.3, 1.0)),
                GradientKey::new(0.5, Vec4::new(0.8, 0.4, 0.1, 0.8)),
                GradientKey::new(1.0, Vec4::new(0.2, 0.2, 0.2, 0.0)),
            ]),
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, 0.0),
                GradientKey::new(0.1, 1.0),
                GradientKey::new(1.0, 0.0),
            ]),
        }),
    );

    // ATP collection effect with magnetic attraction
    let atp_collection = effects.add(
        EffectAsset::new(
            vec![1024, 1],
            Spawner::rate(100.0.into())
                .with_starts_active(false),
            move_modifier(),
        )
        .with_name("atp_collection")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 15.0,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: (20.0..60.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(1.0, 2.0)))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(1.0, 3.0)))
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(1.0, 1.0, 0.3, 0.0)),
                GradientKey::new(0.1, Vec4::new(1.0, 1.0, 0.5, 1.0)),
                GradientKey::new(0.9, Vec4::new(1.0, 0.8, 0.3, 0.8)),
                GradientKey::new(1.0, Vec4::new(0.8, 0.6, 0.2, 0.0)),
            ]),
        }),
    );

    // Pheromone trail effect for chemical communication
    let pheromone_trail = effects.add(
        EffectAsset::new(
            vec![512, 1],
            Spawner::rate(20.0.into())
                .with_starts_active(true),
            move_modifier(),
        )
        .with_name("pheromone_trail")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 3.0,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: (5.0..15.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(3.0, 6.0)))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(0.5, 2.0)))
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(0.8, 0.3, 0.8, 0.0)),
                GradientKey::new(0.2, Vec4::new(0.9, 0.4, 0.9, 0.6)),
                GradientKey::new(0.8, Vec4::new(0.7, 0.2, 0.7, 0.4)),
                GradientKey::new(1.0, Vec4::new(0.5, 0.1, 0.5, 0.0)),
            ]),
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, 0.5),
                GradientKey::new(0.5, 1.0),
                GradientKey::new(1.0, 1.5),
            ]),
        }),
    );

    // Engine trail effect
    let engine_trail = effects.add(
        EffectAsset::new(
            vec![1024, 1],
            Spawner::rate(200.0.into())
                .with_starts_active(true),
            move_modifier(),
        )
        .with_name("engine_trail")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 1.0,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::new(0.0, -1.0, 0.0),
            speed: (10.0..30.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(0.5, 1.5)))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(0.5, 1.5)))
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(0.3, 0.8, 1.0, 1.0)),
                GradientKey::new(0.5, Vec4::new(0.4, 0.9, 1.0, 0.8)),
                GradientKey::new(1.0, Vec4::new(0.6, 1.0, 1.0, 0.0)),
            ]),
        }),
    );

    // Cell burst effect for enemy destruction
    let cell_burst = effects.add(
        EffectAsset::new(
            vec![512, 2],
            Spawner::burst(50.0.into(), 0.05.into())
                .with_starts_active(false),
            move_modifier(),
        )
        .with_name("cell_burst")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 2.0,
            dimension: ShapeDimension::Surface,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: (30.0..100.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(0.3, 1.0)))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(1.0, 4.0)))
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(0.8, 1.0, 0.9, 1.0)),
                GradientKey::new(0.3, Vec4::new(0.6, 0.9, 0.7, 0.8)),
                GradientKey::new(1.0, Vec4::new(0.4, 0.7, 0.5, 0.0)),
            ]),
        }),
    );

    // Spore cloud effect for toxin clouds
    let spore_cloud = effects.add(
        EffectAsset::new(
            vec![2048, 1],
            Spawner::rate(150.0.into())
                .with_starts_active(false),
            move_modifier(),
        )
        .with_name("spore_cloud")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 20.0,
            dimension: ShapeDimension::Volume,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: (5.0..25.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(4.0, 8.0)))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(2.0, 6.0)))
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(0.8, 0.6, 1.0, 0.0)),
                GradientKey::new(0.2, Vec4::new(0.9, 0.7, 1.0, 0.8)),
                GradientKey::new(0.8, Vec4::new(0.7, 0.5, 0.9, 0.6)),
                GradientKey::new(1.0, Vec4::new(0.6, 0.4, 0.8, 0.0)),
            ]),
        }),
    );

    // Background plankton effect
    let background_plankton = effects.add(
        EffectAsset::new(
            vec![1024, 1],
            Spawner::rate(30.0.into())
                .with_starts_active(true),
            move_modifier(),
        )
        .with_name("background_plankton")
        .init(SetPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 100.0,
            dimension: ShapeDimension::Volume,
        })
        .init(SetVelocitySphereModifier {
            center: Vec3::new(0.0, -0.5, 0.0),
            speed: (5.0..20.0).into(),
        })
        .init(SetAttributeModifier::new(Attribute::LIFETIME, Uniform::new(10.0, 20.0)))
        .init(SetAttributeModifier::new(Attribute::SIZE, Uniform::new(0.3, 1.0)))
        .render(ColorOverLifetimeModifier {
            gradient: Gradient::new(vec![
                GradientKey::new(0.0, Vec4::new(0.8, 1.0, 0.9, 0.0)),
                GradientKey::new(0.1, Vec4::new(0.8, 1.0, 0.9, 0.4)),
                GradientKey::new(0.9, Vec4::new(0.6, 0.9, 0.7, 0.3)),
                GradientKey::new(1.0, Vec4::new(0.5, 0.8, 0.6, 0.0)),
            ]),
        }),
    );

    commands.insert_resource(ParticleEffects {
        bioluminescent_trail,
        thermal_vent,
        explosion,
        atp_collection,
        pheromone_trail,
        engine_trail,
        cell_burst,
        spore_cloud,
        background_plankton,
    });
}

fn move_modifier() -> impl ExprWriter {
    AccelModifier::new(Vec3::ZERO)
}

pub fn update_bioluminescent_effects(
    mut effect_query: Query<&mut EffectSpawner, With<HanabiParticleEffect>>,
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut effect in effect_query.iter_mut() {
            // Update effect position to follow player
            effect.set_active(true);
        }
    }
}

pub fn update_thermal_effects(
    mut effect_query: Query<(&mut Transform, &mut EffectSpawner), With<ThermalParticle>>,
    current_generator: Option<Res<CurrentGenerator>>,
    time: Res<Time>,
) {
    if let Some(generator) = current_generator {
        for vent in &generator.thermal_vents {
            if vent.active {
                for (mut transform, mut effect) in effect_query.iter_mut() {
                    transform.translation = vent.position.extend(0.0);
                    effect.set_active(true);
                }
            }
        }
    }
}

pub fn update_pheromone_effects(
    mut effect_query: Query<(&mut Transform, &mut EffectSpawner), With<PheromoneParticle>>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<PheromoneParticle>)>,
) {
    for enemy_transform in enemy_query.iter() {
        for (mut pheromone_transform, mut effect) in effect_query.iter_mut() {
            pheromone_transform.translation = enemy_transform.translation;
            effect.set_active(true);
        }
    }
}

pub fn spawn_hanabi_particles_system(
    mut commands: Commands,
    mut particle_events: EventReader<SpawnParticles>,
    effects: Res<ParticleEffects>,
) {
    for event in particle_events.read() {
        let effect_handle = match &event.config.organic_motion {
            true => effects.bioluminescent_trail.clone(),
            false => effects.explosion.clone(),
        };

        commands.spawn((
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect_handle),
                transform: Transform::from_translation(event.position),
                ..Default::default()
            },
            HanabiParticleEffect {
                effect_type: ParticleEffectType::BioluminescentTrail,
                is_active: true,
            },
        ));
    }
}