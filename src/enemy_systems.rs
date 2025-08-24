use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;
use std::collections::HashMap;

// Enhanced enemy movement system with biological behaviors
pub fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    colony_leader_query: Query<&Transform, (With<ColonyLeader>, Without<Enemy>, Without<Player>)>,
    fluid_environment: Res<FluidEnvironment>,
    chemical_environment: Res<ChemicalEnvironment>,
    time: Res<Time>,
) {
    let player_pos = player_query.single().ok().map(|t| t.translation.truncate());
    
    for (mut transform, mut enemy) in enemy_query.iter_mut() {
        let mut enemy_clone = enemy.clone();

        match &mut enemy_clone.ai_type {
            EnemyAI::Static => {},
            
            EnemyAI::Linear { direction } => {
                // Enhanced linear movement with organic undulation
                let undulation = Vec2::new(
                    (time.elapsed_secs() * 1.8 + transform.translation.y * 0.008).sin() * 8.0,
                    0.0,
                );
                
                // Apply fluid current influence
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                
                let movement = direction.extend(0.0) + undulation.extend(0.0) + (current * 0.3).extend(0.0);
                transform.translation += movement * enemy.speed * time.delta_secs();
            },
            
            EnemyAI::Sine { amplitude, frequency, phase } => {
                *phase += time.delta_secs() * *frequency;
                transform.translation.y -= enemy.speed * time.delta_secs();
                
                // Enhanced sine wave with organic variation
                let organic_variation = (time.elapsed_secs() * 0.4).sin() * 0.15;
                let actual_amplitude = *amplitude * (1.0 + organic_variation);
                
                // Apply fluid influence to sine movement
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                
                transform.translation.x += actual_amplitude * phase.sin() * time.delta_secs();
                transform.translation += (current * 0.2).extend(0.0) * time.delta_secs();
            },
            
            EnemyAI::MiniBoss { pattern: _, timer } => {
                *timer += time.delta_secs();
                transform.translation.y -= enemy.speed * 0.6 * time.delta_secs();
                
                // Boss movement patterns influenced by chemical environment
                let pattern_movement = Vec2::new(
                    (*timer * 0.8).sin() * 100.0,
                    (*timer * 0.5).cos() * 30.0,
                );
                transform.translation += pattern_movement.extend(0.0) * time.delta_secs();
            },
            
            EnemyAI::Kamikaze { target_pos, dive_speed, acquired_target } => {
                if let Some(player_position) = player_pos {
                    if !*acquired_target {
                        *target_pos = player_position;
                        *acquired_target = true;
                    }
                    
                    let direction = (*target_pos - transform.translation.truncate()).normalize_or_zero();
                    transform.translation += direction.extend(0.0) * *dive_speed * time.delta_secs();
                    
                    // Rotate to face movement direction with organic wobble
                    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    let wobble = (time.elapsed_secs() * 8.0).sin() * 0.1;
                    transform.rotation = Quat::from_rotation_z(angle + wobble);
                } else {
                    // No player, drift with current
                    let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                    let current = sample_current(&fluid_environment, grid_pos);
                    transform.translation += (current + Vec2::new(0.0, -50.0)).extend(0.0) * time.delta_secs();
                }
            },
            
            EnemyAI::Turret { rotation, shoot_timer: _, detection_range: _ } => {
                // Turrets sway gently with currents but don't move
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                let sway = current.x * 0.001;
                
                if let Some(player_position) = player_pos {
                    let direction = player_position - transform.translation.truncate();
                    let target_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    let angle_diff = (target_angle - *rotation + std::f32::consts::PI) % std::f32::consts::TAU - std::f32::consts::PI;
                    *rotation += angle_diff.clamp(-1.8 * time.delta_secs(), 1.8 * time.delta_secs());
                    transform.rotation = Quat::from_rotation_z(*rotation + sway);
                }
            },
            
            EnemyAI::Formation { formation_id: _, position_in_formation, leader_offset, formation_timer } => {
                *formation_timer += time.delta_secs();
                
                // Find colony leader position
                let leader_pos = colony_leader_query.iter()
                    .find(|leader_transform| {
                        // PLACEHOLDER
                        // In a real implementation, you'd match formation_id
                        true
                    })
                    .map(|t| t.translation.truncate())
                    .unwrap_or_else(|| Vec2::new(0.0, 400.0));
                
                let target_pos = leader_pos + *leader_offset + *position_in_formation;
                let direction = (target_pos - transform.translation.truncate()).normalize_or_zero();
                
                // Apply fluid influence to formation movement
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                
                let movement = direction + current * 0.4;
                transform.translation += movement.extend(0.0) * enemy.speed * time.delta_secs();
            },
            
            EnemyAI::Spawner { spawn_timer: _, spawn_rate: _, minions_spawned: _, max_minions: _ } => {
                // Spawners move slowly and respond strongly to currents
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                
                let movement = Vec2::new(0.0, -enemy.speed * 0.7) + current * 0.8;
                transform.translation += movement.extend(0.0) * time.delta_secs();
            },
            
            // New biological AI behaviors
            EnemyAI::Chemotaxis { target_chemical, sensitivity, current_direction } => {
                if let Some(player_position) = player_pos {
                    let player_distance = transform.translation.distance(player_position.extend(0.0));
                    
                    if player_distance < 350.0 {
                        // Follow chemical gradient toward player
                        let direction_to_player = (player_position - transform.translation.truncate()).normalize_or_zero();
                        
                        // Chemical gradient strength (simplified model)
                        let chemical_strength = match target_chemical {
                            ChemicalType::PlayerPheromones => 1.0 / (player_distance * 0.01 + 1.0),
                            ChemicalType::NutrientGradient => 0.5,
                            ChemicalType::OxygenSeeker => sample_oxygen(&chemical_environment, transform.translation.truncate()),
                            ChemicalType::ToxinAvoidance => 1.0 - sample_ph_toxicity(&chemical_environment, transform.translation.truncate()),
                        };
                        
                        let influence = chemical_strength * *sensitivity;
                        *current_direction = current_direction.lerp(direction_to_player, influence * time.delta_secs());
                        
                        // Add some organic randomness
                        let random_influence = Vec2::new(
                            (time.elapsed_secs() * 3.2 + transform.translation.x * 0.01).sin() * 0.2,
                            (time.elapsed_secs() * 2.7 + transform.translation.y * 0.01).cos() * 0.2,
                        );
                        *current_direction = (*current_direction + random_influence).normalize_or_zero();
                        
                        transform.translation += current_direction.extend(0.0) * enemy.speed * time.delta_secs();
                    } else {
                        // Random swimming when no strong chemical signal
                        let random_turn = (time.elapsed_secs() * 2.5 + transform.translation.x * 0.005).sin();
                        *current_direction = Vec2::from_angle(current_direction.to_angle() + random_turn * 0.8 * time.delta_secs());
                        transform.translation += current_direction.extend(0.0) * enemy.speed * 0.6 * time.delta_secs();
                    }
                } else {
                    // No player detected, drift with currents
                    let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                    let current = sample_current(&fluid_environment, grid_pos);
                    transform.translation += (current + Vec2::new(0.0, -enemy.speed * 0.3)).extend(0.0) * time.delta_secs();
                }
            },
            
            EnemyAI::CellDivision { division_threshold, division_timer, has_divided } => {
                // Normal movement until ready to divide
                transform.translation.y -= enemy.speed * 0.8 * time.delta_secs();
                
                if enemy.health as f32 <= *division_threshold && !*has_divided {
                    *division_timer -= time.delta_secs();
                    
                    // Visual indication of impending division
                    let division_wobble = (*division_timer * 10.0).sin() * 5.0;
                    transform.translation.x += division_wobble * time.delta_secs();
                    
                    if *division_timer <= 0.0 {
                        *has_divided = true;
                        // Division would be handled in spawning system
                    }
                }
            },
            
            EnemyAI::SymbioticPair { partner_entity, bond_distance, sync_timer } => {
                *sync_timer += time.delta_secs();
                
                // Synchronized swimming pattern
                let sync_movement = Vec2::new(
                    (*sync_timer * 1.8).sin() * *bond_distance * 0.3,
                    (*sync_timer * 1.2).cos() * *bond_distance * 0.2,
                );
                
                // Base downward movement
                let base_movement = Vec2::new(0.0, -enemy.speed * 0.9);
                
                // Apply fluid currents
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                
                let total_movement = base_movement + sync_movement + current * 0.5;
                transform.translation += total_movement.extend(0.0) * time.delta_secs();
                
                // Partner coordination would be implemented here if partner exists
                if partner_entity.is_some() {
                    // In a full implementation, you'd coordinate with the partner's position
                }
            },
            
            EnemyAI::FluidFlow { flow_sensitivity, base_direction } => {
                // Strongly follows fluid currents
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                
                // Blend base direction with current flow (stronger influence)
                let flow_influence = current * *flow_sensitivity * time.delta_secs() * 3.0; // Increased multiplier
                *base_direction = (*base_direction + flow_influence).normalize_or_zero();
                
                // Apply movement with very strong current response
                let movement = *base_direction * enemy.speed * 0.3 + current * 1.5; // Follow current strongly
                transform.translation += movement.extend(0.0) * time.delta_secs();
                
                // Rotate to face movement direction with current influence
                let angle = (current.x * 0.7 + base_direction.x * 0.3).atan2(current.y * 0.7 + base_direction.y * 0.3) - std::f32::consts::FRAC_PI_2;
                transform.rotation = Quat::from_rotation_z(angle);
            },
        }
        
        // Apply chemical environment effects to all enemies
        apply_chemical_effects(&mut transform, &enemy, &chemical_environment, time.delta_secs());
        
        // Update the enemy component
        *enemy = enemy_clone;
    }
}

// Apply chemical environment effects to enemies
fn apply_chemical_effects(
    transform: &mut Transform,
    enemy: &Enemy,
    chemical_env: &ChemicalEnvironment,
    delta_time: f32,
) {
    if enemy.chemical_signature.responds_to_pheromones {
        let local_ph = sample_ph(&chemical_env, transform.translation.truncate());
        
        // Enemies avoid unfavorable pH zones
        let ph_difference = (local_ph - enemy.chemical_signature.ph_preference).abs();
        if ph_difference > 1.2 {
            let avoidance_direction = if local_ph > enemy.chemical_signature.ph_preference {
                Vec2::new(-0.5, 0.2) // Move away from alkaline zones
            } else {
                Vec2::new(0.5, 0.2) // Move away from acidic zones
            };
            
            let avoidance_strength = (ph_difference - 1.2) * 40.0;
            transform.translation += (avoidance_direction * avoidance_strength).extend(0.0) * delta_time;
        }
        
        // Oxygen level response
        let oxygen_level = sample_oxygen(&chemical_env, transform.translation.truncate());
        if oxygen_level < enemy.chemical_signature.oxygen_tolerance {
            // Move toward higher oxygen areas (simplified - move upward)
            transform.translation.y += 20.0 * delta_time;
        }
    }
}

// Spawner enemy behavior with biological reproduction
pub fn update_spawner_enemies(
    mut commands: Commands,
    mut spawner_query: Query<(Entity, &Transform, &mut Enemy)>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    for (_entity, transform, mut enemy) in spawner_query.iter_mut() {

        let enemy_clone = enemy.clone();

        if let EnemyAI::Spawner { spawn_timer, spawn_rate, minions_spawned, max_minions } = &mut enemy.ai_type {
            *spawn_timer -= time.delta_secs();
            
            if *spawn_timer <= 0.0 && *minions_spawned < *max_minions {
                // Biological spawning pattern - offspring spread out naturally
                let spawn_angle = (*minions_spawned as f32 * 1.2) + (time.elapsed_secs() * 0.5).sin();
                let spawn_distance = 25.0 + (time.elapsed_secs() * 2.0).cos() * 10.0;
                let spawn_offset = Vec2::from_angle(spawn_angle) * spawn_distance;
                
                // Choose AI based on parent type and environment
                let offspring_ai = match enemy_clone.enemy_type {
                    EnemyType::ReproductiveVesicle => {
                        if *minions_spawned % 2 == 0 {
                            EnemyAI::Chemotaxis {
                                target_chemical: ChemicalType::PlayerPheromones,
                                sensitivity: 1.0,
                                current_direction: Vec2::new(0.0, -1.0),
                            }
                        } else {
                            EnemyAI::FluidFlow {
                                flow_sensitivity: 1.5,
                                base_direction: Vec2::new(spawn_angle.cos() * 0.3, -0.9).normalize(),
                            }
                        }
                    }
                    _ => EnemyAI::Linear { 
                        direction: Vec2::new(spawn_angle.cos() * 0.4, -0.8).normalize() 
                    },
                };
                
                spawn_events.write(SpawnEnemy {
                    position: transform.translation + spawn_offset.extend(0.0),
                    ai_type: offspring_ai,
                    enemy_type: EnemyType::Offspring,
                });
                
                *minions_spawned += 1;
                *spawn_timer = *spawn_rate * (0.8 + (time.elapsed_secs() * 0.3).sin() * 0.2); // Organic timing variation
            }
        }
    }
}


// Enhanced turret shooting with biological projectiles
pub fn turret_shooting(
    mut commands: Commands,
    mut turret_query: Query<(&Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        if let Ok(player_transform) = player_query.single() {
            for (turret_transform, mut enemy) in turret_query.iter_mut() {
                let enemy_clone = enemy.clone();

                if let EnemyAI::Turret { rotation, shoot_timer, detection_range } = &mut enemy.ai_type {
                    *shoot_timer -= time.delta_secs();
                    
                    let distance = turret_transform.translation.distance(player_transform.translation);
                    
                    if distance <= *detection_range && *shoot_timer <= 0.0 {
                        let direction = (player_transform.translation.truncate() - turret_transform.translation.truncate()).normalize();
                        
                        // Biological projectile characteristics based on enemy type
                        let (projectile_color, damage, velocity) = match enemy_clone.enemy_type {
                            EnemyType::BiofilmColony => (Color::srgb(0.6, 0.8, 0.3), 25, 350.0), // Toxic green
                            _ => (Color::srgb(0.8, 0.4, 0.4), 20, 400.0), // Standard red
                        };
                        
                        // Spawn multiple projectiles for biofilm colonies (represent toxic cloud)
                        let projectile_count = if matches!(enemy_clone.enemy_type, EnemyType::BiofilmColony) { 3 } else { 1 };
                        
                        for i in 0..projectile_count {
                            let spread_angle = if projectile_count > 1 { 
                                (i as f32 - 1.0) * 0.3 
                            } else { 
                                0.0 
                            };
                            
                            let spread_direction = Vec2::new(
                                direction.x * spread_angle.cos() - direction.y * spread_angle.sin(),
                                direction.x * spread_angle.sin() + direction.y * spread_angle.cos(),
                            );
                            
                            commands.spawn((
                                Sprite {
                                    image: assets.projectile_texture.clone(),
                                    color: projectile_color,
                                    ..default()
                                },
                                Transform::from_translation(turret_transform.translation + Vec3::new(0.0, -15.0, 0.0))
                                    .with_rotation(Quat::from_rotation_z(*rotation)),
                                Projectile {
                                    velocity: spread_direction * velocity,
                                    damage,
                                    friendly: false,
                                    organic_trail: enemy_clone.chemical_signature.releases_toxins,
                                },
                                Collider { radius: 4.0 },
                            ));
                        }
                        
                        *shoot_timer = 1.2 + (time.elapsed_secs() * 0.8).sin() * 0.3; // Organic timing variation
                    }
                }
            }
        }
    }
}

fn execute_chemical_signaling(
    commands: &mut Commands,
    assets: &GameAssets,
    colony: &ColonyCommander,
    member_query: &Query<(&mut Enemy, &ColonyMember, &Transform), Without<ColonyCommander>>,
    player_transform: &Transform,
) {
    for &member_entity in &colony.members {
        if let Ok((_, member, member_transform)) = member_query.get(member_entity) {
            let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
            
            // Enhanced projectiles based on colony role
            let (velocity, damage, color) = match member.role {
                ColonyRole::Queen => (480.0, 35, Color::srgb(1.0, 0.3, 0.8)),
                ColonyRole::Worker => (420.0, 25, Color::srgb(0.8, 0.9, 0.3)),
                ColonyRole::Guardian => (380.0, 30, Color::srgb(0.3, 0.8, 0.9)),
                ColonyRole::Symbiont => (350.0, 20, Color::srgb(0.9, 0.6, 1.0)),
            };
            
            commands.spawn((
                Sprite {
                    image: assets.projectile_texture.clone(),
                    color,
                    ..default()
                },
                Transform::from_translation(member_transform.translation),
                Projectile {
                    velocity: direction * velocity,
                    damage,
                    friendly: false,
                    organic_trail: true,
                },
                Collider { radius: 5.0 },
            ));
        }
    }
}

fn execute_swarm_behavior(
    commands: &mut Commands,
    assets: &GameAssets,
    colony: &ColonyCommander,
    member_query: &Query<(&mut Enemy, &ColonyMember, &Transform), Without<ColonyCommander>>,
    player_transform: &Transform,
    swarm_size: u32,
) {
    let workers: Vec<_> = colony.members.iter()
        .filter_map(|&entity| member_query.get(entity).ok())
        .filter(|(_, member, _, )| matches!(member.role, ColonyRole::Worker | ColonyRole::Queen))
        .take(swarm_size as usize)
        .collect();
    
    for (_, member, member_transform) in workers {
        let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
        
        // Spawn multiple projectiles per swarm member
        for i in 0..3 {
            let spread_angle = (i as f32 - 1.0) * 0.25;
            let spread_direction = Vec2::new(
                direction.x * spread_angle.cos() - direction.y * spread_angle.sin(),
                direction.x * spread_angle.sin() + direction.y * spread_angle.cos(),
            );
            
            commands.spawn((
                Sprite {
                    image: assets.projectile_texture.clone(),
                    color: Color::srgb(0.7, 0.5, 1.0),
                    ..default()
                },
                Transform::from_translation(member_transform.translation),
                Projectile {
                    velocity: spread_direction * 400.0,
                    damage: 18,
                    friendly: false,
                    organic_trail: true,
                },
                Collider { radius: 4.0 },
            ));
        }
    }
}

fn execute_biofilm_formation(
    commands: &mut Commands,
    assets: &GameAssets,
    commander_transform: &Transform,
    member_count: u32,
    rotation_offset: f32,
) {
    let angle_step = std::f32::consts::TAU / member_count as f32;
    
    for i in 0..member_count {
        let angle = angle_step * i as f32 + rotation_offset;
        let direction = Vec2::new(angle.cos(), angle.sin());
        
        commands.spawn((
            Sprite {
                image: assets.projectile_texture.clone(),
                color: Color::srgb(0.6, 0.8, 0.4),
                ..default()
            },
            Transform::from_translation(commander_transform.translation),
            Projectile {
                velocity: direction * 340.0,
                damage: 22,
                friendly: false,
                organic_trail: true,
            },
            Collider { radius: 4.0 },
        ));
    }
}

fn execute_pheromone_trail(
    commands: &mut Commands,
    assets: &GameAssets,
    colony: &ColonyCommander,
    member_query: &Query<(&mut Enemy, &ColonyMember, &Transform), Without<ColonyCommander>>,
    player_transform: &Transform,
) {
    // All members follow pheromone trail to player with prediction
    for &member_entity in &colony.members {
        if let Ok((_, member, member_transform)) = member_query.get(member_entity) {
            // Predict player position
            let distance = member_transform.translation.distance(player_transform.translation);
            let time_to_hit = distance / 450.0;
            let predicted_position = player_transform.translation; // Simplified prediction
            
            let direction = (predicted_position.truncate() - member_transform.translation.truncate()).normalize_or_zero();
            
            // High accuracy pheromone-guided shot
            commands.spawn((
                Sprite {
                    image: assets.projectile_texture.clone(),
                    color: Color::srgb(1.0, 0.7, 0.3),
                    custom_size: Some(Vec2::splat(6.0)),
                    ..default()
                },
                Transform::from_translation(member_transform.translation),
                Projectile {
                    velocity: direction * 520.0,
                    damage: 32,
                    friendly: false,
                    organic_trail: true,
                },
                Collider { radius: 5.0 },
            ));
        }
    }
}

fn execute_biological_maneuvers(
    commands: &mut Commands,
    colony: &ColonyCommander,
    member_query: &mut Query<(&mut Enemy, &ColonyMember, &Transform), Without<ColonyCommander>>,
    delta_time: f32,
) {
    // Give special biological behaviors to different roles
    for &member_entity in &colony.members {
        if let Ok((mut enemy, member, transform)) = member_query.get_mut(member_entity) {
            match member.role {
                ColonyRole::Queen => {
                    // Queens coordinate and move more strategically
                    if let EnemyAI::Formation { formation_timer, .. } = &mut enemy.ai_type {
                        *formation_timer += delta_time;
                        // Queens can break formation to avoid danger
                        if *formation_timer > 8.0 && (*formation_timer % 12.0) < 2.0 {
                            enemy.speed = 280.0; // Faster evasion
                        } else {
                            enemy.speed = 160.0; // Normal coordination speed
                        }
                    }
                }
                
                ColonyRole::Worker => {
                    // Workers maintain aggressive positioning
                    if let EnemyAI::Formation { position_in_formation, .. } = &mut enemy.ai_type {
                        // Adjust position for optimal attack angles
                        let tactical_adjustment = Vec2::new(
                            (colony.chemical_timer * 0.7).sin() * 15.0,
                            (colony.chemical_timer * 0.4).cos() * 8.0,
                        );
                        *position_in_formation += tactical_adjustment * delta_time;
                    }
                }
                
                ColonyRole::Guardian => {
                    // Guardians protect the colony formation
                    if colony.members.len() < 4 {
                        // Become more aggressive when colony is threatened
                        enemy.speed = 220.0;
                        
                        // Release defensive pheromones (could spawn particles here)
                    }
                }
                
                ColonyRole::Symbiont => {
                    // Symbionts provide support and coordination
                    if let EnemyAI::Formation { formation_timer, .. } = &mut enemy.ai_type {
                        // Symbionts help maintain formation cohesion
                        *formation_timer += delta_time * 0.8; // Slower, more deliberate movement
                        enemy.speed = 140.0; // Supportive, not aggressive
                    }
                }
            }
        }
    }
}

// Enhanced formation system with biological colony behavior
pub fn update_formations(
    mut colony_leader_query: Query<(&mut Transform, &mut ColonyLeader)>,
    mut colony_member_query: Query<(&mut Enemy, &Transform), Without<ColonyLeader>>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (mut leader_transform, mut colony) in colony_leader_query.iter_mut() {
        colony.pattern_timer += time.delta_secs();
        
        // Colony leaders respond to fluid currents
        let grid_pos = world_to_grid_pos(leader_transform.translation.truncate(), &fluid_environment);
        let current = sample_current(&fluid_environment, grid_pos);
        
        // Base downward movement with current influence
        leader_transform.translation.y -= 90.0 * time.delta_secs();
        leader_transform.translation += (current * 0.6).extend(0.0) * time.delta_secs();
        
        // Organic colony movement patterns
        let colony_movement = match colony.pattern_type {
            ColonyPattern::BiofilmFormation => {
                // Slow, spreading movement
                Vec2::new(
                    (colony.pattern_timer * 0.4).sin() * 30.0,
                    (colony.pattern_timer * 0.6).cos() * 15.0,
                )
            },
            ColonyPattern::LinearChain => {
                // Undulating chain movement
                Vec2::new(
                    (colony.pattern_timer * 1.2).sin() * 40.0,
                    0.0,
                )
            },
            ColonyPattern::CircularCluster => {
                // Breathing circular motion
                let radius_pulse = 20.0 + (colony.pattern_timer * 1.5).sin() * 10.0;
                Vec2::new(
                    (colony.pattern_timer * 0.8).cos() * radius_pulse,
                    (colony.pattern_timer * 0.8).sin() * radius_pulse * 0.5,
                ) * 0.5
            },
            ColonyPattern::SymbioticPair => {
                // Synchronized pair movement
                Vec2::new(
                    (colony.pattern_timer * 2.0).sin() * 25.0,
                    (colony.pattern_timer * 1.5).cos() * 12.0,
                )
            },
        };
        
        leader_transform.translation += colony_movement.extend(0.0) * time.delta_secs();
        
        // Update member positions based on colony pattern
        for (member_index, member_entity) in colony.members.iter().enumerate() {
            if let Ok((mut member_enemy, _)) = colony_member_query.get_mut(*member_entity) {
                if let EnemyAI::Formation { position_in_formation, leader_offset, .. } = &mut member_enemy.ai_type {
                    let new_pos = colony.pattern_type.get_position(
                        member_index, 
                        colony.members.len(), 
                        colony.pattern_timer
                    );
                    *position_in_formation = new_pos;
                    *leader_offset = colony_movement * 0.5; // Partial influence from colony movement
                }
            }
        }
        
        // Remove dead members from colony
        colony.members.retain(|&member_entity| {
            colony_member_query.get(member_entity).is_ok()
        });
    }
}

// Helper functions for biological systems
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

fn sample_ph_toxicity(chemical_env: &ChemicalEnvironment, position: Vec2) -> f32 {
    let ph = sample_ph(chemical_env, position);
    let optimal_ph = 7.0;
    let ph_deviation = (ph - optimal_ph).abs();
    
    // Return toxicity level (0.0 = safe, 1.0 = highly toxic)
    (ph_deviation / 7.0).clamp(0.0, 1.0)
}

// Enhanced formation coordination with actual chemical signaling
pub fn formation_coordination_system(
    mut commands: Commands,
    mut colony_query: Query<(Entity, &Transform, &mut ColonyCommander)>,
    mut member_query: Query<(&mut Enemy, &ColonyMember, &Transform), Without<ColonyCommander>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>, Without<ColonyCommander>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        if let Ok(player_transform) = player_query.single() {
            for (commander_entity, commander_transform, mut colony) in colony_query.iter_mut() {
                colony.chemical_timer += time.delta_secs();
                
                // Execute coordination patterns
                if colony.coordination_pattern.execute(colony.chemical_timer) {
                    match &colony.coordination_pattern {
                        CoordinationPattern::ChemicalSignaling { .. } => {
                            execute_chemical_signaling(&mut commands, &assets, &colony, &member_query, player_transform);
                        }
                        
                        CoordinationPattern::SwarmBehavior { swarm_size, .. } => {
                            execute_swarm_behavior(&mut commands, &assets, &colony, &member_query, player_transform, *swarm_size);
                        }
                        
                        CoordinationPattern::BiofilmFormation { member_count, rotation_speed } => {
                            let rotation_offset = colony.chemical_timer * rotation_speed;
                            execute_biofilm_formation(&mut commands, &assets, commander_transform, *member_count, rotation_offset);
                        }
                        
                        CoordinationPattern::PheromoneTrail { .. } => {
                            execute_pheromone_trail(&mut commands, &assets, &colony, &member_query, player_transform);
                        }
                    }
                    
                    // Execute biological maneuvers
                    execute_biological_maneuvers(&mut commands, &colony, &mut member_query, time.delta_secs());
                }
            }
        }
    }
}

pub fn pheromone_communication_system(
    mut commands: Commands,
    mut colony_query: Query<(&Transform, &mut ColonyCommander)>,
    mut member_query: Query<(&mut Enemy, &Transform, &ColonyMember)>,
    player_query: Query<&Transform, With<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        for (colony_transform, mut colony) in colony_query.iter_mut() {
            colony.chemical_timer += time.delta_secs();
            
            // Release pheromone signals every 2 seconds
            if colony.chemical_timer % 2.0 < 0.1 {
                // Spawn pheromone particles
                for i in 0..8 {
                    let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                    let offset = Vec2::from_angle(angle) * 40.0;
                    
                    commands.spawn((
                        Sprite {
                            image: assets.particle_texture.clone(),
                            color: Color::srgba(0.9, 0.6, 1.0, 0.4),
                            custom_size: Some(Vec2::splat(6.0)),
                            ..default()
                        },
                        Transform::from_translation(colony_transform.translation + offset.extend(0.0)),
                        PheromoneParticle {
                            signal_type: PheromoneType::Coordination,
                            strength: 1.0,
                            decay_rate: 0.5,
                        },
                        Particle {
                            velocity: offset * 0.5,
                            lifetime: 0.0,
                            max_lifetime: 4.0,
                            size: 6.0,
                            fade_rate: 0.8,
                            bioluminescent: true,
                            drift_pattern: DriftPattern::Floating,
                        },
                    ));
                }
                
                // Coordinate member behaviors
                if let Ok(player_transform) = player_query.single() {
                    for &member_entity in &colony.members {
                        if let Ok((mut enemy, member_transform, member)) = member_query.get_mut(member_entity) {
                            match member.role {
                                ColonyRole::Worker => {
                                    enemy.speed = 200.0; // Boost speed for coordinated attack
                                }
                                ColonyRole::Guardian => {
                                    // Move to intercept player
                                    if let EnemyAI::Formation { position_in_formation, .. } = &mut enemy.ai_type {
                                        let intercept_pos = player_transform.translation.truncate() - colony_transform.translation.truncate();
                                        *position_in_formation = intercept_pos * 0.8;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn cell_division_system(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy, &Health), Without<AlreadyDespawned>>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    for (enemy_entity, transform, mut enemy, health) in enemy_query.iter_mut() {
        if let EnemyAI::CellDivision { division_threshold, division_timer, has_divided } = &mut enemy.ai_type {
            if health.0 as f32 <= *division_threshold && !*has_divided {
                *division_timer -= time.delta_secs();
                
                if *division_timer <= 0.0 {
                    *has_divided = true;
                    
                    // Spawn two offspring at split positions
                    let split_distance = 30.0;
                    let split_positions = [
                        transform.translation + Vec3::new(-split_distance, 0.0, 0.0),
                        transform.translation + Vec3::new(split_distance, 0.0, 0.0),
                    ];
                    
                    for split_pos in split_positions {
                        spawn_events.write(SpawnEnemy {
                            position: split_pos,
                            ai_type: EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) },
                            enemy_type: EnemyType::Offspring,
                        });
                    }
                    
                    // Spawn division particles
                    if let Some(assets) = &assets {
                        for i in 0..12 {
                            let angle = (i as f32 / 12.0) * std::f32::consts::TAU;
                            let offset = Vec2::from_angle(angle) * 20.0;
                            
                            commands.spawn((
                                Sprite {
                                    image: assets.particle_texture.clone(),
                                    color: Color::srgb(0.8, 0.9, 0.6),
                                    custom_size: Some(Vec2::splat(3.0)),
                                    ..default()
                                },
                                Transform::from_translation(transform.translation + offset.extend(0.0)),
                                Particle {
                                    velocity: offset * 2.0,
                                    lifetime: 0.0,
                                    max_lifetime: 1.0,
                                    size: 3.0,
                                    fade_rate: 1.0,
                                    bioluminescent: true,
                                    drift_pattern: DriftPattern::Floating,
                                },
                            ));
                        }
                    }
                    
                    // Original cell dies after division
                    commands.entity(enemy_entity)
                        .insert(AlreadyDespawned)
                        .despawn();
                }
            }
        }
    }
}

pub fn symbiotic_pair_system(
    mut commands: Commands,
    pair_query: Query<(Entity, &Transform, &Enemy, &Health), Without<AlreadyDespawned>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
) {
    // Collect all symbiotic pair data
    let pair_data: Vec<(Entity, Vec3, Option<Entity>)> = pair_query.iter()
        .filter_map(|(entity, transform, enemy, _)| {
            if let EnemyAI::SymbioticPair { partner_entity, .. } = &enemy.ai_type {
                Some((entity, transform.translation, *partner_entity))
            } else {
                None
            }
        })
        .collect();
    
    // Check for broken pairs
    for (entity, position, partner_entity) in pair_data {
        if let Some(partner) = partner_entity {
            if pair_query.get(partner).is_err() {
                // Partner died - this one dies too
                explosion_events.write(SpawnExplosion {
                    position,
                    intensity: 1.2,
                    enemy_type: None,
                });
                commands.entity(entity)
                    .insert(AlreadyDespawned)
                    .despawn();
            }
        }
    }
}

pub fn link_symbiotic_pairs(
    mut pair_query: Query<(Entity, &Transform, &mut Enemy)>,
) {
    // Collect unlinked pairs
    let unlinked: Vec<(Entity, Vec3)> = pair_query.iter()
        .filter_map(|(entity, transform, enemy)| {
            if let EnemyAI::SymbioticPair { partner_entity: None, .. } = &enemy.ai_type {
                Some((entity, transform.translation))
            } else {
                None
            }
        })
        .collect();
    
    // Link pairs in groups of 2
    for chunk in unlinked.chunks(2) {
        if chunk.len() == 2 {
            let (entity1, _) = chunk[0];
            let (entity2, _) = chunk[1];
            
            // Set partners using get_mut to avoid conflicts
            if let Ok((_, _, mut enemy1)) = pair_query.get_mut(entity1) {
                if let EnemyAI::SymbioticPair { partner_entity, .. } = &mut enemy1.ai_type {
                    *partner_entity = Some(entity2);
                }
            }
            if let Ok((_, _, mut enemy2)) = pair_query.get_mut(entity2) {
                if let EnemyAI::SymbioticPair { partner_entity, .. } = &mut enemy2.ai_type {
                    *partner_entity = Some(entity1);
                }
            }
        }
    }
}

pub fn procedural_colony_spawning(
    mut commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    ecosystem: Res<EcosystemState>,
    time: Res<Time>,
    mut colony_timer: Local<f32>,
) {
    *colony_timer += time.delta_secs();
    
    // Spawn colonies every 25 seconds after wave 60
    if enemy_spawner.wave_timer > 60.0 && *colony_timer >= 25.0 {
        *colony_timer = 0.0;
        
        let base_x = (time.elapsed_secs() * 40.0).sin() * 200.0;
        let pattern_type = match (time.elapsed_secs() as u32 / 25) % 4 {
            0 => spawn_biofilm_colony(&mut commands, &mut spawn_events, base_x),
            1 => spawn_linear_chain(&mut commands, &mut spawn_events, base_x),
            2 => spawn_circular_cluster(&mut commands, &mut spawn_events, base_x),
            _ => spawn_hunting_pack(&mut commands, &mut spawn_events, base_x),
        };
        
        enemy_spawner.enemies_spawned += pattern_type;
    }
}

fn spawn_biofilm_colony(
    commands: &mut Commands,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    base_x: f32,
) -> u32 {
    let colony_id = (base_x * 1000.0) as u32;
    
    // Central biofilm node
    spawn_events.write(SpawnEnemy {
        position: Vec3::new(base_x, 420.0, 0.0),
        ai_type: EnemyAI::Turret { rotation: 0.0, shoot_timer: 0.0, detection_range: 250.0 },
        enemy_type: EnemyType::BiofilmColony,
    });
    
    // Spawn colony leader
    let leader_entity = commands.spawn((
        Transform::from_xyz(base_x, 420.0, 0.0),
        ColonyLeader {
            colony_id,
            members: Vec::new(),
            pattern_timer: 0.0,
            pattern_type: ColonyPattern::BiofilmFormation,
            chemical_communication: true,
        },
        ColonyCommander {
            colony_id,
            members: Vec::new(),
            coordination_pattern: CoordinationPattern::ChemicalSignaling { interval: 2.0 },
            chemical_timer: 0.0,
        },
    )).id();
    
    // Surrounding defensive cells in organic cluster
    for layer in 0..3 {
        let cells_in_layer = 3 + layer;
        for i in 0..cells_in_layer {
            let angle = (i as f32 / cells_in_layer as f32) * std::f32::consts::TAU;
            let radius = 40.0 + layer as f32 * 25.0;
            let pos = Vec2::from_angle(angle) * radius;
            
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(base_x + pos.x, 420.0 + pos.y, 0.0),
                ai_type: EnemyAI::Formation {
                    formation_id: colony_id,
                    position_in_formation: pos,
                    leader_offset: Vec2::ZERO,
                    formation_timer: 0.0,
                },
                enemy_type: EnemyType::SwarmCell,
            });
        }
    }
    
    12 // Total enemies spawned
}

fn spawn_linear_chain(
    commands: &mut Commands,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    base_x: f32,
) -> u32 {
    let colony_id = (base_x * 1001.0) as u32;
    
    // Chain of 6 cells with undulating movement
    for i in 0..6 {
        let y_offset = i as f32 * 35.0;
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(base_x, 420.0 - y_offset, 0.0),
            ai_type: EnemyAI::Formation {
                formation_id: colony_id,
                position_in_formation: Vec2::new(0.0, -(i as f32 * 35.0)),
                leader_offset: Vec2::ZERO,
                formation_timer: i as f32 * 0.2, // Staggered timing
            },
            enemy_type: EnemyType::AggressiveBacteria,
        });
    }
    
    6
}

fn spawn_circular_cluster(
    commands: &mut Commands,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    base_x: f32,
) -> u32 {
    let colony_id = (base_x * 1002.0) as u32;
    
    // Circular formation with synchronized movement
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
        let radius = 60.0;
        let pos = Vec2::from_angle(angle) * radius;
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(base_x + pos.x, 420.0 + pos.y, 0.0),
            ai_type: EnemyAI::Formation {
                formation_id: colony_id,
                position_in_formation: pos,
                leader_offset: Vec2::ZERO,
                formation_timer: 0.0,
            },
            enemy_type: EnemyType::SwarmCell,
        });
    }
    
    8
}

fn spawn_hunting_pack(
    commands: &mut Commands,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    base_x: f32,
) -> u32 {
    // Pack of chemotaxis hunters
    for i in 0..4 {
        let offset = Vec2::new((i as f32 - 1.5) * 40.0, (i % 2) as f32 * 30.0);
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(base_x + offset.x, 420.0 + offset.y, 0.0),
            ai_type: EnemyAI::Chemotaxis {
                target_chemical: ChemicalType::PlayerPheromones,
                sensitivity: 1.8,
                current_direction: Vec2::new(0.0, -1.0),
            },
            enemy_type: EnemyType::ParasiticProtozoa,
        });
    }
    
    4
}

// Enhanced Enemy AI

// Enhanced predator-prey system
pub fn predator_prey_system(
    mut predator_query: Query<(&mut Transform, &mut Enemy, &PredatorPreyBehavior), Without<AlreadyDespawned>>,
    prey_query: Query<(Entity, &Transform, &Enemy), (With<Enemy>, Without<PredatorPreyBehavior>, Without<AlreadyDespawned>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut predator_transform, mut predator_enemy, behavior) in predator_query.iter_mut() {
            let mut hunting_target: Option<Vec3> = None;
            let mut fleeing_from: Option<Vec3> = None;
            
            // Look for prey and predators
            for (prey_entity, prey_transform, prey_enemy) in prey_query.iter() {
                let distance = predator_transform.translation.distance(prey_transform.translation);
                
                // Check if this enemy is prey for us
                if behavior.prey_types.contains(&prey_enemy.enemy_type) && distance < behavior.hunt_range {
                    if hunting_target.is_none() || distance < predator_transform.translation.distance(hunting_target.unwrap()) {
                        hunting_target = Some(prey_transform.translation);
                    }
                }
                
                // Check if this enemy is a predator to us
                if behavior.predator_types.contains(&prey_enemy.enemy_type) && distance < behavior.flee_range {
                    fleeing_from = Some(prey_transform.translation);
                    break; // Flee immediately
                }
            }
            
            // Execute behavior
            if let Some(flee_pos) = fleeing_from {
                // FLEE BEHAVIOR - Move away from predators
                let flee_direction = (predator_transform.translation - flee_pos).normalize();
                let panic_speed = predator_enemy.speed * (1.0 + behavior.fear_intensity);
                predator_transform.translation += flee_direction * panic_speed * time.delta_secs();
                
                // Panic rotation
                predator_transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 8.0);
                
            } else if let Some(hunt_pos) = hunting_target {
                // HUNT BEHAVIOR - Chase prey
                let hunt_direction = (hunt_pos - predator_transform.translation).normalize();
                let hunt_speed = predator_enemy.speed * behavior.hunting_speed_bonus;
                predator_transform.translation += hunt_direction * hunt_speed * time.delta_secs();
                
                // Face target
                let angle = hunt_direction.y.atan2(hunt_direction.x) - std::f32::consts::FRAC_PI_2;
                predator_transform.rotation = Quat::from_rotation_z(angle);
                
            } else {
                // NEUTRAL BEHAVIOR - Hunt player if no specific prey
                let player_distance = predator_transform.translation.distance(player_transform.translation);
                if player_distance < behavior.hunt_range * 1.5 {
                    let player_direction = (player_transform.translation - predator_transform.translation).normalize();
                    predator_transform.translation += player_direction * predator_enemy.speed * time.delta_secs();
                }
            }
        }
    }
}

// Chemical trail system
pub fn chemical_trail_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut trail_query: Query<(Entity, &mut ChemicalTrail, &Transform), Without<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut player_trail_timer: Local<f32>,
) {
    // Player creates pheromone trail
    if let Ok(player_transform) = player_query.single() {
        *player_trail_timer += time.delta_secs();
        
        if *player_trail_timer >= 0.2 {
            *player_trail_timer = 0.0;
            
            if let Some(assets) = &assets {
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgba(0.8, 0.4, 1.0, 0.3),
                        custom_size: Some(Vec2::splat(6.0)),
                        ..default()
                    },
                    Transform::from_translation(player_transform.translation - Vec3::new(0.0, 20.0, 0.1)),
                    ChemicalTrail {
                        trail_type: ChemicalTrailType::PlayerPheromone,
                        strength: 1.0,
                        decay_rate: 0.4,
                        creation_timer: time.elapsed_secs(),
                    },
                    Particle {
                        velocity: Vec2::ZERO,
                        lifetime: 0.0,
                        max_lifetime: 5.0,
                        size: 6.0,
                        fade_rate: 0.6,
                        bioluminescent: true,
                        drift_pattern: DriftPattern::Floating,
                    },
                ));
            }
        }
    }
    
    // Update existing trails
    for (trail_entity, mut trail, trail_transform) in trail_query.iter_mut() {
        trail.strength -= trail.decay_rate * time.delta_secs();
        
        if trail.strength <= 0.0 {
            commands.entity(trail_entity)
                .insert(AlreadyDespawned)
                .despawn();
        }
    }
}

// Trail following behavior
pub fn chemical_trail_following(
    mut enemy_query: Query<(&mut Transform, &mut Enemy), (With<Enemy>, Without<ChemicalTrail>)>,
    trail_query: Query<(&Transform, &ChemicalTrail), (With<ChemicalTrail>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut enemy_transform, mut enemy) in enemy_query.iter_mut() {
        // Only certain AI types follow trails
        match &enemy.ai_type {
            EnemyAI::Chemotaxis { .. } | EnemyAI::Linear { .. } => {
                let mut strongest_trail: Option<(Vec3, f32)> = None;
                
                // Find strongest nearby trail
                for (trail_transform, trail) in trail_query.iter() {
                    let distance = enemy_transform.translation.distance(trail_transform.translation);
                    
                    if distance < 80.0 {
                        let influence = trail.strength / (distance + 1.0);
                        
                        if strongest_trail.is_none() || influence > strongest_trail.unwrap().1 {
                            strongest_trail = Some((trail_transform.translation, influence));
                        }
                    }
                }
                
                // Follow strongest trail
                if let Some((trail_pos, influence)) = strongest_trail {
                    if influence > 0.1 {
                        
                        let trail_direction = (trail_pos - enemy_transform.translation).normalize();
                        let follow_strength = (influence * 60.0).min(enemy.speed * 0.7);
                        enemy_transform.translation += trail_direction * follow_strength * time.delta_secs();
                        
                        // downgrade from Vec3 to Vec2 with .truncate() opposite of .extend(0.0)
                        let trail_direction = trail_direction.truncate();
                        // Update AI direction if applicable
                        match &mut enemy.ai_type {
                            EnemyAI::Chemotaxis { current_direction, .. } => {
                                *current_direction = (*current_direction + trail_direction * 0.3).normalize();
                            }
                            EnemyAI::Linear { direction } => {
                                *direction = (*direction + trail_direction * 0.1).normalize();
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// Ecosystem balance system
pub fn ecosystem_balance_system(
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut ecosystem: ResMut<EcosystemState>,
    enemy_query: Query<(&Enemy, &EcosystemRole)>,
    player_query: Query<(&Player, &EvolutionSystem)>,
    time: Res<Time>,
) {
    // Count ecosystem roles
    let mut role_counts = HashMap::new();
    let mut total_balance = 0.0;
    
    for (enemy, role) in enemy_query.iter() {
        let count = role_counts.entry(role.role.clone()).or_insert(0u32);
        *count += 1;
        total_balance += role.balance_factor;
    }
    
    // Calculate ecosystem pressure
    let apex_count = *role_counts.get(&EcosystemRoleType::Apex).unwrap_or(&0);
    let primary_count = *role_counts.get(&EcosystemRoleType::Primary).unwrap_or(&0);
    let secondary_count = *role_counts.get(&EcosystemRoleType::Secondary).unwrap_or(&0);
    
    // Adjust spawn rates based on balance
    let balance_factor = (total_balance / (enemy_query.iter().count() as f32 + 1.0)).clamp(0.3, 3.0);
    
    // Too many apex predators slow spawning
    if apex_count > 3 {
        enemy_spawner.spawn_timer *= 1.5;
    }
    
    // Too few primary threats speed up spawning
    if primary_count < 2 && enemy_spawner.wave_timer > 10.0 {
        enemy_spawner.spawn_timer *= 0.7;
    }
    
    // Update ecosystem health
    let ideal_apex_ratio = 0.1;
    let ideal_primary_ratio = 0.4;
    let ideal_secondary_ratio = 0.5;
    
    let total_enemies = enemy_query.iter().count() as f32 + 1.0;
    let apex_ratio = apex_count as f32 / total_enemies;
    let primary_ratio = primary_count as f32 / total_enemies;
    let secondary_ratio = secondary_count as f32 / total_enemies;
    
    let balance_score : f32 = 1.0 - (
        (apex_ratio - ideal_apex_ratio).abs() +
        (primary_ratio - ideal_primary_ratio).abs() +
        (secondary_ratio - ideal_secondary_ratio).abs()
    ) / 3.0;
    
    ecosystem.health = balance_score.clamp(0.0, 1.0);
    ecosystem.population_balance.pathogenic_threats = primary_count + apex_count;
    ecosystem.population_balance.beneficial_microbes = secondary_count;
}

// Adaptive difficulty system
pub fn adaptive_difficulty_system(
    mut commands: Commands,
    mut adaptive_query: Query<(Entity, &mut AdaptiveDifficulty, &mut Enemy)>,
    player_query: Query<(&Player, &EvolutionSystem, &Health)>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    ecosystem: Res<EcosystemState>,
    time: Res<Time>,
) {
    if let Ok((player, evolution, health)) = player_query.single() {
        // Calculate player strength
        let evolution_level = match evolution.primary_evolution {
            EvolutionType::CytoplasmicSpray { .. } => 1.0,
            EvolutionType::PseudopodNetwork { .. } => 2.0,
            EvolutionType::BioluminescentBeam { .. } => 3.0,
            EvolutionType::SymbioticHunters { .. } => 4.0,
            EvolutionType::EnzymeBurst { .. } => 3.5,
            EvolutionType::ToxinCloud { .. } => 4.5,
            EvolutionType::ElectricDischarge { .. } => 5.0,
        };
        
        let player_strength = evolution_level * (health.0 as f32 / 100.0) * (player.lives as f32 * 0.3 + 0.7);
        
        // Adapt enemy difficulty
        for (enemy_entity, mut adaptive, mut enemy) in adaptive_query.iter_mut() {
            let target_threat = player_strength * 0.8; // Enemies should be slightly weaker
            let threat_difference = target_threat - adaptive.threat_level;
            
            // Gradual adaptation
            adaptive.threat_level += threat_difference * adaptive.adaptation_rate * time.delta_secs();
            
            // Apply adaptations
            if adaptive.threat_level > 1.5 {
                enemy.speed *= 1.0 + (adaptive.threat_level - 1.5) * 0.1;
                enemy.health = (enemy.health as f32 * (1.0 + (adaptive.threat_level - 1.5) * 0.15)) as i32;
            }
            
            // Spawn additional threats if player is too strong
            if player_strength > 4.0 && (time.elapsed_secs() % 15.0) < 0.1 {
                let threat_type = if player_strength > 6.0 {
                    EnemyType::InfectedMacrophage
                } else {
                    EnemyType::ParasiticProtozoa
                };
                
                spawn_events.write(SpawnEnemy {
                    position: Vec3::new(
                        (time.elapsed_secs() * 100.0).sin() * 300.0,
                        450.0,
                        0.0
                    ),
                    ai_type: EnemyAI::Chemotaxis {
                        target_chemical: ChemicalType::PlayerPheromones,
                        sensitivity: 2.0,
                        current_direction: Vec2::new(0.0, -1.0),
                    },
                    enemy_type: threat_type,
                });
            }
        }
    }
}

// Initialize enhanced AI components
pub fn setup_enhanced_enemy_ai() -> impl Bundle {
    (
        PredatorPreyBehavior {
            predator_types: vec![EnemyType::InfectedMacrophage],
            prey_types: vec![EnemyType::ViralParticle, EnemyType::Offspring],
            hunt_range: 150.0,
            flee_range: 200.0,
            hunting_speed_bonus: 1.3,
            fear_intensity: 0.8,
        },
        EcosystemRole {
            role: EcosystemRoleType::Primary,
            influence_radius: 100.0,
            balance_factor: 1.0,
        },
        AdaptiveDifficulty {
            threat_level: 1.0,
            adaptation_rate: 0.5,
            player_evolution_response: 1.2,
        },
    )
}

// Predator-prey relationships configuration
impl EnemyType {
    pub fn get_predator_prey_behavior(&self) -> Option<PredatorPreyBehavior> {
        match self {
            EnemyType::InfectedMacrophage => Some(PredatorPreyBehavior {
                predator_types: vec![], // Apex predator
                prey_types: vec![EnemyType::ViralParticle, EnemyType::AggressiveBacteria, EnemyType::Offspring],
                hunt_range: 180.0,
                flee_range: 50.0, // Rarely flees
                hunting_speed_bonus: 1.4,
                fear_intensity: 0.2,
            }),
            EnemyType::ParasiticProtozoa => Some(PredatorPreyBehavior {
                predator_types: vec![EnemyType::InfectedMacrophage],
                prey_types: vec![EnemyType::ViralParticle, EnemyType::SwarmCell],
                hunt_range: 120.0,
                flee_range: 150.0,
                hunting_speed_bonus: 1.2,
                fear_intensity: 0.6,
            }),
            EnemyType::AggressiveBacteria => Some(PredatorPreyBehavior {
                predator_types: vec![EnemyType::InfectedMacrophage, EnemyType::ParasiticProtozoa],
                prey_types: vec![EnemyType::ViralParticle],
                hunt_range: 100.0,
                flee_range: 120.0,
                hunting_speed_bonus: 1.1,
                fear_intensity: 0.7,
            }),
            EnemyType::SwarmCell => Some(PredatorPreyBehavior {
                predator_types: vec![EnemyType::ParasiticProtozoa, EnemyType::InfectedMacrophage],
                prey_types: vec![EnemyType::Offspring],
                hunt_range: 80.0,
                flee_range: 100.0,
                hunting_speed_bonus: 1.0,
                fear_intensity: 0.5,
            }),
            _ => None,
        }
    }
    
    pub fn get_ecosystem_role(&self) -> EcosystemRole {
        match self {
            EnemyType::InfectedMacrophage => EcosystemRole {
                role: EcosystemRoleType::Apex,
                influence_radius: 200.0,
                balance_factor: 3.0,
            },
            EnemyType::ParasiticProtozoa | EnemyType::BiofilmColony => EcosystemRole {
                role: EcosystemRoleType::Primary,
                influence_radius: 120.0,
                balance_factor: 2.0,
            },
            EnemyType::AggressiveBacteria | EnemyType::SwarmCell => EcosystemRole {
                role: EcosystemRoleType::Secondary,
                influence_radius: 80.0,
                balance_factor: 1.0,
            },
            EnemyType::Offspring => EcosystemRole {
                role: EcosystemRoleType::Decomposer,
                influence_radius: 40.0,
                balance_factor: 0.3,
            },
            _ => EcosystemRole {
                role: EcosystemRoleType::Secondary,
                influence_radius: 60.0,
                balance_factor: 1.0,
            },
        }
    }
}

