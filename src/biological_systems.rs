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
    all_transforms: Query<&mut Transform, (Without<Camera2d>, Without<Player>)>,
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

pub fn sample_ph(chemical_env: &ChemicalEnvironment, position: Vec2) -> f32 {
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

            let enemy_clone = enemy.clone();

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
                            lifetime: 0.0,        // FIXED: Initialize lifetime
                            max_lifetime: 4.0,    // FIXED: Set max lifetime to 4 seconds
                        },
                        Particle {
                            velocity: Vec2::new(0.0, 80.0) + offset.normalize() * 20.0,
                            lifetime: 0.0,
                            max_lifetime: 4.0,    // FIXED: Match ThermalParticle lifetime
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

pub fn signal_particle_spawning (
    mut commands: Commands,
    coordination_query: Query<(Entity, &CoordinationIndicator)>,
    transform_lookup: Query<&Transform, Without<Particle>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {

    // 9. COORDINATION INDICATORS (Swarm Cells) - Signal particle spawning
    if let Some(assets) = &assets {
        for (entity, coordination) in coordination_query.iter() {
            if let Ok(transform) = transform_lookup.get(entity) {
                let signal_pulse = (time.elapsed_secs() * 4.0).sin();
                if signal_pulse > 0.9 {
                    // Spawn coordination signal particles occasionally
                    for i in 0..2 {
                        let angle = (i as f32 / 2.0) * std::f32::consts::TAU + time.elapsed_secs();
                        let offset = Vec2::from_angle(angle) * coordination.communication_range;
                        
                        commands.spawn((
                            Sprite {
                                image: assets.particle_texture.clone(),
                                color: Color::srgba(0.4, 0.8, 1.0, 0.6),
                                custom_size: Some(Vec2::splat(3.0)),
                                ..default()
                            },
                            Transform::from_translation(transform.translation + offset.extend(0.0)),
                            Particle {
                                velocity: Vec2::ZERO,
                                lifetime: 0.0,
                                max_lifetime: 0.5,
                                size: 3.0,
                                fade_rate: 2.0,
                                bioluminescent: true,
                                drift_pattern: DriftPattern::Pulsing,
                            },
                        ));
                    }
                }
            }
        }
    }
}

pub fn virus_pulsing_animation (
    mut pulsing_query: Query<(&mut Transform, &mut Sprite, &PulsingAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, mut sprite, pulsing) in pulsing_query.iter_mut() {
        let pulse = (time.elapsed_secs() * pulsing.frequency).sin();
        let scale = 1.0 + pulse * pulsing.intensity;
        transform.scale = Vec3::splat(scale);
        
        let brightness = 0.8 + pulse * 0.2;
        sprite.color = Color::srgba(
            sprite.color.to_srgba().red * brightness,
            sprite.color.to_srgba().green * brightness,
            sprite.color.to_srgba().blue * brightness,
            sprite.color.to_srgba().alpha,
        );
    }
}

pub fn bacteria_flagella_animation (
    mut flagella_query: Query<(&mut Transform, &FlagellaAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, flagella) in flagella_query.iter_mut() {
        let undulation = (time.elapsed_secs() * flagella.undulation_speed).sin();
        transform.translation.x += undulation * flagella.amplitude * time.delta_secs();
        transform.rotation *= Quat::from_rotation_z(undulation * 0.1 * time.delta_secs());
    }
}

pub fn corruption_color_shift (
    mut corruption_query: Query<(&mut Sprite, &CorruptionEffect)>,
    time: Res<Time>,
) {
    for (mut sprite, corruption) in corruption_query.iter_mut() {
        let corruption_pulse = (time.elapsed_secs() * corruption.color_shift_speed).sin();
        let corruption_factor = corruption.intensity * (0.5 + corruption_pulse * 0.5);
        
        let base_color = sprite.color;
        sprite.color = Color::srgba(
            base_color.to_srgba().red * (1.0 - corruption_factor * 0.3),
            base_color.to_srgba().green * (1.0 + corruption_factor * 0.2),
            base_color.to_srgba().blue * (1.0 - corruption_factor * 0.5),
            base_color.to_srgba().alpha,
        );
    }
}

pub fn warning_flash_animation (
    mut warning_query: Query<(&mut Sprite, &WarningFlash)>,
    time: Res<Time>,
) {
    for (mut sprite, warning) in warning_query.iter_mut() {
        let flash = (time.elapsed_secs() * warning.flash_frequency).sin();
        sprite.color = if flash > 0.7 {
            warning.warning_color
        } else {
            Color::srgb(1.0, 0.7, 0.2)
        };
    }
}

pub fn offspring_wiggle_animation (
    mut transforms: Query<&mut Transform>,
    wiggle_query: Query<(Entity, &JuvenileWiggle)>,
    time: Res<Time>,
) {
    for (entity, wiggle) in wiggle_query.iter() {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            let wiggle_x = (time.elapsed_secs() * wiggle.wiggle_speed).sin();
            let wiggle_y = (time.elapsed_secs() * wiggle.wiggle_speed * 1.3).cos();
            transform.translation.x += wiggle_x * wiggle.amplitude * time.delta_secs();
            transform.translation.y += wiggle_y * wiggle.amplitude * 0.5 * time.delta_secs();
        }
    }
}

// 3. PSEUDOPOD ANIMATION (Protozoa) - Transform only
pub fn pseudopod_animation (
    mut transforms: Query<&mut Transform>,
    pseudopod_query: Query<(Entity, &PseudopodAnimation)>,
    time: Res<Time>,
) {
    for (entity, pseudopod) in pseudopod_query.iter() {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            let extension = (time.elapsed_secs() * pseudopod.extension_speed).sin();
            let stretch_x = 1.0 + extension * pseudopod.max_extension * 0.1;
            let stretch_y = 1.0 - extension * pseudopod.max_extension * 0.05;
            transform.scale = Vec3::new(stretch_x, stretch_y, 1.0);
        }
    }
}

// 4. GESTATION ANIMATION (Reproductive Vesicles) - Transform only
pub fn gestation_animation (
    mut transforms: Query<&mut Transform>,
    gestation_query: Query<(Entity, &GestationAnimation)>,
    time: Res<Time>,
) {
    for (entity, gestation) in gestation_query.iter() {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            let growth_pulse = (time.elapsed_secs() * gestation.pulse_frequency).sin();
            let growth = 1.0 + growth_pulse * gestation.growth_factor * 0.1;
            transform.scale = Vec3::splat(growth);
        }
    }
}

// 8. TOXIC AURA (Biofilm Colonies) - Particle spawning
pub fn toxic_aura_animation (
    mut commands: Commands,
    transform_lookup: Query<&Transform, Without<Particle>>,
    aura_query: Query<(Entity, &ToxicAura)>,
    time: Res<Time>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = &assets {
        for (entity, aura) in aura_query.iter() {
            if let Ok(transform) = transform_lookup.get(entity) {
                let pulse = (time.elapsed_secs() * aura.pulse_speed).sin();
                if pulse > 0.8 {
                    // Spawn toxic particles occasionally
                    for i in 0..3 {
                        let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
                        let offset = Vec2::from_angle(angle) * aura.radius;
                        
                        commands.spawn((
                            Sprite {
                                image: assets.particle_texture.clone(),
                                color: Color::srgba(0.6, 0.8, 0.3, 0.5),
                                custom_size: Some(Vec2::splat(4.0)),
                                ..default()
                            },
                            Transform::from_translation(transform.translation + offset.extend(0.0)),
                            Particle {
                                velocity: offset.normalize() * 20.0,
                                lifetime: 0.0,
                                max_lifetime: 2.0,
                                size: 4.0,
                                fade_rate: 0.5,
                                bioluminescent: false,
                                drift_pattern: DriftPattern::Floating,
                            },
                        ));
                    }
                }
            }
        }
    }
}


pub fn enhanced_coral_system(
    mut commands: Commands,
    mut coral_query: Query<(Entity, &mut EnhancedCoral, &mut Sprite, &mut Transform)>,
    mut player_query: Query<(&Transform, &mut Health, &mut ATP), (With<Player>, Without<EnhancedCoral>)>,
    mut enemy_query: Query<(&Transform, &mut Health), (With<Enemy>, Without<EnhancedCoral>, Without<Player>)>,
    mut chemical_environment: ResMut<ChemicalEnvironment>,
    ecosystem: Res<EcosystemState>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
    mut spawn_events: EventWriter<SpawnEnemy>,
) {
    *spawn_timer += time.delta_secs();
    
    // Process existing corals
    let mut corals_to_remove = Vec::new();
    
    for (coral_entity, mut coral, mut sprite, mut transform) in coral_query.iter_mut() {
        // Corruption spreads based on ecosystem health
        let corruption_factor = 1.0 - ecosystem.health;
        coral.corruption_level += coral.spread_rate * corruption_factor * time.delta_secs();
        coral.corruption_level = coral.corruption_level.clamp(0.0, 1.0);
        
        // Update coral health based on corruption
        coral.health -= coral.corruption_level * 10.0 * time.delta_secs();
        
        // Remove dead corals
        if coral.health <= 0.0 {
            corals_to_remove.push(coral_entity);
            continue;
        }
        
        // Visual updates based on coral type and corruption
        update_coral_visuals(&mut coral, &mut sprite, &mut transform, &time);
        
        // Apply gameplay effects
        apply_coral_effects(
            &coral,
            &transform,
            &mut player_query,
            &mut enemy_query,
            &mut chemical_environment,
            &mut commands,
            &assets,
            &mut spawn_events,
            &time,
        );
    }
    
    // Remove dead corals
    for entity in corals_to_remove {
        commands.entity(entity).insert(AlreadyDespawned).despawn();
    }
    
    // Spawn new coral formations periodically
    if *spawn_timer >= 25.0 {
        *spawn_timer = 0.0;
        spawn_coral_formation(&mut commands, &assets, &ecosystem);
    }
}

fn update_coral_visuals(
    coral: &mut EnhancedCoral,
    sprite: &mut Sprite,
    transform: &mut Transform,
    time: &Res<Time>,
) {
    // Base color modification based on corruption
    let corruption_color = Color::srgb(
        coral.original_color.to_srgba().red * (1.0 - coral.corruption_level * 0.7),
        coral.original_color.to_srgba().green * (1.0 - coral.corruption_level * 0.9),
        coral.original_color.to_srgba().blue * (1.0 - coral.corruption_level * 0.5),
    );
    
    // Type-specific visual effects
    match &coral.coral_type {
        CoralType::BioluminescentBeacon { pulse_frequency, .. } => {
            let pulse = (time.elapsed_secs() * pulse_frequency).sin() * 0.5 + 0.5;
            let glow_intensity = 0.6 + pulse * 0.4;
            sprite.color = Color::srgba(
                corruption_color.to_srgba().red * glow_intensity,
                corruption_color.to_srgba().green * glow_intensity,
                corruption_color.to_srgba().blue * glow_intensity,
                1.0,
            );
        }
        
        CoralType::OxygenProducer { .. } => {
            // Gentle breathing motion
            let breath = (time.elapsed_secs() * 2.0).sin() * 0.05 + 1.0;
            transform.scale = Vec3::splat(breath);
            
            // Healthy green tint (less corruption = more green)
            let health_factor = 1.0 - coral.corruption_level;
            sprite.color = Color::srgb(
                corruption_color.to_srgba().red * (1.0 - health_factor * 0.3),
                corruption_color.to_srgba().green + health_factor * 0.3,
                corruption_color.to_srgba().blue * (1.0 - health_factor * 0.2),
            );
        }
        
        CoralType::CorruptedColony { .. } => {
            // Sickly pulsing with corruption
            let corruption_pulse = (time.elapsed_secs() * 4.0).sin() * coral.corruption_level;
            sprite.color = Color::srgb(
                corruption_color.to_srgba().red + corruption_pulse * 0.3,
                corruption_color.to_srgba().green * (1.0 - corruption_pulse * 0.5),
                corruption_color.to_srgba().blue * (1.0 - corruption_pulse * 0.7),
            );
        }
        
        CoralType::AcidicFormation { acid_strength, .. } => {
            // Acidic yellow-green coloration
            let acid_factor = *acid_strength * coral.corruption_level;
            sprite.color = Color::srgb(
                corruption_color.to_srgba().red + acid_factor * 0.4,
                corruption_color.to_srgba().green + acid_factor * 0.6,
                corruption_color.to_srgba().blue * (1.0 - acid_factor * 0.8),
            );
        }
        
        _ => {
            sprite.color = corruption_color;
        }
    }
    
    // Warning bioluminescence for highly corrupted corals
    if coral.bioluminescent_warning && coral.corruption_level > 0.6 {
        let warning_pulse = (time.elapsed_secs() * 6.0 * coral.corruption_level).sin();
        let warning_alpha = 0.7 + warning_pulse * 0.3;
        sprite.color.set_alpha(warning_alpha);
    }
}

fn apply_coral_effects(
    coral: &EnhancedCoral,
    coral_transform: &Transform,
    player_query: &mut Query<(&Transform, &mut Health, &mut ATP), (With<Player>, Without<EnhancedCoral>)>,
    enemy_query: &mut Query<(&Transform, &mut Health), (With<Enemy>, Without<EnhancedCoral>, Without<Player>)>,
    chemical_environment: &mut ChemicalEnvironment,
    commands: &mut Commands,
    assets: &Option<Res<GameAssets>>,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    time: &Res<Time>,
) {
    // Apply effects based on coral type and corruption level
    match &coral.gameplay_effect {
        CoralEffect::Beneficial { healing_per_second, atp_per_second, ph_stabilization, oxygen_boost } => {
            // Only apply beneficial effects if corruption is low
            let effectiveness = (1.0 - coral.corruption_level).max(0.0);
            
            if let Ok((player_transform, mut player_health, mut player_atp)) = player_query.single_mut() {
                let distance = player_transform.translation.distance(coral_transform.translation);
                if distance < coral.influence_radius {
                    let proximity_factor = (coral.influence_radius - distance) / coral.influence_radius;
                    let effect_strength = effectiveness * proximity_factor;
                    
                    // Healing effect
                    if *healing_per_second > 0.0 {
                        let healing = (*healing_per_second * effect_strength * time.delta_secs()) as i32;
                        player_health.0 = (player_health.0 + healing).min(100);
                    }
                    
                    // ATP generation
                    if *atp_per_second > 0.0 {
                        let atp_gain = (*atp_per_second * effect_strength * time.delta_secs()) as u32;
                        player_atp.amount += atp_gain;
                    }
                    
                    // Spawn beneficial particles
                    if (time.elapsed_secs() % 2.0) < 0.1 && effect_strength > 0.5 {
                        spawn_beneficial_particles(commands, assets, coral_transform.translation);
                    }
                }
            }
            
            // Environmental cleanup - reduce nearby chemical contamination
            if *ph_stabilization > 0.0 {
                for zone in &mut chemical_environment.ph_zones {
                    let distance = zone.position.distance(coral_transform.translation.truncate());
                    if distance < coral.influence_radius * 1.5 {
                        // Gradually stabilize pH toward neutral
                        let stabilization_rate = *ph_stabilization * effectiveness * time.delta_secs();
                        if zone.ph_level < 7.0 {
                            zone.ph_level = (zone.ph_level + stabilization_rate).min(7.0);
                        } else if zone.ph_level > 7.0 {
                            zone.ph_level = (zone.ph_level - stabilization_rate).max(7.0);
                        }
                    }
                }
            }
        }
        
        CoralEffect::Harmful { damage_per_second, ph_reduction, spawns_enemies, corruption_spread } => {
            let harm_effectiveness = coral.corruption_level;
            
            // Damage nearby entities
            if *damage_per_second > 0.0 && harm_effectiveness > 0.3 {
                // Damage player
                if let Ok((player_transform, mut player_health, _)) = player_query.single_mut() {
                    let distance = player_transform.translation.distance(coral_transform.translation);
                    if distance < coral.influence_radius {
                        let proximity_factor = (coral.influence_radius - distance) / coral.influence_radius;
                        let damage = (*damage_per_second * harm_effectiveness * proximity_factor * time.delta_secs()) as i32;
                        player_health.0 -= damage;
                    }
                }
                
                // Damage nearby beneficial enemies (if any exist)
                for (enemy_transform, mut enemy_health) in enemy_query.iter_mut() {
                    let distance = enemy_transform.translation.distance(coral_transform.translation);
                    if distance < coral.influence_radius * 0.8 {
                        let damage = (*damage_per_second * harm_effectiveness * time.delta_secs()) as i32;
                        enemy_health.0 -= damage;
                    }
                }
            }
            
            // Spawn hostile microbes
            if *spawns_enemies > 0.0 && (time.elapsed_secs() % (10.0 / spawns_enemies)) < 0.1 {
                let spawn_pos = coral_transform.translation + Vec3::new(
                    (time.elapsed_secs() * 123.45).sin() * 40.0,
                    (time.elapsed_secs() * 67.89).cos() * 40.0,
                    0.0,
                );
                
                spawn_events.write(SpawnEnemy {
                    position: spawn_pos,
                    ai_type: EnemyAI::Chemotaxis {
                        target_chemical: ChemicalType::PlayerPheromones,
                        sensitivity: 1.5,
                        current_direction: Vec2::new(0.0, -1.0),
                    },
                    enemy_type: EnemyType::ViralParticle,
                });
            }
            
            // Create acidic zones
            if *ph_reduction > 0.0 && harm_effectiveness > 0.5 {
                // Add or intensify nearby acidic zones
                let acidic_position = coral_transform.translation.truncate();
                let mut zone_exists = false;
                
                for zone in &mut chemical_environment.ph_zones {
                    if zone.position.distance(acidic_position) < 80.0 {
                        zone.ph_level = (zone.ph_level - *ph_reduction * time.delta_secs()).max(3.0);
                        zone.intensity = (zone.intensity + 0.1 * time.delta_secs()).min(1.5);
                        zone_exists = true;
                        break;
                    }
                }
                
                if !zone_exists {
                    chemical_environment.ph_zones.push(crate::resources::ChemicalZone {
                        position: acidic_position,
                        radius: coral.influence_radius,
                        ph_level: 5.0,
                        intensity: 0.8,
                    });
                }
            }
            
            // Spawn harmful particles
            if (time.elapsed_secs() % 1.5) < 0.1 && harm_effectiveness > 0.4 {
                spawn_harmful_particles(commands, assets, coral_transform.translation, &coral.coral_type);
            }
        }
        
        CoralEffect::Neutral { provides_cover, navigation_aid } => {
            // Neutral corals could provide temporary buffs or serve as landmarks
            if *navigation_aid {
                // Could spawn navigation particles or provide map markers
            }
        }
    }
}

fn spawn_beneficial_particles(commands: &mut Commands, assets: &Option<Res<GameAssets>>, position: Vec3) {
    if let Some(assets) = assets {
        for i in 0..3 {
            let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
            let offset = Vec2::from_angle(angle) * 20.0;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgb(0.4, 1.0, 0.8),
                    custom_size: Some(Vec2::splat(4.0)),
                    ..default()
                },
                Transform::from_translation(position + offset.extend(0.0)),
                Particle {
                    velocity: offset * 0.8,
                    lifetime: 0.0,
                    max_lifetime: 2.0,
                    size: 4.0,
                    fade_rate: 1.0,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Floating,
                },
                BioluminescentParticle {
                    base_color: Color::srgb(0.4, 1.0, 0.8),
                    pulse_frequency: 3.0,
                    pulse_intensity: 0.7,
                    organic_motion: OrganicMotion {
                        undulation_speed: 2.0,
                        response_to_current: 0.6,
                    },
                },
            ));
        }
    }
}

fn spawn_harmful_particles(commands: &mut Commands, assets: &Option<Res<GameAssets>>, position: Vec3, coral_type: &CoralType) {
    if let Some(assets) = assets {
        let (particle_color, particle_count) = match coral_type {
            CoralType::AcidicFormation { .. } => (Color::srgb(0.9, 0.9, 0.3), 4),
            CoralType::CorruptedColony { .. } => (Color::srgb(0.8, 0.3, 0.3), 5),
            CoralType::ParasiticGrowth { .. } => (Color::srgb(0.6, 0.2, 0.8), 3),
            _ => (Color::srgb(0.7, 0.4, 0.4), 2),
        };
        
        for i in 0..particle_count {
            let angle = (i as f32 / particle_count as f32) * std::f32::consts::TAU;
            let offset = Vec2::from_angle(angle) * 25.0;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: particle_color,
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                Transform::from_translation(position + offset.extend(0.0)),
                Particle {
                    velocity: offset * 0.6,
                    lifetime: 0.0,
                    max_lifetime: 3.0,
                    size: 3.0,
                    fade_rate: 0.8,
                    bioluminescent: false,
                    drift_pattern: DriftPattern::Brownian,
                },
            ));
        }
    }
}

fn spawn_coral_formation(commands: &mut Commands, assets: &Option<Res<GameAssets>>, ecosystem: &EcosystemState) {
    if let Some(assets) = assets {
        let x = (rand::random::<f32>() - 0.5) * 1000.0;
        let y = (rand::random::<f32>() - 0.5) * 400.0;
        
        // Coral type based on ecosystem health
        let (coral_type, coral_effect, color, influence_radius) = if ecosystem.health > 0.7 {
            // Healthy ecosystem - beneficial corals
            let beneficial_types = [
                (
                    CoralType::FilterFeeder { purification_rate: 0.5, ph_stabilization: 0.3 },
                    CoralEffect::Beneficial { 
                        healing_per_second: 0.0, 
                        atp_per_second: 0.0, 
                        ph_stabilization: 0.3, 
                        oxygen_boost: 0.2 
                    },
                    Color::srgb(0.3, 0.8, 0.6),
                    100.0,
                ),
                (
                    CoralType::OxygenProducer { oxygen_output: 0.4, photosynthesis_rate: 0.6 },
                    CoralEffect::Beneficial { 
                        healing_per_second: 2.0, 
                        atp_per_second: 1.0, 
                        ph_stabilization: 0.0, 
                        oxygen_boost: 0.4 
                    },
                    Color::srgb(0.2, 0.9, 0.3),
                    80.0,
                ),
                (
                    CoralType::SymbioticReef { healing_rate: 3.0, atp_generation: 2.0 },
                    CoralEffect::Beneficial { 
                        healing_per_second: 3.0, 
                        atp_per_second: 2.0, 
                        ph_stabilization: 0.1, 
                        oxygen_boost: 0.1 
                    },
                    Color::srgb(0.6, 0.3, 0.8),
                    120.0,
                ),
            ];
            beneficial_types[rand::random::<u32>() as usize % beneficial_types.len()].clone()
        } else if ecosystem.health > 0.4 {
            // Neutral ecosystem - mixed corals
            (
                CoralType::BioluminescentBeacon { pulse_frequency: 2.0, detection_range: 150.0 },
                CoralEffect::Neutral { provides_cover: true, navigation_aid: true },
                Color::srgb(0.8, 0.7, 0.2),
                90.0,
            )
        } else {
            // Degraded ecosystem - harmful corals
            let harmful_types = [
                (
                    CoralType::CorruptedColony { toxin_production: 0.3, spawn_hostiles: true },
                    CoralEffect::Harmful { 
                        damage_per_second: 2.0, 
                        ph_reduction: 0.1, 
                        spawns_enemies: 0.5, 
                        corruption_spread: 0.2 
                    },
                    Color::srgb(0.8, 0.3, 0.3),
                    110.0,
                ),
                (
                    CoralType::AcidicFormation { acid_strength: 0.8, corrosion_rate: 0.4 },
                    CoralEffect::Harmful { 
                        damage_per_second: 1.5, 
                        ph_reduction: 0.3, 
                        spawns_enemies: 0.0, 
                        corruption_spread: 0.1 
                    },
                    Color::srgb(0.9, 0.8, 0.2),
                    95.0,
                ),
            ];
            harmful_types[rand::random::<u32>() as usize % harmful_types.len()].clone()
        };
        
        let initial_corruption = (1.0 - ecosystem.health) * 0.3;
        
        commands.spawn((
            Sprite {
                image: assets.enemy_texture.clone(),
                color,
                custom_size: Some(Vec2::splat(50.0 + rand::random::<f32>() * 30.0)),
                ..default()
            },
            Transform::from_xyz(x, y, -0.7),
            EnhancedCoral {
                coral_type,
                health: 100.0,
                corruption_level: initial_corruption,
                spread_rate: 0.02 + rand::random::<f32>() * 0.03,
                bioluminescent_warning: rand::random::<bool>(),
                original_color: color,
                size: Vec2::splat(50.0),
                gameplay_effect: coral_effect,
                influence_radius,
                last_spawn_time: 0.0,
            },
            ParallaxLayer { speed: 0.15, depth: -0.7 },
        ));
    }
}


// Contamination visualization system
pub fn contamination_visualization_system(
    mut commands: Commands,
    mut contamination_query: Query<(&mut ContaminationCloud, &mut Transform, &mut Sprite)>,
    chemical_environment: Res<ChemicalEnvironment>,
    ecosystem: Res<EcosystemState>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    let mut warning_spawns = Vec::new();
    
    for (mut cloud, mut transform, mut sprite) in contamination_query.iter_mut() {
        // Expand contamination based on ecosystem health
        let expansion_factor = 1.0 + (1.0 - ecosystem.health) * 2.0;
        transform.scale += Vec3::splat(cloud.expansion_rate * expansion_factor * time.delta_secs());
        
        // Increase toxicity over time
        cloud.toxicity_level += 0.1 * time.delta_secs();
        cloud.toxicity_level = cloud.toxicity_level.clamp(0.0, 2.0);
        
        // Visual effects based on contamination type
        match cloud.source_type {
            ContaminationType::IndustrialWaste => {
                sprite.color = Color::srgba(0.6, 0.4, 0.2, 0.4 + cloud.toxicity_level * 0.3);
            },
            ContaminationType::BiologicalToxin => {
                let pulse = (time.elapsed_secs() * 3.0).sin() * 0.2 + 0.8;
                sprite.color = Color::srgba(0.3, 0.8, 0.3, (0.3 + cloud.toxicity_level * 0.2) * pulse);
            },
            ContaminationType::RadioactiveSeepage => {
                let flicker = (time.elapsed_secs() * 10.0).sin() > 0.7;
                let intensity = if flicker { 1.0 } else { 0.6 };
                sprite.color = Color::srgba(0.9, 1.0, 0.3, (0.2 + cloud.toxicity_level * 0.3) * intensity);
            },
            ContaminationType::ChemicalSpill => {
                let rainbow_shift = (time.elapsed_secs() + transform.translation.x * 0.01).sin() * 0.5 + 0.5;
                sprite.color = Color::srgba(
                    0.8 + rainbow_shift * 0.2,
                    0.4 + rainbow_shift * 0.4,
                    0.9 - rainbow_shift * 0.3,
                    0.3 + cloud.toxicity_level * 0.2
                );
            },
            ContaminationType::PlasticPollution => {
                sprite.color = Color::srgba(0.7, 0.7, 0.7, 0.5 + cloud.toxicity_level * 0.3);
            },
        }
        
        // Collect warning particle spawn info instead of spawning directly
        if cloud.warning_intensity > 1.0 && (time.elapsed_secs() % 1.0) < 0.1 {
            if assets.is_some() {
                warning_spawns.push((transform.translation, cloud.source_type.clone()));
            }
        }
    }
    
    // Spawn warning particles outside the query loop
    if let Some(assets) = &assets {
        for (position, contamination_type) in warning_spawns {
            spawn_warning_particles(&mut commands, assets, position, &contamination_type);
        }
    }
}


fn spawn_warning_particles(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    contamination_type: &ContaminationType,
) {
    let (particle_color, particle_count) = match contamination_type {
        ContaminationType::RadioactiveSeepage => (Color::srgb(1.0, 1.0, 0.3), 8),
        ContaminationType::BiologicalToxin => (Color::srgb(0.3, 1.0, 0.3), 6),
        ContaminationType::ChemicalSpill => (Color::srgb(0.8, 0.2, 0.8), 10),
        _ => (Color::srgb(0.8, 0.4, 0.2), 5),
    };
    
    for i in 0..particle_count {
        let angle = (i as f32 / particle_count as f32) * std::f32::consts::TAU;
        let offset = Vec2::from_angle(angle) * 30.0;
        
        commands.spawn((
            Sprite {
                image: assets.particle_texture.clone(),
                color: particle_color,
                custom_size: Some(Vec2::splat(4.0)),
                ..default()
            },
            Transform::from_translation(position + offset.extend(0.1)),
            BioluminescentWarning {
                pattern_type: WarningPattern::RadialPulse,
                intensity: 1.0,
                pulse_frequency: 6.0,
                danger_level: 0.8,
            },
            Particle {
                velocity: offset * 0.8,
                lifetime: 0.0,
                max_lifetime: 3.0,
                size: 4.0,
                fade_rate: 0.8,
                bioluminescent: true,
                drift_pattern: DriftPattern::Pulsing,
            },
        ));
    }
}

// Microscopic debris storytelling system
pub fn microscopic_debris_system(
    mut commands: Commands,
    mut debris_query: Query<(Entity, &mut MicroscopicDebris, &mut Transform, &mut Sprite)>,
    player_query: Query<&Transform, (With<Player>, Without<MicroscopicDebris>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut story_timer: Local<f32>,
) {
    *story_timer += time.delta_secs();
    
    if let Ok(player_transform) = player_query.single() {
        let mut entities_to_despawn = Vec::new();
        
        // Update existing debris
        for (entity, mut debris, mut debris_transform, mut sprite) in debris_query.iter_mut() {
            debris.age += time.delta_secs();
            
            // Check if player is near enough to reveal story
            let distance = player_transform.translation.distance(debris_transform.translation);
            if distance < debris.reveal_distance {
                // Visual enhancement when story is revealed
                let reveal_intensity = (debris.reveal_distance - distance) / debris.reveal_distance;
                let enhanced_alpha = 0.4 + reveal_intensity * 0.6;
                sprite.color.set_alpha(enhanced_alpha);
                
                // Scale up when revealed
                let reveal_scale = 1.0 + reveal_intensity * 0.5;
                debris_transform.scale = Vec3::splat(reveal_scale);
                
                // Could trigger UI story text here
                if reveal_intensity > 0.8 && (time.elapsed_secs() % 3.0) < 0.1 {
                    // Spawn story fragment indicator
                    if let Some(assets) = &assets {
                        commands.spawn((
                            Text2d::new(&debris.story_fragment),
                            TextFont { font_size: 10.0, ..default() },
                            TextColor(Color::srgba(0.9, 0.9, 0.9, 0.8)),
                            Transform::from_translation(debris_transform.translation + Vec3::new(0.0, 20.0, 1.0)),
                            DamageText {
                                timer: 2.0,
                                velocity: Vec2::new(0.0, 20.0),
                            },
                        ));
                    }
                }
            }
            
            // Age-based visual changes
            match &mut debris.debris_type {
                DebrisType::BiologicalRemains { decay_level, .. } => {
                    *decay_level += time.delta_secs() * 0.1;
                    let decay_color = Color::srgb(
                        0.6 * (1.0 - *decay_level * 0.5),
                        0.4 * (1.0 - *decay_level * 0.3),
                        0.3 * (1.0 - *decay_level * 0.7),
                    );
                    sprite.color = decay_color;
                }
                DebrisType::MetalParticle { oxidation_level } => {
                    *oxidation_level += time.delta_secs() * 0.05;
                    let rust_color = Color::srgb(
                        0.7 + *oxidation_level * 0.3,
                        0.4 * (1.0 - *oxidation_level * 0.5),
                        0.2 * (1.0 - *oxidation_level * 0.8),
                    );
                    sprite.color = rust_color;
                }
                DebrisType::SyntheticFiber { weathering, .. } => {
                    *weathering += time.delta_secs() * 0.02;
                    sprite.color.set_alpha(0.8 * (1.0 - *weathering * 0.6));
                }
                _ => {}
            }
            
            // Mark very old debris for removal
            if debris.age > 60.0 {
                entities_to_despawn.push(entity);
            }
        }
        
        // Remove old debris entities
        for entity in entities_to_despawn {
            commands.entity(entity)
                .insert(AlreadyDespawned)
                .despawn();
        }
    }
    
    // Spawn new debris periodically
    if *story_timer >= 8.0 {
        *story_timer = 0.0;
        spawn_story_debris(&mut commands, &assets);
    }
}

fn spawn_story_debris(commands: &mut Commands, assets: &Option<Res<GameAssets>>) {
    if let Some(assets) = assets {
        let x = (rand::random::<f32>() - 0.5) * 1200.0;
        let y = 400.0 + rand::random::<f32>() * 100.0;
        
        let debris_stories = [
            ("Microplastic fragment from ocean surface", DebrisType::PlasticFragment { 
                size: 2.0, 
                color: Color::srgb(0.8, 0.8, 0.9) 
            }),
            ("Industrial metal particle, heavily corroded", DebrisType::MetalParticle { 
                oxidation_level: 0.3 
            }),
            ("Chemical residue from agricultural runoff", DebrisType::ChemicalResidue { 
                compound_type: "Pesticide".to_string() 
            }),
            ("Decomposing plankton, ecosystem disruption", DebrisType::BiologicalRemains { 
                species: "Phytoplankton".to_string(), 
                decay_level: 0.2 
            }),
            ("Synthetic fiber from clothing waste", DebrisType::SyntheticFiber { 
                material: "Polyester".to_string(), 
                weathering: 0.1 
            }),
            ("Dead coral fragment, bleaching event", DebrisType::BiologicalRemains { 
                species: "Staghorn Coral".to_string(), 
                decay_level: 0.8 
            }),
            ("Paint chip from ship hull", DebrisType::ChemicalResidue { 
                compound_type: "Lead-based Paint".to_string() 
            }),
            ("Tire particle from road runoff", DebrisType::PlasticFragment { 
                size: 1.5, 
                color: Color::srgb(0.2, 0.2, 0.2) 
            }),
        ];
        
        let (story, debris_type) = &debris_stories[rand::random::<u32>() as usize % debris_stories.len()];
        
        let (texture, color, size) = match debris_type {
            DebrisType::PlasticFragment { size, color } => {
                (assets.particle_texture.clone(), *color, *size)
            },
            DebrisType::MetalParticle { .. } => {
                (assets.particle_texture.clone(), Color::srgb(0.6, 0.6, 0.7), 2.5)
            },
            DebrisType::ChemicalResidue { .. } => {
                (assets.particle_texture.clone(), Color::srgb(0.8, 0.7, 0.3), 3.0)
            },
            DebrisType::BiologicalRemains { .. } => {
                (assets.particle_texture.clone(), Color::srgb(0.5, 0.4, 0.3), 4.0)
            },
            DebrisType::SyntheticFiber { .. } => {
                (assets.projectile_texture.clone(), Color::srgb(0.7, 0.3, 0.8), 1.5)
            },
        };
        
        commands.spawn((
            Sprite {
                image: texture,
                color,
                custom_size: Some(Vec2::splat(size * 2.0)),
                ..default()
            },
            Transform::from_xyz(x, y, -0.3),
            MicroscopicDebris {
                debris_type: debris_type.clone(),
                story_fragment: story.to_string(),
                age: 0.0,
                reveal_distance: 60.0,
            },
            Particle {
                velocity: Vec2::new(0.0, -40.0),
                lifetime: 0.0,
                max_lifetime: 45.0,
                size: size * 2.0,
                fade_rate: 0.8,
                bioluminescent: false,
                drift_pattern: DriftPattern::Brownian,
            },
            ParallaxLayer { speed: 0.3, depth: -0.3 },
        ));
    }
}

// Bioluminescent warning system
pub fn bioluminescent_warning_system(
    mut warning_query: Query<(&mut BioluminescentWarning, &mut Transform, &mut Sprite)>,
    chemical_environment: Res<ChemicalEnvironment>,
    ecosystem: Res<EcosystemState>,
    time: Res<Time>,
) {
    for (mut warning, mut transform, mut sprite) in warning_query.iter_mut() {
        // Intensify warnings based on ecosystem danger
        let danger_multiplier = 1.0 + (1.0 - ecosystem.health) * 2.0;
        warning.intensity *= danger_multiplier.clamp(1.0, 3.0);
        
        match warning.pattern_type {
            WarningPattern::RadialPulse => {
                let pulse = (time.elapsed_secs() * warning.pulse_frequency).sin();
                let scale = 1.0 + pulse * 0.3 * warning.intensity;
                transform.scale = Vec3::splat(scale);
                
                let alpha = 0.3 + (pulse * 0.5 + 0.5) * warning.intensity * 0.7;
                sprite.color.set_alpha(alpha);
            },
            
            WarningPattern::DirectionalStrobe => {
                let strobe = (time.elapsed_secs() * warning.pulse_frequency * 2.0).sin() > 0.5;
                let alpha = if strobe { warning.intensity } else { 0.2 };
                sprite.color.set_alpha(alpha);
                
                // Point toward danger
                transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 2.0);
            },
            
            WarningPattern::ColorShift => {
                let shift = (time.elapsed_secs() * warning.pulse_frequency).sin() * 0.5 + 0.5;
                sprite.color = Color::srgb(
                    0.3 + shift * 0.7,
                    0.8 - shift * 0.5,
                    0.2 + shift * 0.3,
                );
                sprite.color.set_alpha(warning.intensity * 0.6);
            },
            
            WarningPattern::FlashingGrid => {
                let grid_flash = ((time.elapsed_secs() * warning.pulse_frequency).sin() * 
                                (time.elapsed_secs() * warning.pulse_frequency * 1.3).cos()) > 0.3;
                let alpha = if grid_flash { warning.intensity * 0.8 } else { 0.1 };
                sprite.color.set_alpha(alpha);
            },
            
            WarningPattern::ChaotticFlicker => {
                let chaos = (time.elapsed_secs() * warning.pulse_frequency * 3.0 + 
                           transform.translation.x * 0.01).sin() *
                          (time.elapsed_secs() * warning.pulse_frequency * 2.3 + 
                           transform.translation.y * 0.01).cos();
                let alpha = (0.2 + chaos.abs() * warning.intensity * 0.8).clamp(0.0, 1.0);
                sprite.color.set_alpha(alpha);
                
                // Chaotic movement
                let jitter = Vec2::new(chaos * 5.0, chaos * 3.0);
                transform.translation += jitter.extend(0.0) * time.delta_secs();
            },
        }
    }
}

// Environmental narrative triggers
pub fn environmental_narrative_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    ecosystem: Res<EcosystemState>,
    chemical_environment: Res<ChemicalEnvironment>,
    time: Res<Time>,
    mut narrative_timer: Local<f32>,
    mut last_ecosystem_health: Local<f32>,
) {
    *narrative_timer += time.delta_secs();
    
    if let Ok(player_transform) = player_query.single() {
        // Detect major ecosystem changes
        let health_change = ecosystem.health - *last_ecosystem_health;
        
        if health_change.abs() > 0.2 && *narrative_timer > 5.0 {
            *narrative_timer = 0.0;
            
            let narrative_text = if health_change > 0.0 {
                "Ecosystem recovery detected..."
            } else {
                "Environmental degradation accelerating..."
            };
            
            // Spawn environmental narrative text
            commands.spawn((
                Text2d::new(narrative_text),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgba(0.8, 1.0, 0.9, 0.9)),
                Transform::from_translation(player_transform.translation + Vec3::new(0.0, 100.0, 2.0)),
                DamageText {
                    timer: 4.0,
                    velocity: Vec2::new(0.0, 30.0),
                },
            ));
        }
        
        *last_ecosystem_health = ecosystem.health;
        
        // Spawn contamination events based on chemical environment
        let avg_ph = chemical_environment.ph_zones.iter()
            .map(|z| z.ph_level * z.intensity)
            .sum::<f32>() / chemical_environment.ph_zones.len().max(1) as f32;
            
        if (avg_ph < 5.5 || avg_ph > 8.5) && *narrative_timer > 15.0 {
            spawn_contamination_event(&mut commands, player_transform.translation, avg_ph);
            *narrative_timer = 0.0;
        }
    }
}

fn spawn_contamination_event(commands: &mut Commands, player_pos: Vec3, ph_level: f32) {
    let contamination_type = if ph_level < 6.0 {
        ContaminationType::IndustrialWaste
    } else if ph_level > 8.0 {
        ContaminationType::ChemicalSpill
    } else {
        ContaminationType::BiologicalToxin
    };
    
    let spawn_pos = player_pos + Vec3::new(
        (rand::random::<f32>() - 0.5) * 400.0,
        200.0 + rand::random::<f32>() * 100.0,
        -0.5
    );
    
    commands.spawn((
        Sprite {
            color: Color::srgba(0.6, 0.3, 0.3, 0.4),
            custom_size: Some(Vec2::splat(80.0)),
            ..default()
        },
        Transform::from_translation(spawn_pos),
        ContaminationCloud {
            toxicity_level: 0.5,
            expansion_rate: 0.2,
            source_type: contamination_type,
            warning_intensity: 1.0,
        },
        ParallaxLayer { speed: 0.1, depth: -0.5 },
    ));
}


