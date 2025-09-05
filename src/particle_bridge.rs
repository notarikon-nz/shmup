use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::components::*;
use crate::events::*;
use crate::hanabi_particles::*;
use crate::{GameState, IsPaused};

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
                effect_type: if event.config.organic_motion {
                    ParticleEffectType::BioluminescentTrail
                } else {
                    ParticleEffectType::Explosion
                },
                is_active: true,
            },
        ));

        // Configure based on particle type - maintain compatibility with old organic motion system
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

// System to spawn ATP collection effects when enemies die
pub fn spawn_atp_hanabi_effect(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    effects: Res<ParticleEffects>,
) {
    for event in explosion_events.read() {
        if event.enemy_type.is_some() {
            // Spawn ATP collection effect at explosion site
            commands.spawn((
                ParticleEffect::new(effects.atp_collection.clone()),
                Transform::from_translation(event.position),
                HanabiParticleEffect {
                    effect_type: ParticleEffectType::ATP,
                    is_active: true,
                },
            ));
        }
    }
}

// System to spawn explosion effects using hanabi
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
            ParticleEffect::new(effect_handle),
            Transform::from_translation(event.position),
            HanabiParticleEffect {
                effect_type: ParticleEffectType::Explosion,
                is_active: true,
            },
        ));
    }
}

// System to spawn engine trails for player movement
pub fn spawn_engine_trail_effect(
    mut commands: Commands,
    player_query: Query<Entity, (Added<Player>, With<EngineTrail>)>,
    effects: Res<ParticleEffects>,
) {
    for player_entity in player_query.iter() {
        commands.entity(player_entity).with_children(|parent| {
            parent.spawn((
                ParticleEffect::new(effects.engine_trail.clone()),
                Transform::from_xyz(0.0, -20.0, -0.1),
                HanabiParticleEffect {
                    effect_type: ParticleEffectType::EngineTrail,
                    is_active: true,
                },
            ));
        });
    }
}

// System to handle thermal vent effects from thermal vents
pub fn spawn_thermal_hanabi_effects(
    mut commands: Commands,
    thermal_query: Query<&Transform, Added<ThermalParticle>>,
    effects: Res<ParticleEffects>,
) {
    for transform in thermal_query.iter() {
        commands.spawn((
            ParticleEffect::new(effects.thermal_vent.clone()),
            *transform,
            HanabiParticleEffect {
                effect_type: ParticleEffectType::ThermalVent,
                is_active: true,
            },
        ));
    }
}

// System to handle pheromone effects for enemy chemical communication
pub fn spawn_pheromone_hanabi_effects(
    mut commands: Commands,
    pheromone_query: Query<&Transform, Added<PheromoneParticle>>,
    effects: Res<ParticleEffects>,
) {
    for transform in pheromone_query.iter() {
        commands.spawn((
            ParticleEffect::new(effects.pheromone_trail.clone()),
            *transform,
            HanabiParticleEffect {
                effect_type: ParticleEffectType::Pheromone,
                is_active: true,
            },
        ));
    }
}

// System to spawn spore cloud effects for weapon systems
pub fn spawn_spore_cloud_hanabi_effects(
    mut commands: Commands,
    mut spore_events: EventReader<SpawnParticles>,
    effects: Res<ParticleEffects>,
) {
    for event in spore_events.read() {
        // Check if this is a spore/toxin cloud type particle
        if event.config.bioluminescence > 0.7 && event.count > 50 {
            commands.spawn((
                ParticleEffect::new(effects.spore_cloud.clone()),
                Transform::from_translation(event.position),
                HanabiParticleEffect {
                    effect_type: ParticleEffectType::SporeCloud,
                    is_active: true,
                },
            ));
        }
    }
}

// System to spawn background plankton effects for ambience
pub fn spawn_background_plankton_effects(
    mut commands: Commands,
    effects: Res<ParticleEffects>,
    mut spawn_timer: Local<f32>,
    time: Res<Time>,
    camera_query: Query<&Transform, (With<Camera>, Without<HanabiParticleEffect>)>,
) {
    *spawn_timer += time.delta_secs();
    
    // Spawn background plankton every 10 seconds around the camera
    if *spawn_timer >= 10.0 {
        *spawn_timer = 0.0;
        
        if let Ok(camera_transform) = camera_query.single() {
            // Spawn plankton around the camera for ambient effect
            let offset = Vec3::new(
                (time.elapsed_secs() * 123.45).sin() * 300.0,
                (time.elapsed_secs() * 678.90).cos() * 200.0,
                -50.0,
            );
            
            commands.spawn((
                ParticleEffect::new(effects.background_plankton.clone()),
                Transform::from_translation(camera_transform.translation + offset),
                HanabiParticleEffect {
                    effect_type: ParticleEffectType::Background,
                    is_active: true,
                },
            ));
        }
    }
}

// System to update particle effects based on player position
// Engine trails automatically follow player through parent-child hierarchy
pub fn update_player_particle_effects(
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut effect_query: Query<
        &mut Transform, 
        (
            With<HanabiParticleEffect>, 
            Without<Player>,
            With<ParticleEffect>
        )
    >,
) {
    if let Ok(_player_transform) = player_query.single() {
        // Engine trails are handled automatically by parent-child transforms
        // Other effects can be updated here if needed
        for _effect_transform in effect_query.iter_mut() {
            // Future: Could update non-child particle effects here
        }
    }
}

// System to clean up completed particle effects more intelligently
pub fn cleanup_completed_hanabi_effects(
    mut commands: Commands,
    effect_query: Query<(Entity, &HanabiParticleEffect, &Transform), With<ParticleEffect>>,
    time: Res<Time>,
    mut cleanup_timer: Local<f32>,
    camera_query: Query<&Transform, (With<Camera>, Without<HanabiParticleEffect>)>,
) {
    *cleanup_timer += time.delta_secs();
    
    // Clean up effects every 5 seconds
    if *cleanup_timer >= 5.0 {
        *cleanup_timer = 0.0;
        
        if let Ok(camera_transform) = camera_query.single() {
            for (entity, hanabi_effect, effect_transform) in effect_query.iter() {
                let distance_to_camera = camera_transform.translation.distance(effect_transform.translation);
                
                // Clean up effects that are very far from camera (> 1000 units)
                // or have been running for background effects
                let should_cleanup = match hanabi_effect.effect_type {
                    ParticleEffectType::Background => distance_to_camera > 800.0,
                    ParticleEffectType::Explosion | ParticleEffectType::CellBurst => distance_to_camera > 1200.0,
                    ParticleEffectType::ATP => distance_to_camera > 600.0,
                    ParticleEffectType::ThermalVent => distance_to_camera > 1000.0,
                    ParticleEffectType::Pheromone => distance_to_camera > 500.0,
                    ParticleEffectType::SporeCloud => distance_to_camera > 800.0,
                    ParticleEffectType::BioluminescentTrail => distance_to_camera > 700.0,
                    ParticleEffectType::EngineTrail => false, // Never cleanup engine trails automatically
                };
                
                if should_cleanup {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// System to activate/deactivate particle effects based on game state
pub fn manage_particle_effect_states(
    mut effect_query: Query<(&mut ParticleEffect, &HanabiParticleEffect)>,
    game_state: Option<Res<State<GameState>>>,
    pause_state: Option<Res<State<IsPaused>>>,
) {
    // Only manage states if both resources exist
    let should_be_active = match (game_state, pause_state) {
        (Some(game_state), Some(pause_state)) => {
            matches!(game_state.get(), GameState::Playing) && 
            matches!(pause_state.get(), IsPaused::Running)
        }
        _ => true, // Default to active if states don't exist yet
    };
    
    for (mut _particle_effect, _hanabi_effect) in effect_query.iter_mut() {
        // In practice, bevy_hanabi effects are controlled by their spawner settings
        // and don't need manual activation/deactivation
        // This system exists for potential future state management needs
        let _ = should_be_active; // Suppress unused variable warning
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
            spawn_spore_cloud_hanabi_effects,
            spawn_background_plankton_effects,
            update_player_particle_effects,
            cleanup_completed_hanabi_effects,
            manage_particle_effect_states,
        ));
    }
}