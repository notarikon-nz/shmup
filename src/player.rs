use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy::input::gamepad::*;
use bevy::window::WindowResolution;
use bevy::sprite::Anchor;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use crate::lighting::PerformantLightingPlugin;

use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::systems::*;
use crate::input::*;
use crate::physics::*;

/// Enhanced player movement with fluid dynamics and organic motion
pub fn biological_movement_system(
    mut player_query: Query<(&mut Transform, &mut FluidDynamics, &Player)>,
    input_manager: Res<InputManager>, // Changed from InputState
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut fluid, player)) = player_query.single_mut() {
        // Get movement vector from input manager
        let movement = input_manager.movement_vector(); // Smooth analog movement
        
        // Player input creates thrust against fluid resistance
        let thrust = movement * player.speed * 2.0;
        
        // Sample current from fluid field
        let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
        let current = sample_current(&fluid_environment, grid_pos);
        
        // Physics integration with biological properties
        let drag = fluid.velocity * -fluid.viscosity_resistance;
        let buoyancy = Vec2::new(0.0, fluid.buoyancy);
        let current_force = current * fluid.current_influence;
        
        let acceleration = thrust + current_force + drag + buoyancy;
        fluid.velocity += acceleration * time.delta_secs();
        
        // Apply velocity to position with organic damping
        transform.translation += fluid.velocity.extend(0.0) * time.delta_secs();
        
        // Boundary conditions with surface tension effect
        transform.translation.x = transform.translation.x.clamp(-600.0, 600.0);
        transform.translation.y = transform.translation.y.clamp(-350.0, 350.0);
        
        // Organic roll motion based on fluid flow
        let flow_influence = (fluid.velocity.x + current.x) * 0.001;
        let target_roll = -movement.x * player.roll_factor + flow_influence;
        transform.rotation = transform.rotation.lerp(
            Quat::from_rotation_z(target_roll),
            time.delta_secs() * 6.0
        );
    }
}

// Bioluminescent trail system (replaces engine particles)
pub fn spawn_bioluminescent_trail(
    mut commands: Commands,
    player_query: Query<&Transform, With<EngineTrail>>,
    input_manager: Res<InputManager>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut trail_segments: Local<Vec<Vec3>>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer -= time.delta_secs();
    
    if *spawn_timer <= 0.0 {
        for transform in player_query.iter() {
            let intensity = input_manager.movement_vector().length().max(0.2);
            
            // Add new trail segment
            trail_segments.push(transform.translation + Vec3::new(0.0, -18.0, -0.1));
            
            // Keep only last 15 segments
            if trail_segments.len() > 15 {
                trail_segments.remove(0);
            }
            
            // Spawn connected membrane segments
            if let Some(assets) = &assets {
                for (i, &segment_pos) in trail_segments.iter().enumerate() {
                    let age = i as f32 / trail_segments.len() as f32;
                    let alpha = age * 0.6 * intensity;
                    let width = (age * 8.0 + 2.0) * intensity;
                    
                    commands.spawn((
                        Sprite {
                            image: assets.particle_texture.clone(),
                            color: Color::srgba(0.3, 0.9, 1.0, alpha),
                            custom_size: Some(Vec2::splat(width)),
                            ..default()
                        },
                        Transform::from_translation(segment_pos),
                        Particle {
                            velocity: Vec2::ZERO,
                            lifetime: 0.0,
                            max_lifetime: 0.8,
                            size: width,
                            fade_rate: 2.0,
                            bioluminescent: true,
                            drift_pattern: DriftPattern::Floating,
                        },
                    ));
                }
            }
        }
        
        *spawn_timer = 0.05;
    }
}

// Update biological effects (replaces update_player_effects)
pub fn update_biological_effects(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &Transform), With<Player>>,
    mut cell_wall_query: Query<(Entity, &mut CellWallReinforcement)>,
    mut cell_wall_visual_query: Query<(Entity, &mut Transform, &mut Sprite), (With<CellWallVisual>, Without<Player>, Without<AlreadyDespawned>)>,
    mut flagella_query: Query<(Entity, &mut FlagellaBoost)>,
    mut symbiotic_query: Query<(Entity, &mut SymbioticMultiplier)>,
    mut mitochondria_query: Query<(Entity, &mut MitochondriaOvercharge)>,
    mut photosynthesis_query: Query<(Entity, &mut PhotosynthesisActive, &mut Health)>,
    mut chemotaxis_query: Query<(Entity, &mut ChemotaxisActive)>,
    mut osmoregulation_query: Query<(Entity, &mut OsmoregulationActive)>,
    mut binary_fission_query: Query<(Entity, &mut BinaryFissionActive)>,
    mut game_score: ResMut<GameScore>,
    time: Res<Time>,
) {
    if let Ok((_, mut player, _player_transform)) = player_query.single_mut() {
        player.invincible_timer = (player.invincible_timer - time.delta_secs()).max(0.0);
    }

    // Update cell wall reinforcement
    let mut cell_wall_active = false;
    for (entity, mut cell_wall) in cell_wall_query.iter_mut() {
        cell_wall.timer -= time.delta_secs();
        cell_wall.alpha_timer += time.delta_secs();
        cell_wall_active = true;
        
        if cell_wall.timer <= 0.0 {
            commands.entity(entity).remove::<CellWallReinforcement>();
            cell_wall_active = false;
        }
    }

    // Update cell wall visual with organic pulsing
    if let Ok((_player_entity, _, player_transform)) = player_query.single() {
        if cell_wall_active {
            if let Ok((_, mut cell_wall_transform, mut cell_wall_sprite)) = cell_wall_visual_query.single_mut() {
                // Follow player position
                cell_wall_transform.translation = player_transform.translation;
                
                // Organic pulsing effect
                let pulse = (time.elapsed_secs() * 3.0).sin() * 0.15 + 0.85;
                cell_wall_transform.scale = Vec3::splat(pulse);
                
                // Bioluminescent breathing alpha
                let alpha = 0.3 + (time.elapsed_secs() * 2.0).sin().abs() * 0.2;
                cell_wall_sprite.color = Color::srgba(0.4, 1.0, 0.8, alpha);
                
                // Organic rotation
                cell_wall_transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.3);
            } else {
                // Create new cell wall visual
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.4, 1.0, 0.8, 0.4),
                        custom_size: Some(Vec2::splat(70.0)),
                        ..default()
                    },
                    Transform::from_translation(player_transform.translation),
                    CellWallVisual,
                ));
            }
        } else {
            // Remove cell wall visual when expired
            for (cell_wall_visual_entity, _, _) in cell_wall_visual_query.iter() {
                commands.entity(cell_wall_visual_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
            }
        }
    }

    // Update other biological effects
    for (entity, mut flagella) in flagella_query.iter_mut() {
        flagella.timer -= time.delta_secs();
        if flagella.timer <= 0.0 {
            commands.entity(entity).remove::<FlagellaBoost>();
        }
    }

    for (entity, mut symbiotic) in symbiotic_query.iter_mut() {
        symbiotic.timer -= time.delta_secs();
        game_score.score_multiplier = symbiotic.multiplier;
        game_score.multiplier_timer = symbiotic.timer;
        
        if symbiotic.timer <= 0.0 {
            game_score.score_multiplier = 1.0; // Add this line
            game_score.multiplier_timer = 0.0;  // Add this line
            commands.entity(entity).remove::<SymbioticMultiplier>();
        }
    }

    for (entity, mut mitochondria) in mitochondria_query.iter_mut() {
        mitochondria.timer -= time.delta_secs();
        if mitochondria.timer <= 0.0 {
            commands.entity(entity).remove::<MitochondriaOvercharge>();
        }
    }

    // New biological effects
    for (entity, mut photosynthesis, mut health) in photosynthesis_query.iter_mut() {
        photosynthesis.timer -= time.delta_secs();
        
        // Heal over time from photosynthesis
        health.0 = (health.0 + (photosynthesis.energy_per_second * time.delta_secs()) as i32).min(100);
        
        if photosynthesis.timer <= 0.0 {
            commands.entity(entity).remove::<PhotosynthesisActive>();
        }
    }

    for (entity, mut chemotaxis) in chemotaxis_query.iter_mut() {
        chemotaxis.timer -= time.delta_secs();
        if chemotaxis.timer <= 0.0 {
            commands.entity(entity).remove::<ChemotaxisActive>();
        }
    }

    for (entity, mut osmoregulation) in osmoregulation_query.iter_mut() {
        osmoregulation.timer -= time.delta_secs();
        if osmoregulation.timer <= 0.0 {
            commands.entity(entity).remove::<OsmoregulationActive>();
        }
    }

    for (entity, mut binary_fission) in binary_fission_query.iter_mut() {
        binary_fission.timer -= time.delta_secs();
        binary_fission.clone_timer -= time.delta_secs();
        
        if binary_fission.clone_timer <= 0.0 {
            // Spawn clone projectile
            // This would be implemented in the weapon system
            binary_fission.clone_timer = 0.5; // Reset clone timer
        }
        
        if binary_fission.timer <= 0.0 {
            commands.entity(entity).remove::<BinaryFissionActive>();
        }
    }
}
