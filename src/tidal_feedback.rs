use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use std::f32::consts::TAU;
use bevy::audio::*;

#[derive(Component)]
pub struct TidalMovementIndicator {
    pub direction: Vec2,
    pub intensity: f32,
    pub indicator_type: TidalIndicatorType,
    pub lifetime: f32,
    pub pulse_frequency: f32,
}

#[derive(Clone)]
pub enum TidalIndicatorType {
    CurrentFlow { strength: f32 },
    KingTideWarning { countdown: f32 },
    ThermalVentActivity { heat_level: f32 },
    ChemicalGradient { ph_change: f32 },
    EcosystemStress { damage_rate: f32 },
}

#[derive(Component)]
pub struct FluidMotionVisualizer {
    pub grid_position: Vec2,
    pub flow_strength: f32,
    pub flow_direction: Vec2,
    pub turbulence: f32,
    pub last_update: f32,
}

#[derive(Component)]
pub struct TidalWaveEffect {
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_phase: f32,
    pub propagation_distance: f32,
    pub max_distance: f32,
}

#[derive(Resource)]
pub struct TidalFeedbackSystem {
    pub movement_indicators: Vec<Entity>,
    pub flow_visualizers: Vec<Entity>,
    pub feedback_intensity: f32,
    pub visual_feedback_enabled: bool,
    pub audio_feedback_enabled: bool,
    pub last_major_tide_event: f32,
}

impl Default for TidalFeedbackSystem {
    fn default() -> Self {
        Self {
            movement_indicators: Vec::new(),
            flow_visualizers: Vec::new(),
            feedback_intensity: 1.0,
            visual_feedback_enabled: true,
            audio_feedback_enabled: true,
            last_major_tide_event: 0.0,
        }
    }
}

// Enhanced tidal feedback system
pub fn enhanced_tidal_feedback_system(
    mut commands: Commands,
    mut tidal_feedback: ResMut<TidalFeedbackSystem>,
    tidal_physics: Res<TidalPoolPhysics>,
    fluid_environment: Res<FluidEnvironment>,
    current_generator: Res<CurrentGenerator>,
    ecosystem: Res<EcosystemState>,
    chemical_environment: Res<ChemicalEnvironment>,
    player_query: Query<&Transform, With<Player>>,
    mut existing_indicators: Query<(Entity, &mut TidalMovementIndicator, &mut Transform, &mut Sprite), Without<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if !tidal_feedback.visual_feedback_enabled { return; }
    if let (Ok(player_transform), Some(assets)) = (player_query.single(), assets) {
        
        // Update existing indicators
        update_existing_tidal_indicators(
            &mut commands,
            &mut existing_indicators,
            &tidal_physics,
            time.delta_secs(),
        );
        
        // Generate new indicators based on tidal conditions
        if tidal_physics.king_tide_active && time.elapsed_secs() - tidal_feedback.last_major_tide_event > 1.0 {
            spawn_king_tide_warnings(
                &mut commands,
                &mut tidal_feedback,
                &assets,
                player_transform.translation,
                tidal_physics.king_tide_intensity,
                time.elapsed_secs(),
            );
        }
        
        // Current flow indicators
        spawn_current_flow_indicators(
            &mut commands,
            &assets,
            &fluid_environment,
            &current_generator,
            player_transform.translation,
            time.elapsed_secs(),
        );
        
        // Thermal vent activity feedback
        spawn_thermal_activity_indicators(
            &mut commands,
            &assets,
            &current_generator,
            player_transform.translation,
            time.elapsed_secs(),
        );
        
        // Chemical gradient visualization
        spawn_chemical_gradient_indicators(
            &mut commands,
            &assets,
            &chemical_environment,
            player_transform.translation,
            time.elapsed_secs(),
        );
        
        // Ecosystem stress visualization
        if ecosystem.health < 0.5 {
            spawn_ecosystem_stress_indicators(
                &mut commands,
                &assets,
                &ecosystem,
                player_transform.translation,
                time.elapsed_secs(),
            );
        }
        
        // Generate fluid motion visualizers around player
        generate_fluid_motion_visualizers(
            &mut commands,
            &mut tidal_feedback,
            &assets,
            &fluid_environment,
            player_transform.translation,
            time.elapsed_secs(),
        );
    }
}

fn update_existing_tidal_indicators(
    commands: &mut Commands,
    indicators: &mut Query<(Entity, &mut TidalMovementIndicator, &mut Transform, &mut Sprite), Without<Player>>,
    tidal_physics: &TidalPoolPhysics,
    delta_time: f32,
) {
    for (entity, mut indicator, mut transform, mut sprite) in indicators.iter_mut() {
        indicator.lifetime -= delta_time;
        
        if indicator.lifetime <= 0.0 {
            commands.entity(entity).insert(AlreadyDespawned).despawn();
            continue;
        }
        
        // Update indicator based on type
        match &mut indicator.indicator_type {
            TidalIndicatorType::CurrentFlow { strength } => {
                // Animate current flow
                let pulse = (indicator.lifetime * indicator.pulse_frequency).sin() * 0.5 + 0.5;
                let flow_intensity = *strength * pulse;
                
                // Move in current direction
                transform.translation += indicator.direction.extend(0.0) * flow_intensity * 20.0 * delta_time;
                
                // Fade over lifetime
                let alpha = indicator.lifetime / 3.0;
                sprite.color.set_alpha(alpha * 0.8);
                
                // Scale based on flow strength
                let scale = 1.0 + flow_intensity * 0.3;
                transform.scale = Vec3::splat(scale);
            }
            
            TidalIndicatorType::KingTideWarning { countdown } => {
                *countdown -= delta_time;
                
                // Pulsing red warning
                let warning_pulse = (*countdown * 6.0).sin() * 0.5 + 0.5;
                sprite.color = Color::srgb(1.0, 0.2 * warning_pulse, 0.2 * warning_pulse);
                
                // Growing scale for urgency
                let urgency_scale = 1.0 + (1.0 - *countdown / 5.0) * 0.5;
                transform.scale = Vec3::splat(urgency_scale);
                
                // Spawn ripple effects
                if (*countdown * 10.0) % 1.0 < 0.1 {
                    // Would spawn ripple particles here
                }
            }
            
            TidalIndicatorType::ThermalVentActivity { heat_level } => {
                // Heat shimmer effect
                let shimmer = (indicator.lifetime * 8.0).sin() * *heat_level * 3.0;
                transform.translation.x += shimmer * delta_time;
                
                // Color shift from orange to white based on heat
                let heat_intensity = heat_level.clamp(0.0, 1.0);
                sprite.color = Color::srgb(
                    1.0,
                    0.6 + heat_intensity * 0.4,
                    0.2 + heat_intensity * 0.6,
                );
            }
            
            TidalIndicatorType::ChemicalGradient { ph_change } => {
                // Color based on pH change
                let ph_color = if *ph_change < 0.0 {
                    Color::srgb(1.0, 0.3, 0.3) // Acidic - red
                } else {
                    Color::srgb(0.3, 0.3, 1.0) // Basic - blue
                };
                
                let intensity = ph_change.abs() * 0.5 + 0.5;
                sprite.color = Color::srgba(
                    ph_color.to_srgba().red,
                    ph_color.to_srgba().green,
                    ph_color.to_srgba().blue,
                    intensity * 0.6,
                );
                
                // Diffusion animation
                let diffusion = 1.0 + (3.0 - indicator.lifetime) * 0.1;
                transform.scale = Vec3::splat(diffusion);
            }
            
            TidalIndicatorType::EcosystemStress { damage_rate } => {
                // Stressed ecosystem particles
                let stress_pulse = (indicator.lifetime * 4.0).sin().abs();
                sprite.color = Color::srgb(0.8, 0.8 * (1.0 - stress_pulse), 0.2);
                
                // Erratic movement for stress
                let stress_movement = Vec2::new(
                    (indicator.lifetime * 12.0).sin() * *damage_rate * 10.0,
                    (indicator.lifetime * 15.0).cos() * *damage_rate * 8.0,
                );
                transform.translation += stress_movement.extend(0.0) * delta_time;
            }
        }
    }
}

fn spawn_king_tide_warnings(
    commands: &mut Commands,
    tidal_feedback: &mut TidalFeedbackSystem,
    assets: &GameAssets,
    player_pos: Vec3,
    intensity: f32,
    current_time: f32,
) {
    tidal_feedback.last_major_tide_event = current_time;
    
    // Spawn warning indicators in a circle around player
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * TAU;
        let radius = 100.0 + intensity * 50.0;
        let position = player_pos + Vec3::new(
            angle.cos() * radius,
            angle.sin() * radius,
            0.5,
        );
        
        commands.spawn((
            Sprite {
                image: assets.particle_texture.clone(),
                color: Color::srgb(1.0, 0.3, 0.3),
                custom_size: Some(Vec2::splat(12.0)),
                ..default()
            },
            Transform::from_translation(position),
            TidalMovementIndicator {
                direction: Vec2::from_angle(angle),
                intensity,
                indicator_type: TidalIndicatorType::KingTideWarning { countdown: 5.0 },
                lifetime: 8.0,
                pulse_frequency: 6.0,
            },
        ));
    }
    
    // Central warning
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgba(1.0, 0.5, 0.5, 0.7),
            custom_size: Some(Vec2::splat(40.0)),
            ..default()
        },
        Transform::from_translation(player_pos + Vec3::new(0.0, 0.0, 0.8)),
        TidalWaveEffect {
            wave_height: intensity * 20.0,
            wave_speed: 200.0,
            wave_phase: 0.0,
            propagation_distance: 0.0,
            max_distance: 400.0,
        },
    ));
}

fn spawn_current_flow_indicators(
    commands: &mut Commands,
    assets: &GameAssets,
    fluid_environment: &FluidEnvironment,
    current_generator: &CurrentGenerator,
    player_pos: Vec3,
    _current_time: f32,
) {
    // Sample current around player and create flow indicators
    let sample_positions = [
        Vec2::new(-80.0, -80.0),
        Vec2::new(80.0, -80.0),
        Vec2::new(-80.0, 80.0),
        Vec2::new(80.0, 80.0),
        Vec2::new(0.0, -120.0),
        Vec2::new(0.0, 120.0),
    ];
    
    for sample_pos in sample_positions {
        let world_pos = player_pos.truncate() + sample_pos;
        let grid_pos = world_to_grid_pos(world_pos, fluid_environment);
        let current = sample_current(fluid_environment, grid_pos);
        
        if current.length() > 20.0 { // Only show significant currents
            commands.spawn((
                Sprite {
                    image: assets.projectile_texture.clone(),
                    color: Color::srgba(0.4, 0.8, 1.0, 0.6),
                    custom_size: Some(Vec2::splat(6.0)),
                    ..default()
                },
                Transform::from_translation((player_pos.truncate() + sample_pos).extend(0.3))
                    .with_rotation(Quat::from_rotation_z(current.to_angle())),
                TidalMovementIndicator {
                    direction: current.normalize_or_zero(),
                    intensity: (current.length() / 100.0).clamp(0.1, 1.0),
                    indicator_type: TidalIndicatorType::CurrentFlow { 
                        strength: current.length() / 100.0 
                    },
                    lifetime: 3.0,
                    pulse_frequency: 3.0,
                },
                BioluminescentParticle {
                    base_color: Color::srgb(0.4, 0.8, 1.0),
                    pulse_frequency: 2.0,
                    pulse_intensity: 0.4,
                    organic_motion: OrganicMotion {
                        undulation_speed: 1.0,
                        response_to_current: 1.0,
                    },
                },
            ));
        }
    }
}

fn spawn_thermal_activity_indicators(
    commands: &mut Commands,
    assets: &GameAssets,
    current_generator: &CurrentGenerator,
    player_pos: Vec3,
    _current_time: f32,
) {
    for vent in &current_generator.thermal_vents {
        if !vent.active { continue; }
        
        let distance = player_pos.truncate().distance(vent.position);
        if distance < 200.0 {
            // Spawn heat indicators around active vents
            for i in 0..4 {
                let angle = (i as f32 / 4.0) * TAU;
                let offset = Vec2::from_angle(angle) * (30.0 + i as f32 * 10.0);
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgb(1.0, 0.7, 0.2),
                        custom_size: Some(Vec2::splat(4.0)),
                        ..default()
                    },
                    Transform::from_translation((vent.position + offset).extend(0.2)),
                    TidalMovementIndicator {
                        direction: offset.normalize(),
                        intensity: vent.strength / 200.0,
                        indicator_type: TidalIndicatorType::ThermalVentActivity { 
                            heat_level: vent.strength / 200.0 
                        },
                        lifetime: 2.5,
                        pulse_frequency: 4.0,
                    },
                ));
            }
        }
    }
}

fn spawn_chemical_gradient_indicators(
    commands: &mut Commands,
    assets: &GameAssets,
    chemical_environment: &ChemicalEnvironment,
    player_pos: Vec3,
    _current_time: f32,
) {
    // pH zone indicators
    for zone in &chemical_environment.ph_zones {
        let distance = player_pos.truncate().distance(zone.position);
        if distance < zone.radius + 100.0 {
            // Show pH gradient with particles
            let ph_deviation = (zone.ph_level - 7.0).abs();
            if ph_deviation > 0.5 {
                for i in 0..6 {
                    let angle = (i as f32 / 6.0) * TAU;
                    let radius = zone.radius * 0.7;
                    let pos = zone.position + Vec2::from_angle(angle) * radius;
                    
                    commands.spawn((
                        Sprite {
                            image: assets.particle_texture.clone(),
                            color: if zone.ph_level < 7.0 {
                                Color::srgb(1.0, 0.4, 0.4) // Acidic
                            } else {
                                Color::srgb(0.4, 0.4, 1.0) // Basic
                            },
                            custom_size: Some(Vec2::splat(8.0)),
                            ..default()
                        },
                        Transform::from_translation(pos.extend(0.1)),
                        TidalMovementIndicator {
                            direction: (pos - zone.position).normalize_or_zero(),
                            intensity: ph_deviation / 3.0,
                            indicator_type: TidalIndicatorType::ChemicalGradient { 
                                ph_change: zone.ph_level - 7.0 
                            },
                            lifetime: 4.0,
                            pulse_frequency: 2.0,
                        },
                        BioluminescentParticle {
                            base_color: if zone.ph_level < 7.0 {
                                Color::srgb(1.0, 0.4, 0.4)
                            } else {
                                Color::srgb(0.4, 0.4, 1.0)
                            },
                            pulse_frequency: 1.5,
                            pulse_intensity: 0.6,
                            organic_motion: OrganicMotion {
                                undulation_speed: 0.8,
                                response_to_current: 0.5,
                            },
                        },
                    ));
                }
            }
        }
    }
    
    // Oxygen zone indicators
    for oxygen_zone in &chemical_environment.oxygen_zones {
        let distance = player_pos.truncate().distance(oxygen_zone.position);
        if distance < oxygen_zone.radius + 80.0 {
            // Show oxygen bubbles for high oxygen zones
            if oxygen_zone.oxygen_level > 0.7 {
                for i in 0..4 {
                    let angle = (i as f32 / 4.0) * TAU + _current_time;
                    let radius = oxygen_zone.radius * 0.5;
                    let pos = oxygen_zone.position + Vec2::from_angle(angle) * radius;
                    
                    commands.spawn((
                        Sprite {
                            image: assets.particle_texture.clone(),
                            color: Color::srgba(0.6, 1.0, 0.8, 0.7),
                            custom_size: Some(Vec2::splat(5.0)),
                            ..default()
                        },
                        Transform::from_translation(pos.extend(0.1)),
                        TidalMovementIndicator {
                            direction: Vec2::new(0.0, 1.0), // Bubbles rise
                            intensity: oxygen_zone.oxygen_level,
                            indicator_type: TidalIndicatorType::ChemicalGradient { 
                                ph_change: 0.0 // Using for oxygen visualization
                            },
                            lifetime: 3.0,
                            pulse_frequency: 1.0,
                        },
                    ));
                }
            }
        }
    }
}

fn spawn_ecosystem_stress_indicators(
    commands: &mut Commands,
    assets: &GameAssets,
    ecosystem: &EcosystemState,
    player_pos: Vec3,
    _current_time: f32,
) {
    let stress_level = 1.0 - ecosystem.health;
    if stress_level > 0.5 {
        // Spawn stress particles around player
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * TAU;
            let radius = 60.0 + stress_level * 40.0;
            let pos = player_pos.truncate() + Vec2::from_angle(angle) * radius;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgb(0.9, 0.7 * (1.0 - stress_level), 0.3),
                    custom_size: Some(Vec2::splat(6.0)),
                    ..default()
                },
                Transform::from_translation(pos.extend(0.2)),
                TidalMovementIndicator {
                    direction: Vec2::from_angle(angle + std::f32::consts::PI), // Inward movement
                    intensity: stress_level,
                    indicator_type: TidalIndicatorType::EcosystemStress { 
                        damage_rate: stress_level * 10.0 
                    },
                    lifetime: 5.0,
                    pulse_frequency: 3.0 + stress_level * 2.0,
                },
            ));
        }
    }
}

fn generate_fluid_motion_visualizers(
    commands: &mut Commands,
    tidal_feedback: &mut TidalFeedbackSystem,
    assets: &GameAssets,
    fluid_environment: &FluidEnvironment,
    player_pos: Vec3,
    current_time: f32,
) {
    // Clean up old visualizers
    tidal_feedback.flow_visualizers.retain(|&entity| {
        // In real implementation, check if entity still exists
        true
    });
    
    // Generate new visualizers if needed
    if tidal_feedback.flow_visualizers.len() < 12 {
        let grid_positions = [
            Vec2::new(-2.0, -2.0), Vec2::new(0.0, -2.0), Vec2::new(2.0, -2.0),
            Vec2::new(-2.0, 0.0),  Vec2::new(0.0, 0.0),  Vec2::new(2.0, 0.0),
            Vec2::new(-2.0, 2.0),  Vec2::new(0.0, 2.0),  Vec2::new(2.0, 2.0),
            Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0), Vec2::new(0.0, 1.0),
        ];
        
        for grid_offset in grid_positions {
            let world_pos = player_pos.truncate() + grid_offset * 40.0;
            let grid_pos = world_to_grid_pos(world_pos, fluid_environment);
            let current = sample_current(fluid_environment, grid_pos);
            
            if current.length() > 10.0 {
                let visualizer = commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgba(0.3, 0.7, 1.0, 0.4),
                        custom_size: Some(Vec2::splat(3.0)),
                        ..default()
                    },
                    Transform::from_translation(world_pos.extend(0.1))
                        .with_rotation(Quat::from_rotation_z(current.to_angle())),
                    FluidMotionVisualizer {
                        grid_position: world_pos,
                        flow_strength: current.length(),
                        flow_direction: current.normalize_or_zero(),
                        turbulence: fluid_environment.turbulence_intensity,
                        last_update: current_time,
                    },
                )).id();
                
                tidal_feedback.flow_visualizers.push(visualizer);
            }
        }
    }
}

// System to update fluid motion visualizers
pub fn update_fluid_motion_visualizers(
    mut commands: Commands,
    mut visualizer_query: Query<(Entity, &mut FluidMotionVisualizer, &mut Transform, &mut Sprite)>,
    fluid_environment: Res<FluidEnvironment>,
    player_query: Query<&Transform, (With<Player>, Without<FluidMotionVisualizer>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (entity, mut visualizer, mut transform, mut sprite) in visualizer_query.iter_mut() {
            // Update position based on current flow
            let grid_pos = world_to_grid_pos(visualizer.grid_position, &fluid_environment);
            let current = sample_current(&fluid_environment, grid_pos);
            
            if current.length() < 5.0 {
                // Current too weak, remove visualizer
                commands.entity(entity).insert(AlreadyDespawned).despawn();
                continue;
            }
            
            // Update visualizer properties
            visualizer.flow_strength = current.length();
            visualizer.flow_direction = current.normalize_or_zero();
            visualizer.turbulence = fluid_environment.turbulence_intensity;
            
            // Move with the current
            transform.translation += current.extend(0.0) * time.delta_secs() * 0.5;
            visualizer.grid_position = transform.translation.truncate();
            
            // Rotate to show flow direction
            let target_rotation = Quat::from_rotation_z(current.to_angle());
            transform.rotation = transform.rotation.lerp(target_rotation, time.delta_secs() * 3.0);
            
            // Visual effects based on flow properties
            let flow_intensity = (current.length() / 100.0).clamp(0.1, 1.0);
            sprite.color = Color::srgba(
                0.3 + flow_intensity * 0.4,
                0.7,
                1.0,
                0.2 + flow_intensity * 0.4,
            );
            
            // Scale based on turbulence
            let turbulence_scale = 1.0 + visualizer.turbulence * 0.3;
            let flow_scale = 0.8 + flow_intensity * 0.4;
            transform.scale = Vec3::splat(turbulence_scale * flow_scale);
            
            // Remove if too far from player
            let distance = transform.translation.distance(player_transform.translation);
            if distance > 300.0 {
                commands.entity(entity).insert(AlreadyDespawned).despawn();
            }
        }
    }
}

// System to update tidal wave effects
pub fn update_tidal_wave_effects(
    mut commands: Commands,
    mut wave_query: Query<(Entity, &mut TidalWaveEffect, &mut Transform, &mut Sprite)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    for (entity, mut wave, mut transform, mut sprite) in wave_query.iter_mut() {
        wave.wave_phase += time.delta_secs() * wave.wave_speed;
        wave.propagation_distance += wave.wave_speed * time.delta_secs();
        
        if wave.propagation_distance >= wave.max_distance {
            commands.entity(entity).insert(AlreadyDespawned).despawn();
            continue;
        }
        
        // Animate wave propagation
        let progress = wave.propagation_distance / wave.max_distance;
        let wave_height = wave.wave_height * (1.0 - progress) * (wave.wave_phase.sin().abs());
        
        // Update visual properties
        let scale = 1.0 + progress * 3.0;
        transform.scale = Vec3::splat(scale);
        
        let alpha = (1.0 - progress) * 0.7;
        sprite.color.set_alpha(alpha);
        
        // Spawn wave particles at the edge
        if let Some(assets) = &assets {
            if (wave.wave_phase * 4.0) % (2.0 * std::f32::consts::PI) < 0.5 {
                for i in 0..8 {
                    let angle = (i as f32 / 8.0) * TAU;
                    let edge_pos = transform.translation.truncate() + 
                        Vec2::from_angle(angle) * wave.propagation_distance;
                    
                    commands.spawn((
                        Sprite {
                            image: assets.particle_texture.clone(),
                            color: Color::srgba(0.8, 0.9, 1.0, 0.6),
                            custom_size: Some(Vec2::splat(4.0)),
                            ..default()
                        },
                        Transform::from_translation(edge_pos.extend(0.3)),
                        Particle {
                            velocity: Vec2::from_angle(angle) * 50.0,
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
    }
}

// Audio feedback system for tidal events
pub fn tidal_audio_feedback_system(
    tidal_feedback: Res<TidalFeedbackSystem>,
    tidal_physics: Res<TidalPoolPhysics>,
    mut tidal_events: EventReader<TidalEvent>,
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
) {
    if !tidal_feedback.audio_feedback_enabled { return; }
    
    for event in tidal_events.read() {
        if let Some(assets) = &assets {
            match event {
                TidalEvent::KingTideBegin { intensity, .. } => {
                    commands.spawn((
                        AudioPlayer::new(assets.sfx_explosion.clone()),
                        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.8 * intensity)),
                    ));
                }
                
                TidalEvent::HighTideReached => {
                    commands.spawn((
                        AudioPlayer::new(assets.sfx_powerup.clone()),
                        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.4)),
                    ));
                }
                
                TidalEvent::CurrentReversal { .. } => {
                    commands.spawn((
                        AudioPlayer::new(assets.sfx_shoot.clone()),
                        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.3)),
                    ));
                }
                
                _ => {}
            }
        }
    }
}

// Player movement response system
pub fn tidal_movement_response_system(
    mut player_query: Query<(&mut Transform, &mut FluidDynamics), With<Player>>,
    tidal_physics: Res<TidalPoolPhysics>,
    fluid_environment: Res<FluidEnvironment>,
    mut feedback_particles: EventWriter<SpawnParticles>,
    time: Res<Time>,
) {
    if let Ok((mut player_transform, mut fluid_dynamics)) = player_query.single_mut() {
        let player_pos = player_transform.translation.truncate();
        let grid_pos = world_to_grid_pos(player_pos, &fluid_environment);
        let local_current = sample_current(&fluid_environment, grid_pos);
        
        // Enhanced player response to strong currents
        if local_current.length() > 50.0 {
            let current_force = local_current * fluid_dynamics.current_influence * 1.5;
            player_transform.translation += current_force.extend(0.0) * time.delta_secs();
            
            // Spawn feedback particles when caught in strong current
            if local_current.length() > 100.0 {
                feedback_particles.write(SpawnParticles {
                    position: player_transform.translation,
                    count: 5,
                    config: ParticleConfig {
                        color_start: Color::srgb(0.5, 0.8, 1.0),
                        color_end: Color::srgba(0.3, 0.6, 0.9, 0.0),
                        velocity_range: (
                            local_current * 0.8,
                            local_current * 1.2,
                        ),
                        lifetime_range: (0.5, 1.5),
                        size_range: (2.0, 4.0),
                        gravity: Vec2::ZERO,
                        organic_motion: true,
                        bioluminescence: 0.6,
                    },
                });
            }
        }
        
        // King tide effects on player movement
        if tidal_physics.king_tide_active {
            let chaos_force = Vec2::new(
                (time.elapsed_secs() * 8.0).sin() * 30.0,
                (time.elapsed_secs() * 6.0).cos() * 20.0,
            ) * tidal_physics.king_tide_intensity;
            
            player_transform.translation += chaos_force.extend(0.0) * time.delta_secs();
            
            // Enhanced visual feedback during king tide
            if (time.elapsed_secs() * 4.0) % 1.0 < 0.1 {
                feedback_particles.write(SpawnParticles {
                    position: player_transform.translation + Vec3::new(0.0, -20.0, 0.0),
                    count: 8,
                    config: ParticleConfig {
                        color_start: Color::srgb(1.0, 0.8, 0.6),
                        color_end: Color::srgba(0.8, 0.5, 0.3, 0.0),
                        velocity_range: (Vec2::new(-80.0, -40.0), Vec2::new(80.0, 40.0)),
                        lifetime_range: (1.0, 2.5),
                        size_range: (3.0, 8.0),
                        gravity: Vec2::new(0.0, -15.0),
                        organic_motion: true,
                        bioluminescence: 0.8,
                    },
                });
            }
        }
    }
}

// Helper functions
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
