use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;

// FIXED: ATP pickup system - resolved query conflicts
pub fn atp_pickup_system(
    mut commands: Commands,
    // FIXED: Separate the ATP pickup query from player query
    atp_query: Query<(Entity, &Transform, &Collider, &ATP), (With<ATP>, Without<Player>)>,
    mut player_query: Query<(&Transform, &Collider, &mut ATP), With<Player>>,
    mut game_score: ResMut<GameScore>,
) {
    if let Ok((player_transform, player_collider, mut player_atp)) = player_query.single_mut() {
        for (atp_entity, atp_transform, atp_collider, atp_component) in atp_query.iter() {
            let distance = player_transform.translation.distance(atp_transform.translation);
            if distance < player_collider.radius + atp_collider.radius {
                // Collect ATP with organic absorption effect
                player_atp.amount += atp_component.amount;
                game_score.current += atp_component.amount * 10; // ATP also gives points
                commands.entity(atp_entity).despawn();
            }
        }
    }
}

// Spawn ATP on enemy death with biological considerations
pub fn spawn_atp_on_death(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in explosion_events.read() {
            if let Some(enemy_type) = &event.enemy_type {
                let (atp_amount, spawn_chance, particle_count) = match enemy_type {
                    EnemyType::ViralParticle => (1, 0.7, 3),
                    EnemyType::AggressiveBacteria => (2, 0.8, 5),
                    EnemyType::ParasiticProtozoa => (5, 0.9, 8),
                    EnemyType::InfectedMacrophage => (25, 1.0, 15),
                    EnemyType::SuicidalSpore => (3, 0.6, 4),
                    EnemyType::BiofilmColony => (8, 0.8, 10),
                    EnemyType::SwarmCell => (4, 0.7, 6),
                    EnemyType::ReproductiveVesicle => (15, 0.9, 12),
                    EnemyType::Offspring => (1, 0.5, 2),
                };
                
                // Random chance to drop ATP based on organism energy content
                if (event.position.x * 123.456).sin().abs() < spawn_chance {
                    // Main ATP drop
                    commands.spawn((
                        Sprite {
                            image: assets.multiplier_powerup_texture.clone(),
                            color: Color::srgb(1.0, 1.0, 0.3), // Golden energy color
                            custom_size: Some(Vec2::splat(18.0)),
                            ..default()
                        },
                        Transform::from_translation(event.position),
                        ATP { amount: atp_amount },
                        Collider { radius: 9.0 },
                        BioluminescentParticle {
                            base_color: Color::srgb(1.0, 1.0, 0.3),
                            pulse_frequency: 3.0,
                            pulse_intensity: 0.6,
                            organic_motion: OrganicMotion {
                                undulation_speed: 2.0,
                                response_to_current: 0.4,
                            },
                        },
                    ));
                    
                    // Spawn smaller ATP particles for organic feel
                    for i in 0..particle_count {
                        let angle = (i as f32 / particle_count as f32) * std::f32::consts::TAU;
                        let offset = Vec2::from_angle(angle) * 20.0;
                        
                        commands.spawn((
                            Sprite {
                                image: assets.particle_texture.clone(),
                                color: Color::srgba(1.0, 1.0, 0.5, 0.8),
                                custom_size: Some(Vec2::splat(4.0)),
                                ..default()
                            },
                            Transform::from_translation(event.position + offset.extend(0.0)),
                            Particle {
                                velocity: offset * 2.0,
                                lifetime: 0.0,
                                max_lifetime: 1.5,
                                size: 4.0,
                                fade_rate: 1.0,
                                bioluminescent: true,
                                drift_pattern: DriftPattern::Floating,
                            },
                            BioluminescentParticle {
                                base_color: Color::srgb(1.0, 1.0, 0.5),
                                pulse_frequency: 4.0,
                                pulse_intensity: 0.4,
                                organic_motion: OrganicMotion {
                                    undulation_speed: 3.0,
                                    response_to_current: 0.8,
                                },
                            },
                        ));
                    }
                }
            }
        }
    }
}

// Move ATP with organic floating motion
pub fn move_atp(
    mut atp_query: Query<(&mut Transform, &mut BioluminescentParticle), With<ATP>>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (mut transform, mut bio_particle) in atp_query.iter_mut() {
        // Gentle downward drift
        transform.translation.y -= 80.0 * time.delta_secs();
        
        // Organic floating animation
        let bob_phase = time.elapsed_secs() * bio_particle.pulse_frequency + transform.translation.x * 0.01;
        let bob_amplitude = 15.0;
        transform.translation.y += bob_phase.sin() * bob_amplitude * time.delta_secs();
        
        // Horizontal drift based on position
        let drift = (time.elapsed_secs() * 1.5 + transform.translation.y * 0.005).sin() * 30.0;
        transform.translation.x += drift * time.delta_secs();
        
        // Respond to fluid currents
        let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
        let current = sample_current(&fluid_environment, grid_pos);
        transform.translation += (current * bio_particle.organic_motion.response_to_current).extend(0.0) * time.delta_secs();
        
        // Organic rotation
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 1.5);
    }
}

// Evolution Chamber interaction (renamed from upgrade_station_interaction)
pub fn evolution_chamber_interaction(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    chamber_query: Query<&Transform, With<EvolutionChamber>>,
    mut player_query: Query<(&Transform, &mut ATP, &mut EvolutionSystem, &mut CellularUpgrades), With<Player>>,
) {
    if let Ok((player_transform, mut atp, mut evolution_system, mut upgrades)) = player_query.single_mut() {
        for chamber_transform in chamber_query.iter() {
            let distance = player_transform.translation.distance(chamber_transform.translation);
            if distance < 60.0 {
                // Player is near evolution chamber
                if keyboard.just_pressed(KeyCode::Digit1) && atp.amount >= 10 {
                    // Enhance cellular damage output
                    atp.amount -= 10;
                    upgrades.damage_amplification *= 1.2;
                    evolution_system.cellular_adaptations.membrane_permeability *= 1.2;
                }
                
                if keyboard.just_pressed(KeyCode::Digit2) && atp.amount >= 15 {
                    // Enhance metabolic rate
                    atp.amount -= 15;
                    upgrades.metabolic_rate *= 1.3;
                    evolution_system.cellular_adaptations.metabolic_efficiency *= 1.3;
                }
                
                if keyboard.just_pressed(KeyCode::Digit3) && atp.amount >= 20 {
                    // Strengthen cellular integrity
                    atp.amount -= 20;
                    upgrades.max_health += 25;
                }
                
                if keyboard.just_pressed(KeyCode::Digit4) && atp.amount >= 25 {
                    // Develop enzyme production
                    atp.amount -= 25;
                    evolution_system.cellular_adaptations.extremophile_traits = true;
                }
                
                if keyboard.just_pressed(KeyCode::Digit5) && atp.amount >= 30 {
                    // Enhance bioluminescence
                    atp.amount -= 30;
                    evolution_system.cellular_adaptations.biofilm_formation = true;
                }
                
                if keyboard.just_pressed(KeyCode::Digit6) && atp.amount >= 20 {
                    // Develop emergency spore
                    atp.amount -= 20;
                    evolution_system.emergency_spores += 1;
                }
                
                if keyboard.just_pressed(KeyCode::Digit7) && atp.amount >= 50 {
                    // Evolve to Pseudopod Network
                    atp.amount -= 50;
                    evolution_system.primary_evolution = EvolutionType::PseudopodNetwork {
                        damage: 8,
                        fire_rate: 0.15,
                        tendril_count: 5,
                        spread_angle: 0.6,
                    };
                }
                
                if keyboard.just_pressed(KeyCode::Digit8) && atp.amount >= 75 {
                    // Evolve to Symbiotic Hunters
                    atp.amount -= 75;
                    evolution_system.primary_evolution = EvolutionType::SymbioticHunters {
                        damage: 25,
                        fire_rate: 0.8,
                        homing_strength: 2.0,
                        blast_radius: 50.0,
                    };
                }
                
                if keyboard.just_pressed(KeyCode::Digit9) && atp.amount >= 100 {
                    // Evolve to Bioluminescent Beam
                    atp.amount -= 100;
                    evolution_system.primary_evolution = EvolutionType::BioluminescentBeam {
                        damage: 15,
                        charge_time: 1.0,
                        duration: 2.0,
                        width: 20.0,
                    };
                }
            }
        }
    }
}

// Evolution power-up collection
pub fn evolution_powerup_collection(
    mut commands: Commands,
    powerup_query: Query<(Entity, &Transform, &Collider, &EvolutionPowerUp)>,
    mut player_query: Query<(Entity, &Transform, &Collider, &mut EvolutionSystem), With<Player>>,
) {
    if let Ok((player_entity, player_transform, player_collider, mut evolution_system)) = player_query.single_mut() {
        for (powerup_entity, powerup_transform, powerup_collider, evolution_powerup) in powerup_query.iter() {
            let distance = player_transform.translation.distance(powerup_transform.translation);
            if distance < player_collider.radius + powerup_collider.radius {
                match &evolution_powerup.adaptation_type {
                    AdaptationType::MetabolicBoost(multiplier) => {
                        evolution_system.cellular_adaptations.metabolic_efficiency *= multiplier;
                        if evolution_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryMetabolicBoost {
                                timer: evolution_powerup.duration.unwrap_or(10.0),
                                multiplier: *multiplier,
                            });
                        }
                    }
                    
                    AdaptationType::CellularDivisionRate(multiplier) => {
                        evolution_system.cellular_adaptations.membrane_permeability *= multiplier;
                        if evolution_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryCellularDivision {
                                timer: evolution_powerup.duration.unwrap_or(10.0),
                                multiplier: *multiplier,
                            });
                        }
                    }
                    
                    AdaptationType::EnzymeProduction => {
                        evolution_system.cellular_adaptations.extremophile_traits = true;
                        if evolution_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryEnzymeProduction {
                                timer: evolution_powerup.duration.unwrap_or(15.0),
                            });
                        }
                    }
                    
                    AdaptationType::Bioluminescence => {
                        evolution_system.cellular_adaptations.biofilm_formation = true;
                        if evolution_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryBioluminescence {
                                timer: evolution_powerup.duration.unwrap_or(15.0),
                            });
                        }
                    }
                    
                    AdaptationType::ChemicalResistance => {
                        evolution_system.cellular_adaptations.chemoreceptor_sensitivity *= 0.5; // Less sensitive = more resistant
                        if evolution_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryChemicalResistance {
                                timer: evolution_powerup.duration.unwrap_or(20.0),
                            });
                        }
                    }
                    
                    AdaptationType::EvolutionSwap(new_evolution) => {
                        evolution_system.secondary_evolution = Some(evolution_system.primary_evolution.clone());
                        evolution_system.primary_evolution = new_evolution.clone();
                        if evolution_powerup.temporary {
                            commands.entity(player_entity).insert(TemporaryEvolutionSwap {
                                timer: evolution_powerup.duration.unwrap_or(25.0),
                                original_evolution: evolution_system.secondary_evolution.clone().unwrap(),
                            });
                        }
                    }
                }
                
                commands.entity(powerup_entity).despawn();
            }
        }
    }
}

// Update temporary evolution effects
pub fn update_temporary_evolution_effects(
    mut commands: Commands,
    mut metabolic_boost_query: Query<(Entity, &mut TemporaryMetabolicBoost)>,
    mut cellular_division_query: Query<(Entity, &mut TemporaryCellularDivision)>,
    mut enzyme_production_query: Query<(Entity, &mut TemporaryEnzymeProduction)>,
    mut bioluminescence_query: Query<(Entity, &mut TemporaryBioluminescence)>,
    mut chemical_resistance_query: Query<(Entity, &mut TemporaryChemicalResistance)>,
    mut evolution_swap_query: Query<(Entity, &mut TemporaryEvolutionSwap)>,
    mut player_query: Query<&mut EvolutionSystem, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut evolution_system) = player_query.single_mut() {
        // Update metabolic boost
        for (entity, mut boost) in metabolic_boost_query.iter_mut() {
            boost.timer -= time.delta_secs();
            if boost.timer <= 0.0 {
                evolution_system.cellular_adaptations.metabolic_efficiency /= boost.multiplier;
                commands.entity(entity).remove::<TemporaryMetabolicBoost>();
            }
        }
        
        // Update cellular division rate
        for (entity, mut division) in cellular_division_query.iter_mut() {
            division.timer -= time.delta_secs();
            if division.timer <= 0.0 {
                evolution_system.cellular_adaptations.membrane_permeability /= division.multiplier;
                commands.entity(entity).remove::<TemporaryCellularDivision>();
            }
        }
        
        // Update enzyme production
        for (entity, mut enzyme) in enzyme_production_query.iter_mut() {
            enzyme.timer -= time.delta_secs();
            if enzyme.timer <= 0.0 {
                evolution_system.cellular_adaptations.extremophile_traits = false;
                commands.entity(entity).remove::<TemporaryEnzymeProduction>();
            }
        }
        
        // Update bioluminescence
        for (entity, mut biolum) in bioluminescence_query.iter_mut() {
            biolum.timer -= time.delta_secs();
            if biolum.timer <= 0.0 {
                evolution_system.cellular_adaptations.biofilm_formation = false;
                commands.entity(entity).remove::<TemporaryBioluminescence>();
            }
        }
        
        // Update chemical resistance
        for (entity, mut resistance) in chemical_resistance_query.iter_mut() {
            resistance.timer -= time.delta_secs();
            if resistance.timer <= 0.0 {
                evolution_system.cellular_adaptations.chemoreceptor_sensitivity *= 2.0; // Restore sensitivity
                commands.entity(entity).remove::<TemporaryChemicalResistance>();
            }
        }
        
        // Update evolution swap
        for (entity, mut swap) in evolution_swap_query.iter_mut() {
            swap.timer -= time.delta_secs();
            if swap.timer <= 0.0 {
                evolution_system.primary_evolution = swap.original_evolution.clone();
                evolution_system.secondary_evolution = None;
                commands.entity(entity).remove::<TemporaryEvolutionSwap>();
            }
        }
    }
}

// Spawn evolution power-ups with biological themes
pub fn spawn_evolution_powerups(
    mut commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        // Spawn evolution power-ups less frequently than regular power-ups
        if enemy_spawner.wave_timer % 30.0 < 0.1 && enemy_spawner.wave_timer > 0.1 {
            let x_position = (time.elapsed_secs() * 25.0).sin() * 280.0;
            
            let (adaptation_type, texture, color, temporary) = match (time.elapsed_secs() as u32 / 30) % 6 {
                0 => (
                    AdaptationType::MetabolicBoost(1.5), 
                    assets.health_powerup_texture.clone(), 
                    Color::srgb(0.8, 1.0, 0.6), // Healthy green
                    true
                ),
                1 => (
                    AdaptationType::CellularDivisionRate(1.8), 
                    assets.rapidfire_powerup_texture.clone(), 
                    Color::srgb(0.6, 0.9, 1.0), // Cellular blue
                    true
                ),
                2 => (
                    AdaptationType::EnzymeProduction, 
                    assets.shield_powerup_texture.clone(), 
                    Color::srgb(0.9, 0.7, 1.0), // Enzyme purple
                    true
                ),
                3 => (
                    AdaptationType::Bioluminescence, 
                    assets.multiplier_powerup_texture.clone(), 
                    Color::srgb(1.0, 1.0, 0.7), // Bioluminescent yellow
                    true
                ),
                4 => (
                    AdaptationType::EvolutionSwap(EvolutionType::PseudopodNetwork {
                        damage: 12, fire_rate: 0.12, tendril_count: 7, spread_angle: 0.8
                    }), 
                    assets.speed_powerup_texture.clone(), 
                    Color::srgb(1.0, 0.8, 0.6), // Organic orange
                    true
                ),
                _ => (
                    AdaptationType::EvolutionSwap(EvolutionType::EnzymeBurst {
                        damage: 6, fire_rate: 0.05, acid_damage: 3.0
                    }), 
                    assets.explosion_texture.clone(), 
                    Color::srgb(0.8, 1.0, 0.8), // Chemical green
                    true
                ),
            };
            
            commands.spawn((
                Sprite {
                    image: texture,
                    color,
                    ..default()
                },
                Transform::from_xyz(x_position, 400.0, 0.0),
                EvolutionPowerUp {
                    evolution_type: EvolutionType::CytoplasmicSpray { damage: 10, fire_rate: 0.1 },
                    adaptation_type,
                    temporary,
                    duration: Some(20.0),
                },
                Collider { radius: 14.0 },
                BioluminescentParticle {
                    base_color: color,
                    pulse_frequency: 2.0,
                    pulse_intensity: 0.5,
                    organic_motion: OrganicMotion {
                        undulation_speed: 1.5,
                        response_to_current: 0.6,
                    },
                },
            ));
        }
    }
}

// Spawn biological power-ups with enhanced organic effects
pub fn spawn_biological_powerups(
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnPowerUp>,
    time: Res<Time>,
) {
    enemy_spawner.powerup_timer -= time.delta_secs();
    
    if enemy_spawner.powerup_timer <= 0.0 {
        let x_position = (time.elapsed_secs() * 40.0).sin() * 320.0;
        
        let power_type = match (time.elapsed_secs() as u32 / 18) % 9 {
            0 => PowerUpType::CellularRegeneration { amount: 30 },
            1 => PowerUpType::CellWall { duration: 12.0 },
            2 => PowerUpType::Flagella { multiplier: 1.6, duration: 10.0 },
            3 => PowerUpType::SymbioticBoost { multiplier: 2.2, duration: 18.0 },
            4 => PowerUpType::MitochondriaOvercharge { rate_multiplier: 2.5, duration: 12.0 },
            5 => PowerUpType::Photosynthesis { energy_regen: 8.0, duration: 15.0 },
            6 => PowerUpType::Chemotaxis { homing_strength: 2.5, duration: 12.0 },
            7 => PowerUpType::Osmoregulation { immunity_duration: 10.0 },
            _ => PowerUpType::BinaryFission { clone_duration: 20.0 },
        };
        
        spawn_events.write(SpawnPowerUp {
            position: Vec3::new(x_position, 420.0, 0.0),
            power_type,
        });
        
        enemy_spawner.powerup_timer = 15.0; // Slightly more frequent for biological variety
    }
}

// Enhanced powerup collection with biological effects
pub fn handle_biological_powerup_collection(
    mut commands: Commands,
    powerup_query: Query<(Entity, &Transform, &Collider, &PowerUp)>,
    mut player_query: Query<(Entity, &Transform, &Collider, &mut Health), With<Player>>,
    mut particle_events: EventWriter<SpawnParticles>,
    assets: Option<Res<GameAssets>>,
) {
    if let Ok((player_entity, player_transform, player_collider, mut player_health)) = player_query.single_mut() {
        for (powerup_entity, powerup_transform, powerup_collider, powerup) in powerup_query.iter() {
            let distance = player_transform.translation.distance(powerup_transform.translation);
            if distance < player_collider.radius + powerup_collider.radius {
                
                // Spawn organic absorption particles
                particle_events.write(SpawnParticles {
                    position: powerup_transform.translation,
                    count: 12,
                    config: ParticleConfig {
                        color_start: Color::srgb(0.4, 1.0, 0.8),
                        color_end: Color::srgba(0.2, 0.8, 1.0, 0.0),
                        velocity_range: (Vec2::new(-60.0, -60.0), Vec2::new(60.0, 60.0)),
                        lifetime_range: (0.5, 1.2),
                        size_range: (0.30, 0.80),
                        gravity: Vec2::new(0.0, -20.0),
                        organic_motion: true,
                        bioluminescence: 1.0,
                    },
                });
                
                match &powerup.power_type {
                    PowerUpType::CellularRegeneration { amount } => {
                        player_health.0 = (player_health.0 + amount).min(100);
                        
                        // Spawn healing particles
                        if let Some(assets) = &assets {
                            for i in 0..8 {
                                let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                                let offset = Vec2::from_angle(angle) * 25.0;
                                
                                commands.spawn((
                                    Sprite {
                                        image: assets.particle_texture.clone(),
                                        color: Color::srgba(0.4, 1.0, 0.6, 0.8),
                                        custom_size: Some(Vec2::splat(4.0)),
                                        ..default()
                                    },
                                    Transform::from_translation(player_transform.translation + offset.extend(0.0)),
                                    BioluminescentParticle {
                                        base_color: Color::srgb(0.4, 1.0, 0.6),
                                        pulse_frequency: 3.0,
                                        pulse_intensity: 0.7,
                                        organic_motion: OrganicMotion {
                                            undulation_speed: 2.0,
                                            response_to_current: 0.5,
                                        },
                                    },
                                    Particle {
                                        velocity: offset * 0.5,
                                        lifetime: 0.0,
                                        max_lifetime: 2.0,
                                        size: 4.0,
                                        fade_rate: 1.0,
                                        bioluminescent: true,
                                        drift_pattern: DriftPattern::Floating,
                                    },
                                ));
                            }
                        }
                    }
                    
                    PowerUpType::CellWall { duration } => {
                        commands.entity(player_entity).insert(CellWallReinforcement {
                            timer: *duration,
                            alpha_timer: 0.0,
                        });
                        
                        if let Some(assets) = &assets {
                            // Spawn cell wall visual effect
                            commands.spawn((
                                Sprite {
                                    image: assets.barrier_texture.clone(),
                                    color: Color::srgba(0.4, 1.0, 0.8, 0.4),
                                    custom_size: Some(Vec2::splat(70.0)),
                                    ..default()
                                },
                                Transform::from_translation(player_transform.translation),
                                CellWallVisual,
                                BioluminescentParticle {
                                    base_color: Color::srgb(0.4, 1.0, 0.8),
                                    pulse_frequency: 2.0,
                                    pulse_intensity: 0.5,
                                    organic_motion: OrganicMotion {
                                        undulation_speed: 1.0,
                                        response_to_current: 0.1,
                                    },
                                },
                            ));
                        }
                    }
                    
                    PowerUpType::Flagella { multiplier, duration } => {
                        commands.entity(player_entity).insert(FlagellaBoost {
                            timer: *duration,
                            multiplier: *multiplier,
                        });
                    }
                    
                    PowerUpType::SymbioticBoost { multiplier, duration } => {
                        commands.entity(player_entity).insert(SymbioticMultiplier {
                            timer: *duration,
                            multiplier: *multiplier,
                        });
                    }
                    
                    PowerUpType::MitochondriaOvercharge { rate_multiplier, duration } => {
                        commands.entity(player_entity).insert(MitochondriaOvercharge {
                            timer: *duration,
                            rate_multiplier: *rate_multiplier,
                        });
                    }
                    
                    PowerUpType::Photosynthesis { energy_regen, duration } => {
                        commands.entity(player_entity).insert(PhotosynthesisActive {
                            timer: *duration,
                            energy_per_second: *energy_regen,
                        });
                    }
                    
                    PowerUpType::Chemotaxis { homing_strength, duration } => {
                        commands.entity(player_entity).insert(ChemotaxisActive {
                            timer: *duration,
                            homing_strength: *homing_strength,
                        });
                    }
                    
                    PowerUpType::Osmoregulation { immunity_duration } => {
                        commands.entity(player_entity).insert(OsmoregulationActive {
                            timer: *immunity_duration,
                        });
                    }
                    
                    PowerUpType::BinaryFission { clone_duration } => {
                        commands.entity(player_entity).insert(BinaryFissionActive {
                            timer: *clone_duration,
                            clone_timer: 0.5,
                        });
                    }
                }
                
                commands.entity(powerup_entity).despawn();
            }
        }
    }
}

// Enhanced organic power-up movement
pub fn move_biological_powerups(
    mut powerup_query: Query<(&mut Transform, &mut PowerUp, Option<&BioluminescentParticle>)>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (mut transform, mut powerup, bio_particle) in powerup_query.iter_mut() {
        // Slow downward movement
        transform.translation.y -= 120.0 * time.delta_secs();
        
        // Organic bobbing animation
        powerup.bob_timer += time.delta_secs() * 2.5;
        let bob_amplitude = if bio_particle.is_some() { 20.0 } else { 15.0 };
        transform.translation.y += powerup.bob_timer.sin() * bob_amplitude * time.delta_secs();
        
        // Horizontal drift based on biological type
        let drift_frequency = 1.5 + powerup.bioluminescent_pulse * 0.5;
        let drift = (time.elapsed_secs() * drift_frequency + transform.translation.x * 0.005).sin() * 40.0;
        transform.translation.x += drift * time.delta_secs();
        
        // Respond to fluid currents
        let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
        let current = sample_current(&fluid_environment, grid_pos);
        transform.translation += (current * 0.6).extend(0.0) * time.delta_secs();
        
        // Organic rotation with variation
        let rotation_speed = 1.8 + (powerup.bioluminescent_pulse * 0.5);
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * rotation_speed);
        
        // Update bioluminescent pulse for organic glow
        powerup.bioluminescent_pulse += time.delta_secs() * 2.0;
    }
}

// Enhanced ATP collection with organic energy transfer
pub fn collect_atp_with_energy_transfer(
    mut commands: Commands,
    atp_query: Query<(Entity, &Transform, &Collider, &ATP), Without<Player>>,
    mut player_query: Query<(&Transform, &Collider, &mut ATP), With<Player>>,
    mut particle_events: EventWriter<SpawnParticles>,
    mut game_score: ResMut<GameScore>,
) {
    if let Ok((player_transform, player_collider, mut player_atp)) = player_query.single_mut() {
        for (atp_entity, atp_transform, atp_collider, atp_component) in atp_query.iter() {
            let distance = player_transform.translation.distance(atp_transform.translation);
            
            // Extended collection range for more organic feel
            if distance < (player_collider.radius + atp_collider.radius) * 1.5 {
                // Energy transfer particle stream
                particle_events.write(SpawnParticles {
                    position: atp_transform.translation,
                    count: 8,
                    config: ParticleConfig {
                        color_start: Color::srgb(1.0, 1.0, 0.4),
                        color_end: Color::srgba(1.0, 0.8, 0.2, 0.0),
                        velocity_range: (
                            (player_transform.translation.truncate() - atp_transform.translation.truncate()) * 2.0,
                            (player_transform.translation.truncate() - atp_transform.translation.truncate()) * 3.0
                        ),
                        lifetime_range: (0.3, 0.8),
                        size_range: (0.20, 0.50),
                        gravity: Vec2::ZERO,
                        organic_motion: true,
                        bioluminescence: 1.0,
                    },
                });
                
                // Collect ATP
                player_atp.amount += atp_component.amount;
                game_score.current += atp_component.amount * 12; // Slightly higher points for biological energy
                
                commands.entity(atp_entity).despawn();
            }
        }
    }
}

// Spawn evolution chambers with organic growth patterns
pub fn spawn_evolution_chambers(
    mut commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        // Spawn evolution chamber every 75 seconds
        if enemy_spawner.wave_timer % 75.0 < 0.1 && enemy_spawner.wave_timer > 45.0 {
            // Main chamber structure
            let _chamber_entity = commands.spawn((
                Sprite {
                    image: assets.enemy_texture.clone(),
                    color: Color::srgb(0.3, 0.9, 0.6), // Organic chamber green
                    custom_size: Some(Vec2::splat(60.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 380.0, 0.0),
                EvolutionChamber,
                BioluminescentParticle {
                    base_color: Color::srgb(0.3, 0.9, 0.6),
                    pulse_frequency: 1.0,
                    pulse_intensity: 0.6,
                    organic_motion: OrganicMotion {
                        undulation_speed: 0.8,
                        response_to_current: 0.2,
                    },
                },
            )).id();
            
            // Spawn organic tendrils around the chamber
            for i in 0..6 {
                let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
                let radius = 40.0;
                let tendril_pos = Vec2::from_angle(angle) * radius;
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgba(0.4, 0.8, 0.5, 0.7),
                        custom_size: Some(Vec2::splat(8.0)),
                        ..default()
                    },
                    Transform::from_xyz(tendril_pos.x, 380.0 + tendril_pos.y, -0.1),
                    BioluminescentParticle {
                        base_color: Color::srgb(0.4, 0.8, 0.5),
                        pulse_frequency: 2.0 + i as f32 * 0.3,
                        pulse_intensity: 0.4,
                        organic_motion: OrganicMotion {
                            undulation_speed: 3.0,
                            response_to_current: 0.8,
                        },
                    },
                    Particle {
                        velocity: Vec2::ZERO,
                        lifetime: 0.0,
                        max_lifetime: 60.0, // Last until chamber despawns
                        size: 8.0,
                        fade_rate: 1.0,
                        bioluminescent: true,
                        drift_pattern: DriftPattern::Floating,
                    },
                ));
            }
        }
    }
}

// Update evolution chamber UI with biological terminology
pub fn update_evolution_ui(
    mut commands: Commands,
    chamber_query: Query<&Transform, With<EvolutionChamber>>,
    player_query: Query<(&Transform, &ATP), With<Player>>,
    existing_ui_query: Query<Entity, With<EvolutionUI>>,
) {
    if let Ok((player_transform, atp)) = player_query.single() {
        let near_chamber = chamber_query.iter().any(|chamber_transform| {
            player_transform.translation.distance(chamber_transform.translation) < 60.0
        });
        
        if near_chamber {
            // Show evolution UI if not already showing
            if existing_ui_query.is_empty() {
                spawn_evolution_ui(&mut commands, atp.amount);
            }
        } else {
            // Hide evolution UI if showing
            for entity in existing_ui_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_evolution_ui(commands: &mut Commands, atp_amount: u32) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(120.0),
            width: Val::Px(350.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.2, 0.15, 0.9)),
        BorderColor(Color::srgb(0.3, 0.8, 0.6)),
        EvolutionUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("🧬 EVOLUTION CHAMBER"),
            TextFont { font_size: 22.0, ..default() },
            TextColor(Color::srgb(0.3, 1.0, 0.7)),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new(format!("ATP Available: {}⚡", atp_amount)),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 0.4)),
            Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() },
        ));
        
        let evolutions = [
            ("1 - Membrane Reinforcement +20% (10 ATP)", 10, "🧱 Enhanced cellular damage output"),
            ("2 - Metabolic Enhancement +30% (15 ATP)", 15, "⚡ Faster movement and energy production"),
            ("3 - Cellular Integrity +25 HP (20 ATP)", 20, "❤️ Increased maximum health capacity"),
            ("4 - Enzyme Production (25 ATP)", 25, "🧪 Resistance to environmental toxins"),
            ("5 - Bioluminescence (30 ATP)", 30, "💡 Enhanced visibility and coordination"),
            ("6 - Emergency Spore +1 (20 ATP)", 20, "💥 Additional emergency reproduction"),
            ("7 - Pseudopod Network (50 ATP)", 50, "🕷️ Multi-directional organic tendrils"),
            ("8 - Symbiotic Hunters (75 ATP)", 75, "🎯 Self-guided cooperative organisms"),
            ("9 - Bioluminescent Beam (100 ATP)", 100, "🌟 Concentrated energy discharge"),
        ];
        
        for (text, cost, description) in evolutions {
            let color = if atp_amount >= cost {
                Color::srgb(0.9, 1.0, 0.9)
            } else {
                Color::srgb(0.5, 0.6, 0.5)
            };
            
            parent.spawn((
                Text::new(text),
                TextFont { font_size: 14.0, ..default() },
                TextColor(color),
                Node { margin: UiRect::bottom(Val::Px(3.0)), ..default() },
            ));
            
            parent.spawn((
                Text::new(description),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.7, 0.8, 0.7)),
                Node { 
                    margin: UiRect::bottom(Val::Px(8.0)),
                    // margin_left: Val::Px(15.0),
                    ..default() 
                },
            ));
        }
        
        parent.spawn((
            Text::new("💡 Tip: Adaptations help you survive in different chemical environments"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.6, 0.9, 0.8)),
            Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
        ));
    });
}

// Helper functions from biological_systems.rs
fn world_to_grid_pos(world_pos: Vec2, fluid_env: &FluidEnvironment) -> (usize, usize) {
    let grid_x = ((world_pos.x + 640.0) / fluid_env.cell_size).clamp(0.0, (fluid_env.grid_size - 1) as f32) as usize;
    let grid_y = ((world_pos.y + 360.0) / fluid_env.cell_size).clamp(0.0, (fluid_env.grid_size - 1) as f32) as usize;
    (grid_x, grid_y)
}

fn sample_current(fluid_env: &FluidEnvironment, grid_pos: (usize, usize)) -> Vec2 {
    let index = grid_pos.1 * fluid_env.grid_size + grid_pos.0;
    if index < fluid_env.current_field.len() {
        fluid_env.current_field[index]
    } else {
        Vec2::ZERO
    }
}
