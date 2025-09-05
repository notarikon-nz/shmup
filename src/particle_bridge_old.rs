use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::components::*;
use crate::events::*;
use crate::hanabi_particles::*;

// Bridge system to convert old particle events to hanabi effects
pub fn bridge_particle_events(
    mut commands: Commands,
    mut particle_events: EventReader<SpawnParticles>,
    effects: Res<ParticleEffects>,
) {
    for event in particle_events.read() {
        let effect_handle = if event.config.organic_motion {
            effects.bioluminescent_trail.clone()
        } else {
            effects.explosion.clone()
        };

        // Spawn hanabi particle effect
        let mut spawner = commands.spawn((
            ParticleEffect::new(effect_handle.clone()),
            Transform::from_translation(event.position),
            HanabiParticleEffect {
                effect_type: ParticleEffectType::BioluminescentTrail,
                is_active: true,
            },
        ));

        // Configure based on particle type
        if event.config.organic_motion {
            spawner.insert(BioluminescentParticle {
                base_color: event.config.color_start,
                pulse_frequency: 2.0,
                pulse_intensity: event.config.bioluminescence,
                organic_motion: OrganicMotion {
                    undulation_speed: 1.0,
                    response_to_current: 0.6,
                },
            });
        }
    }
}

// System to spawn ATP collection effects
pub fn spawn_atp_hanabi_effect(
    mut commands: Commands,
    mut collision_events: EventReader<EnemyHit>,
    effects: Res<ParticleEffects>,
    explosion_events: EventReader<SpawnExplosion>,
) {
    for event in explosion_events.read() {
        if event.enemy_type.is_some() {
            // Spawn ATP collection effect at explosion site
            commands.spawn((
                (
                    effect: ParticleEffect::new(effects.atp_collection.clone()),
                    transform: Transform::from_translation(event.position),
                    ..Default::default()
                },
                HanabiParticleEffect {
                    effect_type: ParticleEffectType::ATP,
                    is_active: true,
                },
            ));
        }
    }
}

// System to spawn explosion effects
pub fn spawn_explosion_hanabi_effect(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    effects: Res<ParticleEffects>,
) {
    for event in explosion_events.read() {
        let effect_handle = if event.enemy_type.is_some() {
            effects.cell_burst.clone()
        } else {
            effects.explosion.clone()
        };

        commands.spawn((
            (
                effect: ParticleEffect::new(effect_handle),
                transform: Transform::from_translation(event.position),
                ..Default::default()
            },
            HanabiParticleEffect {
                effect_type: ParticleEffectType::Explosion,
                is_active: true,
            },
        ));
    }
}

// System to spawn engine trails for player
pub fn spawn_engine_trail_effect(
    mut commands: Commands,
    player_query: Query<Entity, (Added<Player>, With<EngineTrail>)>,
    effects: Res<ParticleEffects>,
) {
    for player_entity in player_query.iter() {
        commands.entity(player_entity).with_children(|parent| {
            parent.spawn((
                (
                    effect: ParticleEffect::new(effects.engine_trail.clone()),
                    transform: Transform::from_xyz(0.0, -20.0, -0.1),
                    ..Default::default()
                },
                HanabiParticleEffect {
                    effect_type: ParticleEffectType::EngineTrial,
                    is_active: true,
                },
            ));
        });
    }
}

// System to handle thermal vent effects
pub fn spawn_thermal_hanabi_effects(
    mut commands: Commands,
    thermal_query: Query<&Transform, Added<ThermalParticle>>,
    effects: Res<ParticleEffects>,
) {
    for transform in thermal_query.iter() {
        commands.spawn((
            (
                effect: ParticleEffect::new(effects.thermal_vent.clone()),
                transform: *transform,
                ..Default::default()
            },
            HanabiParticleEffect {
                effect_type: ParticleEffectType::ThermalVent,
                is_active: true,
            },
        ));
    }
}

// System to handle pheromone effects
pub fn spawn_pheromone_hanabi_effects(
    mut commands: Commands,
    pheromone_query: Query<&Transform, Added<PheromoneParticle>>,
    effects: Res<ParticleEffects>,
) {
    for transform in pheromone_query.iter() {
        commands.spawn((
            (
                effect: ParticleEffect::new(effects.pheromone_trail.clone()),
                transform: *transform,
                ..Default::default()
            },
            HanabiParticleEffect {
                effect_type: ParticleEffectType::Pheromone,
                is_active: true,
            },
        ));
    }
}

// System to clean up completed particle effects
pub fn cleanup_completed_hanabi_effects(
    mut commands: Commands,
    effect_query: Query<(Entity, &EffectSpawner), With<HanabiParticleEffect>>,
) {
    for (entity, spawner) in effect_query.iter() {
        // Clean up effects that are no longer active and have no living particles
        if !spawner.is_active() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct ParticleBridgePlugin;

impl Plugin for ParticleBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            bridge_particle_events,
            spawn_atp_hanabi_effect,
            spawn_explosion_hanabi_effect,
            spawn_engine_trail_effect,
            spawn_thermal_hanabi_effects,
            spawn_pheromone_hanabi_effects,
            cleanup_completed_hanabi_effects,
        ));
    }
}