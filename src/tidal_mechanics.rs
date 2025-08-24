use std::f32::consts::{TAU};
use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;
use crate::achievements::*;

pub fn advanced_tidal_system(
    mut tidal_physics: ResMut<TidalPoolPhysics>,
    mut fluid_environment: ResMut<FluidEnvironment>,
    mut current_generator: ResMut<CurrentGenerator>,
    mut chemical_environment: ResMut<ChemicalEnvironment>,
    mut tidal_events: EventWriter<TidalEvent>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
    mut last_king_tide: Local<f32>,
    mut current_tide_phase: Local<TidePhase>,
) {
    let old_tide_level = tidal_physics.tide_level;
    
    // Update base tidal cycle (slower, more dramatic)
    tidal_physics.tide_level += tidal_physics.tide_cycle_speed * time.delta_secs();
    if tidal_physics.tide_level > TAU {
        tidal_physics.tide_level -= TAU;
    }
    
    // Calculate current tide strength (-1.0 to 1.0)
    let tide_strength = (tidal_physics.tide_level).sin();
    
    // Detect tide phase changes
    let new_phase = if tide_strength > 0.8 {
        TidePhase::HighTide
    } else if tide_strength < -0.8 {
        TidePhase::LowTide
    } else if tide_strength > 0.0 && old_tide_level.sin() <= 0.0 {
        TidePhase::Rising
    } else if tide_strength < 0.0 && old_tide_level.sin() >= 0.0 {
        TidePhase::Receding
    } else {
        *current_tide_phase
    };
    
    // Trigger events on phase change
    if !matches!(*current_tide_phase, TidePhase::HighTide) && matches!(new_phase, TidePhase::HighTide) {
        tidal_events.write(TidalEvent::HighTideReached);
    }
    
    *current_tide_phase = new_phase;
    
    // KING TIDE EVENTS - Rare, dramatic events every 90-120 seconds
    *last_king_tide += time.delta_secs();
    if *last_king_tide > 90.0 && (time.elapsed_secs() * 0.1).sin().abs() < 0.01 {
        *last_king_tide = 0.0;
        trigger_king_tide(&mut tidal_physics, &mut tidal_events);
    }
    

    // Apply tidal effects to environment
    apply_tidal_effects(
        &tidal_physics,
        &mut fluid_environment,
        &mut current_generator,
        &mut chemical_environment,
        tide_strength,
        time.delta_secs(),
    );
    
    // Visual camera effects for dramatic tides
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let tide_sway = tide_strength * 2.0;
        camera_transform.translation.x += tide_sway * time.delta_secs();
        
        // King tide camera shake
        if tidal_physics.king_tide_active {
            let king_tide_shake = (time.elapsed_secs() * 12.0).sin() * 8.0;
            camera_transform.translation += Vec3::new(king_tide_shake, king_tide_shake * 0.5, 0.0) * time.delta_secs();
        }
    }
}

fn trigger_king_tide(
    tidal_physics: &mut TidalPoolPhysics,
    tidal_events: &mut EventWriter<TidalEvent>,
) {
    tidal_physics.king_tide_active = true;
    tidal_physics.king_tide_timer = 0.0;
    tidal_physics.king_tide_intensity = 2.5;
    tidal_physics.current_strength *= 3.0; // Massive current boost
    
    tidal_events.write(TidalEvent::KingTideBegin {
        intensity: tidal_physics.king_tide_intensity,
        duration: 15.0,
    });
}

fn apply_tidal_effects(
    tidal_physics: &TidalPoolPhysics,
    mut fluid_environment: &mut FluidEnvironment,
    mut current_generator: &mut CurrentGenerator,
    mut chemical_environment: &mut ChemicalEnvironment,
    tide_strength: f32,
    delta_time: f32,
) {
    // 1. CURRENT FIELD UPDATES - Modify the flow field, not direct positions
    let tide_direction_multiplier = if tide_strength > 0.0 { 1.0 } else { -1.0 };
    
    // Update the current field instead of directly moving entities
    for y in 0..fluid_environment.grid_size {
        for x in 0..fluid_environment.grid_size {
            let index = y * fluid_environment.grid_size + x;
            if index < fluid_environment.current_field.len() {
                // Apply gentle tidal influence to current field
                let tidal_current = Vec2::new(
                    tide_strength * 15.0 * tide_direction_multiplier, // Much gentler - was causing entities to fly off
                    tide_strength * 5.0, // Vertical component
                );
                
                // Blend with existing current instead of overriding
                fluid_environment.current_field[index] = 
                    fluid_environment.current_field[index] * 0.7 + tidal_current * 0.3;
            }
        }
    }
    
    // 2. TURBULENCE - Safe to modify
    fluid_environment.turbulence_intensity = 0.3 + tide_strength.abs() * 0.4;
    if tidal_physics.king_tide_active {
        fluid_environment.turbulence_intensity *= 1.5; // Reduced from 2.0
    }
    
    // 3. CHEMICAL ZONES - Gentle movement instead of aggressive pushing
    for zone in &mut chemical_environment.ph_zones {
        // Much gentler zone movement - was causing zones to fly off screen
        let gentle_drift = Vec2::new(
            tide_strength * 8.0 * delta_time, // Reduced from 20.0
            (tide_strength * 0.3).sin() * 3.0 * delta_time, // Reduced from 10.0
        );
        
        zone.position += gentle_drift;
        
        // Keep zones within reasonable bounds
        zone.position.x = zone.position.x.clamp(-800.0, 800.0);
        zone.position.y = zone.position.y.clamp(-400.0, 400.0);
        
        // pH intensity changes - this is safe
        if tidal_physics.king_tide_active {
            let intensity_boost = 1.0 + tidal_physics.king_tide_intensity * 0.1; // Reduced from 0.2
            zone.intensity = (zone.intensity * intensity_boost).min(1.2); // Reduced max from 1.5
        }
    }
    
    // 4. OXYGEN ZONES - Apply similar gentle movement
    for oxygen_zone in &mut chemical_environment.oxygen_zones {
        let gentle_drift = Vec2::new(
            tide_strength * 6.0 * delta_time,
            (tide_strength * 0.4).cos() * 4.0 * delta_time,
        );
        
        oxygen_zone.position += gentle_drift;
        
        // Keep oxygen zones in bounds
        oxygen_zone.position.x = oxygen_zone.position.x.clamp(-800.0, 800.0);
        oxygen_zone.position.y = oxygen_zone.position.y.clamp(-400.0, 400.0);
        
        // Tidal effects on oxygen levels
        if tidal_physics.king_tide_active {
            oxygen_zone.oxygen_level *= 0.95; // Slight depletion during king tides
            oxygen_zone.depletion_rate *= 1.1; // Faster depletion
        }
    }
    
    // 5. THERMAL VENTS - Only modify strength, not position
    for vent in &mut current_generator.thermal_vents {
        if tidal_physics.king_tide_active {
            vent.strength *= 1.2; // Reduced from 1.5
            vent.active = true;
        } else {
            // Gentle modulation instead of aggressive changes
            let activity_mod = 0.9 + tide_strength.abs() * 0.2; // Reduced range
            vent.strength *= activity_mod;
        }
        
        // Clamp vent strength to prevent extreme values
        vent.strength = vent.strength.clamp(50.0, 300.0);
    }
    
    // 6. MAJOR CURRENTS - Fix the main problem - don't make currents too strong
    for current in &mut current_generator.major_currents {
        let base_strength = 100.0; // Set a reasonable base strength
        current.strength = base_strength * tide_direction_multiplier * (1.0 + tide_strength.abs() * 0.3); // Much gentler
        
        // King tide: Moderate chaos instead of extreme
        if tidal_physics.king_tide_active {
            let chaos = (tidal_physics.king_tide_timer * 4.0).sin(); // Reduced frequency
            current.strength *= 1.0 + chaos * 0.2; // Reduced chaos from 0.5
        }
        
        // Clamp current strength to prevent entities flying off screen
        current.strength = current.strength.clamp(-200.0, 200.0);
    }
    
    // 7. TIDAL PHASE EFFECTS - New: Different effects based on tide phase
    match tidal_physics.tide_level.sin() {
        t if t > 0.8 => {
            // High tide - stronger currents, more active chemistry
            fluid_environment.turbulence_intensity *= 1.1;
            for zone in &mut chemical_environment.ph_zones {
                zone.intensity *= 1.05;
            }
        }
        t if t < -0.8 => {
            // Low tide - calmer waters, more concentrated chemicals
            fluid_environment.turbulence_intensity *= 0.9;
            for zone in &mut chemical_environment.ph_zones {
                zone.radius *= 0.95; // Slightly smaller but more intense
                zone.intensity *= 1.1;
            }
        }
        _ => {
            // Transitional tides - normal behavior
        }
    }
}

// Tidal event processing system
pub fn process_tidal_events(
    mut commands: Commands,
    mut tidal_events: EventReader<TidalEvent>,
    mut achievement_events: EventWriter<AchievementEvent>, // NEW
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_enemy_events: EventWriter<SpawnEnemy>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    for event in tidal_events.read() {
        match event {
            TidalEvent::KingTideBegin { intensity, duration } => {
                println!("KING TIDE EVENT! Intensity: {:.1}", intensity);
                
                // Achievement tracking for surviving king tides
                achievement_events.write(AchievementEvent::KingTideSurvived);
                
                spawn_tidal_debris(&mut commands, &assets, *intensity);
                spawn_king_tide_enemies(&mut spawn_enemy_events, *intensity);
                enemy_spawner.spawn_timer *= 0.3;
            }
            
            TidalEvent::HighTideReached => {
                enemy_spawner.spawn_timer *= 0.7;
                spawn_thermal_vent_activation(&mut commands, &assets);
            }
            
            TidalEvent::LowTideReached => {
                enemy_spawner.spawn_timer *= 1.5;
            }
            
            TidalEvent::CurrentReversal { new_direction } => {
                spawn_current_reversal_effect(&mut commands, &assets, *new_direction);
            }

            _ => {}
        }
    }
}

fn spawn_tidal_debris(commands: &mut Commands, assets: &Option<Res<GameAssets>>, intensity: f32) {
    if let Some(assets) = assets {
        // Spawn organic debris carried by king tide
        for i in 0..((intensity * 8.0) as u32) {
            let x_pos = (i as f32 * 100.0 - 400.0) + (intensity * 50.0).sin() * 200.0;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgb(0.6, 0.4, 0.2), // Debris brown
                    custom_size: Some(Vec2::splat(8.0 + intensity * 4.0)),
                    ..default()
                },
                Transform::from_xyz(x_pos, 400.0, -0.5),
                TidalDebris {
                    velocity: Vec2::new(
                        (intensity * 2.0).sin() * 150.0,
                        -200.0 - intensity * 100.0,
                    ),
                    lifetime: 0.0,
                    max_lifetime: 8.0,
                    spin_speed: intensity * 3.0,
                },
                Collider { radius: 4.0 + intensity * 2.0 },
            ));
        }
    }
}

fn spawn_king_tide_enemies(spawn_events: &mut EventWriter<SpawnEnemy>, intensity: f32) {
    // King tide brings special enemy formations
    let enemy_count = (intensity * 3.0) as u32;
    
    for i in 0..enemy_count {
        let x_offset = (i as f32 - enemy_count as f32 / 2.0) * 80.0;
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(x_offset, 450.0, 0.0),
            ai_type: EnemyAI::FluidFlow {
                flow_sensitivity: intensity * 2.0,
                base_direction: Vec2::new(0.0, -1.0),
            },
            enemy_type: EnemyType::ParasiticProtozoa, // Hardy enemies that thrive in chaos
        });
    }
}

fn spawn_thermal_vent_activation(commands: &mut Commands, assets: &Option<Res<GameAssets>>) {
    if let Some(assets) = assets {
        // Visual effect for thermal vent activation
        commands.spawn((
            Sprite {
                image: assets.explosion_texture.clone(),
                color: Color::srgb(1.0, 0.6, 0.2),
                custom_size: Some(Vec2::splat(60.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.2),
            ThermalVentActivation {
                timer: 0.0,
                max_time: 2.0,
                pulse_frequency: 4.0,
            },
        ));
    }
}

fn spawn_current_reversal_effect(commands: &mut Commands, assets: &Option<Res<GameAssets>>, direction: Vec2) {
    if let Some(assets) = assets {
        // Arrow indicators showing new current direction
        for i in 0..5 {
            let pos = Vec2::new((i as f32 - 2.0) * 120.0, 100.0);
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgb(0.3, 0.8, 1.0),
                    custom_size: Some(Vec2::splat(12.0)),
                    ..default()
                },
                Transform::from_translation(pos.extend(0.5))
                    .with_rotation(Quat::from_rotation_z(direction.to_angle())),
                CurrentIndicator {
                    timer: 0.0,
                    max_time: 3.0,
                    direction,
                },
            ));
        }
    }
}

// Update king tide state
pub fn update_king_tide(
    mut tidal_physics: ResMut<TidalPoolPhysics>,
    mut tidal_events: EventWriter<TidalEvent>,
    time: Res<Time>,
) {
    if tidal_physics.king_tide_active {
        tidal_physics.king_tide_timer += time.delta_secs();
        
        if tidal_physics.king_tide_timer >= 15.0 {
            // King tide ends
            tidal_physics.king_tide_active = false;
            tidal_physics.king_tide_intensity = 1.0;
            tidal_physics.current_strength = 1.0; // Reset to normal
            
            tidal_events.write(TidalEvent::KingTideEnd);
            println!("🌊 King Tide subsides...");
        }
    }
}

// Update tidal debris
pub fn update_tidal_debris(
    mut commands: Commands,
    mut debris_query: Query<(Entity, &mut Transform, &mut TidalDebris), Without<AlreadyDespawned>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut debris) in debris_query.iter_mut() {
        debris.lifetime += time.delta_secs();
        
        if debris.lifetime >= debris.max_lifetime {
            commands.entity(entity)
                .insert(AlreadyDespawned)
                .despawn();
            continue;
        }
        
        // Move debris with tidal forces
        transform.translation += debris.velocity.extend(0.0) * time.delta_secs();
        
        // Organic spinning motion
        transform.rotation *= Quat::from_rotation_z(debris.spin_speed * time.delta_secs());
        
        // Debris slows down over time
        debris.velocity *= 0.995;
    }
}