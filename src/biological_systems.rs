use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use std::f32::consts::{PI, TAU};
use crate::enemy_types::*;

// Fluid Dynamics System - Core water physics simulation
pub fn fluid_dynamics_system(
    mut fluid_environment: ResMut<FluidEnvironment>,
    mut current_generator: ResMut<CurrentGenerator>,
    tidal_physics: Res<TidalPoolPhysics>,
    time: Res<Time>,
) {
    // Update tidal cycle
    current_generator.tidal_cycle += time.delta_secs() * tidal_physics.tide_cycle_speed;
    current_generator.update_timer += time.delta_secs();
    
    // Regenerate current field every few seconds for performance
    if current_generator.update_timer >= 0.5 {
        current_generator.update_timer = 0.0;
        
        let tidal_strength = (current_generator.tidal_cycle * TAU).sin() * tidal_physics.current_strength;
        
        for y in 0..fluid_environment.grid_size {
            for x in 0..fluid_environment.grid_size {
                let world_pos = grid_to_world_pos(x, y, &fluid_environment);
                let mut flow = Vec2::ZERO;
                
                // Base tidal current - horizontal flow that changes with tide
                flow.x += tidal_strength * 0.4;
                
                // Add noise-based turbulence for organic feel
                let noise_x = (world_pos.x * 0.005 + time.elapsed_secs() * 0.2).sin();
                let noise_y = (world_pos.y * 0.007 + time.elapsed_secs() * 0.15).cos();
                flow += Vec2::new(noise_x, noise_y) * fluid_environment.turbulence_intensity * 30.0;
                
                // Thermal vent influences create upward currents
                for vent in &current_generator.thermal_vents {
                    if vent.active {
                        let distance = world_pos.distance(vent.position);
                        if distance < 250.0 { // Increased range
                            let direction = (world_pos - vent.position).normalize_or_zero();
                            let strength = vent.strength * (1.0 - distance / 250.0).powi(2);
                            
                            // Thermal vents create strong rising currents with swirl
                            let swirl_angle = (distance * 0.02) + (time.elapsed_secs() * 0.8);
                            let swirl_direction = Vec2::new(swirl_angle.cos(), swirl_angle.sin());
                            
                            flow += direction * strength * 0.7 + Vec2::new(0.0, strength * 1.2); // Stronger upward
                            flow += swirl_direction * strength * 0.4; // Add swirl component
                        }
                    }
                }
                
                // Major currents create directional flow
                for current in &current_generator.major_currents {
                    let current_direction = (current.end_pos - current.start_pos).normalize_or_zero();
                    let distance_to_current = point_to_line_distance(world_pos, current.start_pos, current.end_pos);
                    
                    if distance_to_current < current.width {
                        let influence = 1.0 - (distance_to_current / current.width);
                        flow += current_direction * current.strength * influence;
                    }
                }
                
                // Surface effects near boundaries
                let boundary_distance = (world_pos.x.abs() - 580.0).max(0.0);
                if boundary_distance > 0.0 {
                    let surface_force = Vec2::new(-world_pos.x.signum() * 50.0, 0.0);
                    flow += surface_force * (boundary_distance / 20.0);
                }
                
                let index = y * fluid_environment.grid_size + x;
                fluid_environment.current_field[index] = flow;
            }
        }
    }
}

// Scale Transition System - Seamless zoom between molecular and macro levels
pub fn scale_transition_system(
    mut scale_manager: ResMut<ScaleManager>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut all_transforms: Query<&mut Transform, (Without<Camera2d>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    trigger_conditions: Query<&ScaleTransitionTrigger>,
    mut physics: ResMut<MicroscopicPhysics>,
    time: Res<Time>,
) {
    // Check for scale transition triggers
    for trigger in trigger_conditions.iter() {
        if trigger.should_transition {
            scale_manager.target_scale = trigger.target_scale;
        }
    }
    
    // Smooth scale interpolation
    let scale_diff = scale_manager.target_scale - scale_manager.current_scale;
    if scale_diff.abs() > 0.01 {
        scale_manager.current_scale += scale_diff * scale_manager.transition_speed * time.delta_secs();
        
        // Update camera scale
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.scale = Vec3::splat(scale_manager.current_scale);
        }
        
        // Update physics based on scale
        scale_manager.physics_scale_factor = match scale_manager.current_scale {
            s if s < 0.5 => 3.0, // Molecular level - intense Brownian motion
            s if s < 2.0 => 1.0, // Cellular level - normal physics
            _ => 0.3,           // Tissue level - damped motion
        };
        
        physics.brownian_motion_intensity = scale_manager.physics_scale_factor;
        physics.molecular_collision_rate = scale_manager.physics_scale_factor * 0.8;
    }
}

// Procedural Current Generation System
pub fn generate_procedural_currents(
    mut fluid_environment: ResMut<FluidEnvironment>,
    mut current_generator: ResMut<CurrentGenerator>,
    tidal_physics: Res<TidalPoolPhysics>,
    time: Res<Time>,
) {
    // Update noise offset for moving turbulence
    current_generator.noise_offset += Vec2::new(
        time.delta_secs() * 10.0,
        time.delta_secs() * 5.0,
    );
    
    // Generate organic current patterns
    for y in 0..fluid_environment.grid_size {
        for x in 0..fluid_environment.grid_size {
            let world_pos = grid_to_world_pos(x, y, &fluid_environment);
            let mut flow = Vec2::ZERO;
            
            // Base tidal flow - varies with position
            let tidal_influence = (current_generator.tidal_cycle * TAU).sin();
            let position_factor = (world_pos.x * 0.001).sin();
            flow.x += tidal_influence * tidal_physics.current_strength * position_factor * 0.5;
            
            // Organic turbulence using multiple noise octaves
            let noise_pos = world_pos * 0.003 + current_generator.noise_offset;
            let turbulence1 = Vec2::new(
                (noise_pos.x).sin() * (noise_pos.y * 1.7).cos(),
                (noise_pos.y).sin() * (noise_pos.x * 1.3).cos(),
            );
            let turbulence2 = Vec2::new(
                (noise_pos.x * 2.1).sin() * (noise_pos.y * 2.8).cos(),
                (noise_pos.y * 2.3).sin() * (noise_pos.x * 2.9).cos(),
            ) * 0.5;
            
            flow += (turbulence1 + turbulence2) * fluid_environment.turbulence_intensity * 40.0;
            
            // Vortex generation around thermal vents
            for vent in &current_generator.thermal_vents {
                if vent.active {
                    let distance = world_pos.distance(vent.position);
                    if distance < 250.0 && distance > 20.0 {
                        let angle = (world_pos - vent.position).to_angle();
                        let spiral_angle = angle + (distance * 0.01) + (time.elapsed_secs() * 0.5);
                        let spiral_strength = vent.strength * (1.0 - distance / 250.0) * 0.3;
                        
                        flow += Vec2::from_angle(spiral_angle) * spiral_strength;
                        // Add upward thermal current
                        flow.y += spiral_strength * 0.8;
                    }
                }
            }
            
            // Boundary currents - water flows along edges
            let boundary_influence = calculate_boundary_current(world_pos);
            flow += boundary_influence * 30.0;
            
            let index = y * fluid_environment.grid_size + x;
            if index < fluid_environment.current_field.len() {
                fluid_environment.current_field[index] = flow;
            }
        }
    }
}

// Organic Particle Effects System
pub fn spawn_organic_particle_effects(
    mut commands: Commands,
    mut particle_events: EventWriter<SpawnParticles>,
    enemy_query: Query<(&Transform, &Enemy, &Health)>,
    player_query: Query<&Transform, With<Player>>,
    chemical_environment: Res<ChemicalEnvironment>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer -= time.delta_secs();
    
    if *spawn_timer <= 0.0 && assets.is_some() {
        let assets = assets.unwrap();
        
        // Spawn nutrient particles in healthy zones
        for zone in &chemical_environment.ph_zones {
            if zone.ph_level >= 6.8 && zone.ph_level <= 7.2 {
                // Good pH zone - spawn nutrients
                particle_events.write(SpawnParticles {
                    position: zone.position.extend(0.0),
                    count: 3,
                    config: ParticleConfig {
                        color_start: Color::srgb(0.4, 0.9, 0.6),
                        color_end: Color::srgba(0.2, 0.7, 0.4, 0.0),
                        velocity_range: (Vec2::new(-15.0, -15.0), Vec2::new(15.0, 15.0)),
                        lifetime_range: (3.0, 6.0),
                        size_range: (0.2, 0.4), // 2-4
                        gravity: Vec2::new(0.0, -10.0),
                        organic_motion: true,
                        bioluminescence: 0.6,
                    },
                });
            }
        }
        
        // Spawn cellular debris around damaged enemies
        for (transform, enemy, health) in enemy_query.iter() {
            if health.0 < enemy.health / 2 {
                // Damaged enemy - spawn debris
                let debris_color = match enemy.enemy_type {
                    EnemyType::ViralParticle => Color::srgb(0.9, 0.9, 1.0),
                    EnemyType::AggressiveBacteria => Color::srgb(1.0, 0.4, 0.4),
                    EnemyType::ParasiticProtozoa => Color::srgb(0.7, 0.9, 0.4),
                    _ => Color::srgb(0.8, 0.8, 0.8),
                };
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: debris_color,
                        custom_size: Some(Vec2::splat(2.0)),
                        ..default()
                    },
                    Transform::from_translation(transform.translation),
                    BioluminescentParticle {
                        base_color: debris_color,
                        pulse_frequency: 4.0,
                        pulse_intensity: 0.3,
                        organic_motion: OrganicMotion {
                            undulation_speed: 2.0,
                            response_to_current: 0.9,
                        },
                    },
                    Particle {
                        velocity: Vec2::new(
                            (time.elapsed_secs() * 123.45 + transform.translation.x * 0.01).sin() * 50.0,
                            (time.elapsed_secs() * 67.89 + transform.translation.y * 0.01).cos() * 40.0,
                        ),
                        lifetime: 0.0,
                        max_lifetime: 2.0,
                        size: 2.0,
                        fade_rate: 1.0,
                        bioluminescent: true,
                        drift_pattern: DriftPattern::Brownian,
                    },
                ));
            }
        }
        
        // Spawn bioluminescent plankton near player
        if let Ok(player_transform) = player_query.single() {
            for i in 0..2 {
                let offset = Vec2::new(
                    (time.elapsed_secs() * 45.67 + i as f32 * 100.0).sin() * 200.0,
                    (time.elapsed_secs() * 23.45 + i as f32 * 150.0).cos() * 150.0,
                );
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgba(0.3, 0.8, 1.0, 0.7),
                        custom_size: Some(Vec2::splat(3.0)),
                        ..default()
                    },
                    Transform::from_translation(player_transform.translation + offset.extend(0.0)),
                    BioluminescentParticle {
                        base_color: Color::srgb(0.3, 0.8, 1.0),
                        pulse_frequency: 1.0 + (i as f32 * 0.5),
                        pulse_intensity: 0.5,
                        organic_motion: OrganicMotion {
                            undulation_speed: 1.5,
                            response_to_current: 0.7,
                        },
                    },
                    Particle {
                        velocity: Vec2::new(
                            (time.elapsed_secs() * 89.12).sin() * 10.0,
                            (time.elapsed_secs() * 56.78).cos() * 8.0,
                        ),
                        lifetime: 0.0,
                        max_lifetime: 12.0,
                        size: 3.0,
                        fade_rate: 1.0,
                        bioluminescent: true,
                        drift_pattern: DriftPattern::Floating,
                    },
                ));
            }
        }
        
        *spawn_timer = 2.0; // Spawn every 2 seconds
    }
}

// Helper functions
fn grid_to_world_pos(grid_x: usize, grid_y: usize, fluid_env: &FluidEnvironment) -> Vec2 {
    Vec2::new(
        grid_x as f32 * fluid_env.cell_size - 640.0,
        grid_y as f32 * fluid_env.cell_size - 360.0,
    )
}

pub fn world_to_grid_pos(world_pos: Vec2, fluid_env: &FluidEnvironment) -> (usize, usize) {
    let grid_x = ((world_pos.x + 640.0) / fluid_env.cell_size).clamp(0.0, (fluid_env.grid_size - 1) as f32) as usize;
    let grid_y = ((world_pos.y + 360.0) / fluid_env.cell_size).clamp(0.0, (fluid_env.grid_size - 1) as f32) as usize;
    (grid_x, grid_y)
}

pub fn sample_current(fluid_env: &FluidEnvironment, grid_pos: (usize, usize)) -> Vec2 {
    let index = grid_pos.1 * fluid_env.grid_size + grid_pos.0;
    if index < fluid_env.current_field.len() {
        fluid_env.current_field[index]
    } else {
        Vec2::ZERO
    }
}

fn sample_ph(chemical_env: &ChemicalEnvironment, position: Vec2) -> f32 {
    let mut ph = chemical_env.base_ph;
    
    for zone in &chemical_env.ph_zones {
        let distance = position.distance(zone.position);
        if distance < zone.radius {
            let influence = (1.0 - distance / zone.radius) * zone.intensity;
            ph = ph * (1.0 - influence) + zone.ph_level * influence;
        }
    }
    
    ph.clamp(0.0, 14.0)
}

fn sample_oxygen(chemical_env: &ChemicalEnvironment, position: Vec2) -> f32 {
    let mut oxygen = chemical_env.base_oxygen;
    
    for zone in &chemical_env.oxygen_zones {
        let distance = position.distance(zone.position);
        if distance < zone.radius {
            let influence = 1.0 - distance / zone.radius;
            oxygen = oxygen * (1.0 - influence) + zone.oxygen_level * influence;
        }
    }
    
    oxygen.clamp(0.0, 1.0)
}

fn point_to_line_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    
    if line_vec.length() == 0.0 {
        return point_vec.length();
    }
    
    let t = (point_vec.dot(line_vec) / line_vec.length_squared()).clamp(0.0, 1.0);
    let projection = line_start + line_vec * t;
    point.distance(projection)
}

fn calculate_boundary_current(world_pos: Vec2) -> Vec2 {
    let bounds = Vec2::new(600.0, 350.0);
    let mut boundary_flow = Vec2::ZERO;
    
    // Distance from each boundary
    let dist_right = bounds.x - world_pos.x;
    let dist_left = bounds.x + world_pos.x;
    let dist_top = bounds.y - world_pos.y;
    let dist_bottom = bounds.y + world_pos.y;
    
    let boundary_influence = 50.0;
    
    // Flow along boundaries
    if dist_right < boundary_influence {
        boundary_flow.y += (1.0 - dist_right / boundary_influence) * 0.5;
    }
    if dist_left < boundary_influence {
        boundary_flow.y -= (1.0 - dist_left / boundary_influence) * 0.5;
    }
    if dist_top < boundary_influence {
        boundary_flow.x += (1.0 - dist_top / boundary_influence) * 0.5;
    }
    if dist_bottom < boundary_influence {
        boundary_flow.x -= (1.0 - dist_bottom / boundary_influence) * 0.5;
    }
    
    boundary_flow
}

// New component for scale transitions
#[derive(Component)]
pub struct ScaleTransitionTrigger {
    pub should_transition: bool,
    pub target_scale: f32,
    pub transition_type: TransitionType,
}

#[derive(Clone)]
pub enum TransitionType {
    EnterOrganism, // Zoom into bloodstream/digestive system
    ExitOrganism,  // Return to open water
    MolecularLevel, // Ultra-zoom for cellular combat
}

// Chemical Environment System - pH and oxygen simulation
pub fn chemical_environment_system(
    mut chemical_env: ResMut<ChemicalEnvironment>,
    mut organism_query: Query<(&Transform, &mut Health, &ChemicalSensitivity, Option<&OsmoregulationActive>)>,
    enemy_query: Query<(&Transform, &Enemy), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    // Update chemical zone dynamics
    for zone in &mut chemical_env.ph_zones {
        // pH zones slowly drift and change intensity
        zone.intensity += (time.elapsed_secs() * 0.5 + zone.position.x * 0.001).sin() * 0.01;
        zone.intensity = zone.intensity.clamp(0.3, 1.0);
        
        // Zones slowly move with currents
        zone.position.x += (time.elapsed_secs() * 0.3).sin() * 10.0 * time.delta_secs();
        zone.position.y += (time.elapsed_secs() * 0.2).cos() * 5.0 * time.delta_secs();
    }
    
    for oxygen_zone in &mut chemical_env.oxygen_zones {
        // Oxygen depletion near large groups of organisms
        let mut nearby_organisms = 0;
        for (enemy_transform, _) in enemy_query.iter() {
            if enemy_transform.translation.distance(oxygen_zone.position.extend(0.0)) < oxygen_zone.radius {
                nearby_organisms += 1;
            }
        }
        
        // Deplete oxygen based on organism density
        oxygen_zone.oxygen_level -= nearby_organisms as f32 * oxygen_zone.depletion_rate * time.delta_secs();
        oxygen_zone.oxygen_level = oxygen_zone.oxygen_level.clamp(0.1, 1.0);
        
        // Slow oxygen regeneration
        oxygen_zone.oxygen_level += 0.1 * time.delta_secs();
        oxygen_zone.oxygen_level = oxygen_zone.oxygen_level.min(1.0);
    }
    
    // Apply chemical effects to organisms
    for (transform, mut health, sensitivity, osmoregulation) in organism_query.iter_mut() {
        // Skip if organism has osmoregulation active
        if osmoregulation.is_some() {
            continue;
        }
        
        let current_ph = sample_ph(&chemical_env, transform.translation.truncate());
        let current_oxygen = sample_oxygen(&chemical_env, transform.translation.truncate());
        
        // pH damage
        if current_ph < sensitivity.ph_tolerance_min || current_ph > sensitivity.ph_tolerance_max {
            let ph_damage = sensitivity.damage_per_second_outside_range as f32 * time.delta_secs();
            health.0 -= ph_damage as i32;
        }
        
        // Oxygen requirements
        if current_oxygen < sensitivity.oxygen_requirement {
            let oxygen_damage = (sensitivity.damage_per_second_outside_range / 2) as f32 * time.delta_secs();
            health.0 -= oxygen_damage as i32;
        }
        
        // Beneficial effects in optimal conditions
        if current_ph >= sensitivity.ph_tolerance_min + 0.2 && 
           current_ph <= sensitivity.ph_tolerance_max - 0.2 &&
           current_oxygen >= sensitivity.oxygen_requirement + 0.2 {
            // Slow healing in optimal conditions
            health.0 += (2.0 * time.delta_secs()) as i32;
            health.0 = health.0.min(100);
        }
    }
}

// Current Field Update System
pub fn update_current_field(
    mut current_indicators: Query<(&mut Transform, &mut Sprite, &CurrentField), Without<Player>>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (mut transform, mut sprite, _) in current_indicators.iter_mut() {
        // Sample local current
        let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
        let local_current = sample_current(&fluid_environment, grid_pos);
        
        // Visualize current strength with alpha
        let current_strength = local_current.length() / 100.0;
        sprite.color.set_alpha(0.05 + current_strength * 0.2);
        
        // Rotate indicator to show current direction
        if local_current.length() > 10.0 {
            let angle = local_current.y.atan2(local_current.x) - PI / 2.0;
            transform.rotation = Quat::from_rotation_z(angle);
        }
        
        // Gentle movement to show flow
        transform.translation += local_current.extend(0.0) * 0.1 * time.delta_secs();
        
        // Reset position if moved too far
        if transform.translation.length() > 800.0 {
            transform.translation = Vec3::new(
                (time.elapsed_secs() * 50.0).sin() * 300.0,
                (time.elapsed_secs() * 30.0).cos() * 200.0,
                transform.translation.z,
            );
        }
    }
}

// Bioluminescence System - Organic lighting effects
pub fn bioluminescence_system(
    mut commands: Commands,
    mut bioluminescence_manager: ResMut<BioluminescenceManager>,
    mut bioluminescent_query: Query<(&mut Transform, &mut Sprite, &mut BioluminescentParticle)>,
    player_query: Query<&Transform, (With<Player>, Without<BioluminescentParticle>)>,
    enemy_query: Query<(&Transform, &Enemy), (Without<Player>, Without<BioluminescentParticle>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    // Update existing bioluminescent particles
    for (mut transform, mut sprite, mut bio_particle) in bioluminescent_query.iter_mut() {
        // Organic pulsing
        let pulse_phase = time.elapsed_secs() * bio_particle.pulse_frequency;
        let pulse_intensity = (pulse_phase.sin() * 0.5 + 0.5) * bio_particle.pulse_intensity;
        
        // Apply pulsing to color brightness
        let mut color = bio_particle.base_color;
        let brightness = 0.6 + pulse_intensity * 0.4;
        color = Color::srgba(
            color.to_srgba().red * brightness,
            color.to_srgba().green * brightness,
            color.to_srgba().blue * brightness,
            color.to_srgba().alpha,
        );
        sprite.color = color;
        
        // Organic undulation motion
        let undulation = (time.elapsed_secs() * bio_particle.organic_motion.undulation_speed).sin();
        transform.translation.y += undulation * 3.0 * time.delta_secs();
        
        // Gentle rotation
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.2);
    }
    
    // Spawn ambient bioluminescent particles
    if let Some(assets) = assets {
        // Spawn particles near player
        if let Ok(player_transform) = player_query.single() {
            if (time.elapsed_secs() * 3.0).sin() > 0.8 {
                let spawn_offset = Vec2::new(
                    (time.elapsed_secs() * 123.45).sin() * 150.0,
                    (time.elapsed_secs() * 67.89).cos() * 100.0,
                );
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgba(0.3, 0.8, 1.0, 0.6),
                        custom_size: Some(Vec2::splat(4.0)),
                        ..default()
                    },
                    Transform::from_translation(player_transform.translation + spawn_offset.extend(0.0)),
                    BioluminescentParticle {
                        base_color: Color::srgb(0.3, 0.8, 1.0),
                        pulse_frequency: 1.5 + (time.elapsed_secs() * 0.1).sin(),
                        pulse_intensity: 0.4,
                        organic_motion: OrganicMotion {
                            undulation_speed: 2.0,
                            response_to_current: 0.8,
                        },
                    },
                    Particle {
                        velocity: Vec2::new(
                            (time.elapsed_secs() * 45.67).sin() * 20.0,
                            (time.elapsed_secs() * 89.12).cos() * 15.0,
                        ),
                        lifetime: 0.0,
                        max_lifetime: 8.0,
                        size: 4.0,
                        fade_rate: 1.0,
                        bioluminescent: true,
                        drift_pattern: DriftPattern::Floating,
                    },
                ));
            }
        }
        
        // Spawn particles near enemies occasionally
        for (enemy_transform, enemy) in enemy_query.iter() {
            if enemy.chemical_signature.releases_toxins && (time.elapsed_secs() * 5.0 + enemy_transform.translation.x * 0.001).sin() > 0.9 {
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgba(1.0, 0.4, 0.2, 0.7),
                        custom_size: Some(Vec2::splat(3.0)),
                        ..default()
                    },
                    Transform::from_translation(enemy_transform.translation),
                    BioluminescentParticle {
                        base_color: Color::srgb(1.0, 0.4, 0.2),
                        pulse_frequency: 3.0,
                        pulse_intensity: 0.6,
                        organic_motion: OrganicMotion {
                            undulation_speed: 1.0,
                            response_to_current: 0.5,
                        },
                    },
                    Particle {
                        velocity: Vec2::new(
                            (time.elapsed_secs() * 234.56).sin() * 30.0,
                            (time.elapsed_secs() * 178.90).cos() * 25.0,
                        ),
                        lifetime: 0.0,
                        max_lifetime: 3.0,
                        size: 3.0,
                        fade_rate: 1.0,
                        bioluminescent: true,
                        drift_pattern: DriftPattern::Brownian,
                    },
                ));
            }
        }
    }
}

// Organic AI System - Enhanced biological behaviors
pub fn organic_ai_system(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    chemical_environment: Res<ChemicalEnvironment>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut transform, mut enemy) in enemy_query.iter_mut() {

            let mut enemy_clone = enemy.clone();

            match &mut enemy.ai_type {
                EnemyAI::Chemotaxis { target_chemical, sensitivity, current_direction } => {
                    // Follow chemical gradients toward player
                    let player_distance = transform.translation.distance(player_transform.translation);
                    
                    if player_distance < 300.0 {
                        let direction_to_player = (player_transform.translation.truncate() - transform.translation.truncate()).normalize_or_zero();
                        
                        // Sample chemical gradient (simplified)
                        let chemical_strength = 1.0 / (player_distance * 0.01 + 1.0);
                        let influence = chemical_strength * *sensitivity;
                        
                        *current_direction = current_direction.lerp(direction_to_player, influence * time.delta_secs());
                        transform.translation += current_direction.extend(0.0) * enemy.speed * time.delta_secs();
                    } else {
                        // Random movement when no chemical trail
                        let random_turn = (time.elapsed_secs() * 3.0 + transform.translation.x * 0.01).sin();
                        *current_direction = Vec2::from_angle(current_direction.to_angle() + random_turn * 0.5 * time.delta_secs());
                        transform.translation += current_direction.extend(0.0) * enemy.speed * 0.5 * time.delta_secs();
                    }
                }
                
                EnemyAI::CellDivision { division_threshold, division_timer, has_divided } => {
                    if enemy_clone.health as f32 <= *division_threshold && !*has_divided {
                        *division_timer -= time.delta_secs();
                        
                        if *division_timer <= 0.0 {
                            // This would trigger cell division (spawn new enemy)
                            *has_divided = true;
                            // Division logic would be handled in enemy spawning system
                        }
                    }
                }
                
                EnemyAI::FluidFlow { flow_sensitivity, base_direction } => {
                    // Follow fluid currents more strongly than normal enemies
                    let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                    let current = sample_current(&fluid_environment, grid_pos);
                    
                    let flow_influence = current * *flow_sensitivity * time.delta_secs();
                    *base_direction = (*base_direction + flow_influence).normalize_or_zero();
                    
                    transform.translation += base_direction.extend(0.0) * enemy.speed * time.delta_secs();
                }
                
                EnemyAI::SymbioticPair { partner_entity, bond_distance, sync_timer } => {
                    *sync_timer += time.delta_secs();
                    
                    // Synchronize movement with partner (if it exists)
                    if let Some(partner) = partner_entity {
                        // Partner coordination would be implemented here
                        // For now, add gentle oscillation
                        let sync_offset = Vec2::new(
                            (*sync_timer * 2.0).sin() * *bond_distance * 0.5,
                            (*sync_timer * 1.5).cos() * *bond_distance * 0.3,
                        );
                        transform.translation += sync_offset.extend(0.0) * time.delta_secs();
                    }
                }
                
                // Enhanced existing AI with biological touches
                EnemyAI::Linear { direction } => {
                    // Add slight organic undulation to linear movement
                    let undulation = Vec2::new(
                        (time.elapsed_secs() * 2.0 + transform.translation.y * 0.01).sin() * 10.0,
                        0.0,
                    );
                    transform.translation += (direction.extend(0.0) + undulation.extend(0.0)) * enemy.speed * time.delta_secs();
                }
                
                EnemyAI::Sine { amplitude, frequency, phase } => {
                    *phase += time.delta_secs() * *frequency;
                    transform.translation.y -= enemy_clone.speed * time.delta_secs();
                    
                    // Add organic variation to sine wave
                    let organic_variation = (time.elapsed_secs() * 0.5).sin() * 0.2;
                    let actual_amplitude = *amplitude * (1.0 + organic_variation);
                    transform.translation.x += actual_amplitude * phase.sin() * time.delta_secs();
                }
                
                _ => {
                    // Default behavior remains unchanged
                }
            }
            
            // Apply chemical environment effects to AI
            if enemy.chemical_signature.responds_to_pheromones {
                let local_ph = sample_ph(&chemical_environment, transform.translation.truncate());
                
                // Enemies move away from unfavorable pH
                let ph_difference = (local_ph - enemy.chemical_signature.ph_preference).abs();
                if ph_difference > 1.0 {
                    let avoidance_direction = if local_ph > enemy.chemical_signature.ph_preference {
                        Vec2::new(-1.0, 0.0) // Move away from alkaline
                    } else {
                        Vec2::new(1.0, 0.0) // Move away from acidic
                    };
                    
                    transform.translation += avoidance_direction.extend(0.0) * enemy.speed * 0.3 * time.delta_secs();
                }
            }
        }
    }
}


pub fn apply_chemical_damage_system(
    mut player_query: Query<(&Transform, &mut Health, &ChemicalSensitivity, Option<&OsmoregulationActive>), With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Health, &Enemy), Without<Player>>,
    chemical_environment: Res<ChemicalEnvironment>,
    time: Res<Time>,
) {
    // Player chemical damage
    if let Ok((transform, mut health, sensitivity, osmo)) = player_query.single_mut() {
        if osmo.is_some() { return; } // Immune to chemical damage
        
        let ph = sample_ph(&chemical_environment, transform.translation.truncate());
        let oxygen = sample_oxygen(&chemical_environment, transform.translation.truncate());
        
        if ph < sensitivity.ph_tolerance_min || ph > sensitivity.ph_tolerance_max {
            let damage = (sensitivity.damage_per_second_outside_range as f32 * time.delta_secs()) as i32;
            health.0 -= damage;
        }
        
        if oxygen < sensitivity.oxygen_requirement {
            health.0 -= (3.0 * time.delta_secs()) as i32;
        }
    }
    
    // Enemy chemical effects
    for (transform, mut health, enemy) in enemy_query.iter_mut() {
        let ph = sample_ph(&chemical_environment, transform.translation.truncate());
        let ph_diff = (ph - enemy.chemical_signature.ph_preference).abs();
        
        if ph_diff > 1.5 {
            let damage = ((ph_diff - 1.5) * 8.0 * time.delta_secs()) as i32;
            health.0 -= damage;
        }
    }
}


// Advanced Particle Systems
pub fn advanced_bioluminescence_system(
    mut bio_query: Query<(&mut Sprite, &mut BioluminescentParticle, &Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<BioluminescentParticle>)>,
    chemical_environment: Res<ChemicalEnvironment>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut sprite, mut bio_particle, transform) in bio_query.iter_mut() {
            // Distance-based intensity
            let distance_to_player = transform.translation.distance(player_transform.translation);
            let proximity_boost = (200.0 - distance_to_player.min(200.0)) / 200.0;
            
            // Chemical environment affects bioluminescence
            let ph = sample_ph(&chemical_environment, transform.translation.truncate());
            let ph_factor = 1.0 - ((ph - 7.0).abs() * 0.1).min(0.5);
            
            // Dynamic pulsing
            let pulse_phase = time.elapsed_secs() * bio_particle.pulse_frequency;
            let pulse = (pulse_phase.sin() * 0.5 + 0.5) * bio_particle.pulse_intensity;
            
            let final_intensity = (0.4 + pulse * 0.6 + proximity_boost * 0.3) * ph_factor;
            
            let mut color = bio_particle.base_color;
            color = Color::srgba(
                color.to_srgba().red * final_intensity,
                color.to_srgba().green * final_intensity,
                color.to_srgba().blue * final_intensity,
                color.to_srgba().alpha,
            );
            sprite.color = color;
        }
    }
}

// Ecosystem State Tracking
pub fn ecosystem_monitoring_system(
    mut ecosystem: ResMut<EcosystemState>,
    enemy_query: Query<&Enemy>,
    chemical_environment: Res<ChemicalEnvironment>,
    player_query: Query<&Health, With<Player>>,
    time: Res<Time>,
) {
    // Count organism populations
    let mut pathogenic = 0;
    let mut beneficial = 0;
    
    for enemy in enemy_query.iter() {
        match enemy.enemy_type {
            EnemyType::AggressiveBacteria | EnemyType::ViralParticle => pathogenic += 1,
            EnemyType::SwarmCell => beneficial += 1,
            _ => {}
        }
    }
    
    ecosystem.population_balance.pathogenic_threats = pathogenic;
    ecosystem.population_balance.beneficial_microbes = beneficial;
    
    // Calculate ecosystem health
    let pathogen_ratio = pathogenic as f32 / (pathogenic + beneficial + 1) as f32;
    ecosystem.infection_level = pathogen_ratio;
    ecosystem.health = 1.0 - (pathogen_ratio * 0.8);
    
    // pH stability
    let avg_ph = chemical_environment.ph_zones.iter()
        .map(|z| z.ph_level)
        .sum::<f32>() / chemical_environment.ph_zones.len().max(1) as f32;
    ecosystem.ph_stability = 1.0 - ((avg_ph - 7.0).abs() / 7.0);
    
    // Player health affects ecosystem
    if let Ok(player_health) = player_query.single() {
        ecosystem.symbiotic_activity = (player_health.0 as f32 / 100.0) * 0.6 + 0.4;
    }
}

pub fn thermal_vent_effects_system(
    mut commands: Commands,
    current_generator: Res<CurrentGenerator>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy, &mut Health), Without<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut vent_timer: Local<f32>,
) {
    *vent_timer += time.delta_secs();
    
    for vent in &current_generator.thermal_vents {
        if !vent.active { continue; }
        
        // Spawn thermal particles
        if *vent_timer % 0.3 < 0.1 {
            if let Some(assets) = &assets {
                for i in 0..5 {
                    let angle = (i as f32 / 5.0) * std::f32::consts::TAU + *vent_timer;
                    let offset = Vec2::from_angle(angle) * (20.0 + (*vent_timer * 2.0).sin() * 10.0);
                    
                    commands.spawn((
                        Sprite {
                            image: assets.particle_texture.clone(),
                            color: Color::srgb(1.0, 0.6, 0.2), // Orange thermal glow
                            custom_size: Some(Vec2::splat(4.0)),
                            ..default()
                        },
                        Transform::from_translation(vent.position.extend(0.0) + offset.extend(0.0)),
                        ThermalParticle {
                            heat_intensity: vent.strength / 200.0,
                            rise_speed: 60.0,
                        },
                        Particle {
                            velocity: Vec2::new(0.0, 80.0) + offset.normalize() * 20.0,
                            lifetime: 0.0,
                            max_lifetime: 3.0,
                            size: 4.0,
                            fade_rate: 0.8,
                            bioluminescent: true,
                            drift_pattern: DriftPattern::Floating,
                        },
                    ));
                }
            }
        }
        
        // Player thermal effects
        if let Ok((player_transform, mut player_health)) = player_query.single_mut() {
            let distance = player_transform.translation.distance(vent.position.extend(0.0));
            
            if distance < 120.0 {
                let heat_intensity = (120.0 - distance) / 120.0;
                
                if heat_intensity > 0.7 {
                    // Damage from extreme heat
                    player_health.0 -= (heat_intensity * 15.0 * time.delta_secs()) as i32;
                } else if heat_intensity > 0.3 {
                    // Beneficial warmth - slight healing
                    player_health.0 = (player_health.0 + (2.0 * time.delta_secs()) as i32).min(100);
                }
            }
        }
        
        // Enemy thermal effects
        for (enemy_entity, enemy_transform, mut enemy, mut enemy_health) in enemy_query.iter_mut() {
            let distance = enemy_transform.translation.distance(vent.position.extend(0.0));
            
            if distance < 150.0 {
                let heat_factor = (150.0 - distance) / 150.0;
                
                // Heat affects different enemy types differently
                match enemy.enemy_type {
                    EnemyType::ViralParticle => {
                        // Viruses are destroyed by heat
                        if heat_factor > 0.5 {
                            enemy_health.0 -= (heat_factor * 20.0 * time.delta_secs()) as i32;
                        }
                    }
                    EnemyType::AggressiveBacteria => {
                        // Some bacteria thrive in heat
                        if heat_factor > 0.3 && heat_factor < 0.8 {
                            enemy.speed = 180.0 * (1.0 + heat_factor * 0.5); // Speed boost
                        }
                    }
                    EnemyType::ParasiticProtozoa => {
                        // Protozoa seek thermal vents
                        if distance < 100.0 {
                            // Attracted to thermal vents - move towards them
                            let direction = (vent.position - enemy_transform.translation.truncate()).normalize();
                            enemy.speed = 120.0; // Slower but persistent
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn scroll_thermal_vents(
    mut current_generator: ResMut<CurrentGenerator>,
    time: Res<Time>,
) {
    for vent in &mut current_generator.thermal_vents {
        // Scroll vents downward like everything else
        vent.position.y -= 50.0 * time.delta_secs();
        
        // Reset position when off-screen
        if vent.position.y < -400.0 {
            vent.position.y = 450.0;
            vent.position.x = (time.elapsed_secs() * 123.45).sin() * 500.0; // Random X
            vent.active = (time.elapsed_secs() * 67.89).sin() > 0.0; // Random activation
        }
    }
}

pub fn update_thermal_particles(
    mut commands: Commands,
    mut thermal_query: Query<(Entity, &mut Transform, &mut ThermalParticle, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut thermal, mut sprite) in thermal_query.iter_mut() {
        // Rise and dissipate
        transform.translation.y += thermal.rise_speed * time.delta_secs();
        
        // Heat shimmer effect
        let shimmer = (time.elapsed_secs() * 8.0 + transform.translation.x * 0.1).sin() * 3.0;
        transform.translation.x += shimmer * time.delta_secs();
        
        // Fade with height
        thermal.heat_intensity -= time.delta_secs() * 0.4;
        let alpha = thermal.heat_intensity.clamp(0.0, 1.0);
        sprite.color = Color::srgba(1.0, 0.6 + alpha * 0.4, 0.2, alpha * 0.8);
        
        if thermal.heat_intensity <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn dynamic_chemical_zone_system(
    mut chemical_environment: ResMut<ChemicalEnvironment>,
    enemy_query: Query<(&Transform, &Enemy)>,
    player_query: Query<&Transform, With<Player>>,
    ecosystem: Res<EcosystemState>,
    time: Res<Time>,
    mut zone_timer: Local<f32>,
) {
    *zone_timer += time.delta_secs();
    
    // Generate new zones every 8 seconds
    if *zone_timer >= 8.0 {
        *zone_timer = 0.0;
        
        // Spawn acidic zones near aggressive bacteria clusters
        let bacteria_positions: Vec<Vec2> = enemy_query.iter()
            .filter(|(_, enemy)| matches!(enemy.enemy_type, EnemyType::AggressiveBacteria))
            .map(|(transform, _)| transform.translation.truncate())
            .collect();
        
        for cluster_center in cluster_positions(&bacteria_positions, 80.0) {
            chemical_environment.ph_zones.push(crate::resources::ChemicalZone {
                position: cluster_center,
                radius: 100.0,
                ph_level: 5.0 + (ecosystem.infection_level * -1.5), // More acidic with higher infection
                intensity: 0.7,
            });
        }
        
        // Spawn oxygen depletion zones in dense areas
        let all_positions: Vec<Vec2> = enemy_query.iter()
            .map(|(transform, _)| transform.translation.truncate())
            .collect();
        
        for cluster_center in cluster_positions(&all_positions, 120.0) {
            chemical_environment.oxygen_zones.push(OxygenZone {
                position: cluster_center,
                radius: 90.0,
                oxygen_level: 0.2, // Low oxygen
                depletion_rate: 0.05,
            });
        }
        
        // Generate beneficial zones near player
        if let Ok(player_transform) = player_query.single() {
            let player_pos = player_transform.translation.truncate();
            chemical_environment.ph_zones.push(crate::resources::ChemicalZone {
                position: player_pos + Vec2::new(
                    (time.elapsed_secs() * 50.0).sin() * 150.0,
                    (time.elapsed_secs() * 30.0).cos() * 100.0,
                ),
                radius: 80.0,
                ph_level: 7.2, // Slightly alkaline, beneficial
                intensity: 0.5,
            });
        }
        
        let chem_env_clone = chemical_environment.clone();
        // Remove old zones (keep only last 6)
        if chemical_environment.ph_zones.len() > 6 {
            chemical_environment.ph_zones.drain(0..chem_env_clone.ph_zones.len()-6);
        }
        if chemical_environment.oxygen_zones.len() > 4 {
            chemical_environment.oxygen_zones.drain(0..chem_env_clone.oxygen_zones.len()-4);
        }
    }
}

// Helper function to find cluster centers
fn cluster_positions(positions: &[Vec2], cluster_radius: f32) -> Vec<Vec2> {
    let mut clusters = Vec::new();
    let mut used = vec![false; positions.len()];
    
    for (i, &pos) in positions.iter().enumerate() {
        if used[i] { continue; }
        
        let mut cluster_center = pos;
        let mut cluster_size = 1;
        used[i] = true;
        
        // Find nearby positions
        for (j, &other_pos) in positions.iter().enumerate() {
            if i != j && !used[j] && pos.distance(other_pos) < cluster_radius {
                cluster_center = (cluster_center * cluster_size as f32 + other_pos) / (cluster_size + 1) as f32;
                cluster_size += 1;
                used[j] = true;
            }
        }
        
        if cluster_size >= 3 { // Only create zones for significant clusters
            clusters.push(cluster_center);
        }
    }
    
    clusters
}

