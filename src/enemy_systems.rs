use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;

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
            
            EnemyAI::Formation { formation_id, position_in_formation, leader_offset, formation_timer } => {
                *formation_timer += time.delta_secs();
                
                // Find colony leader position
                let leader_pos = colony_leader_query.iter()
                    .find(|leader_transform| {
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
                
                // Blend base direction with current flow
                let flow_influence = current * *flow_sensitivity * time.delta_secs();
                *base_direction = (*base_direction + flow_influence).normalize_or_zero();
                
                // Apply movement with strong current response
                let movement = *base_direction * enemy.speed + current * 0.8;
                transform.translation += movement.extend(0.0) * time.delta_secs();
                
                // Rotate to face movement direction
                let angle = base_direction.y.atan2(base_direction.x) - std::f32::consts::FRAC_PI_2;
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
    for (entity, transform, mut enemy) in spawner_query.iter_mut() {

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

// FIXED: Enhanced projectile collision system - properly resolved borrowing issues
pub fn enhanced_projectile_collisions(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &Transform, &Collider, &Projectile, Option<&ExplosiveProjectile>, Option<&mut ArmorPiercing>)>,
    mut enemy_query: Query<(Entity, &Transform, &Collider, &mut Health), (With<Enemy>, Without<Projectile>)>,
    laser_query: Query<(&Transform, &LaserBeam, &Collider)>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    time: Res<Time>,
    mut last_laser_damage: Local<f32>,
) {
    *last_laser_damage += time.delta_secs();
    
    // Laser collision with enemies
    if *last_laser_damage >= 0.1 {
        for (laser_transform, laser_beam, laser_collider) in laser_query.iter() {
            for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health) in enemy_query.iter_mut() {
                let distance = laser_transform.translation.distance(enemy_transform.translation);
                if distance < laser_collider.radius + enemy_collider.radius {
                    let damage = (laser_beam.damage_per_second as f32 * 0.1) as i32;
                    enemy_health.0 -= damage;
                    
                    if enemy_health.0 <= 0 {
                        explosion_events.write(SpawnExplosion {
                            position: enemy_transform.translation,
                            intensity: 1.2,
                            enemy_type: None,
                        });
                        commands.entity(enemy_entity).despawn();
                    }
                }
            }
        }
        *last_laser_damage = 0.0;
    }
    
    // FIXED: Collect all enemy positions first to avoid borrowing conflicts
    let enemy_positions: Vec<(Entity, Vec3, f32)> = enemy_query.iter()
        .map(|(entity, transform, collider, _)| (entity, transform.translation, collider.radius))
        .collect();
    
    // Regular projectile collisions
    for (proj_entity, proj_transform, proj_collider, projectile, explosive, mut armor_piercing) in projectile_query.iter_mut() {
        if !projectile.friendly { continue; }
        
        let mut hit_target = None;
        let mut blast_targets = Vec::new();
        
        // Find collision target using collected positions
        for &(enemy_entity, enemy_pos, enemy_radius) in &enemy_positions {
            let distance = proj_transform.translation.distance(enemy_pos);
            if distance < proj_collider.radius + enemy_radius {
                hit_target = Some(enemy_entity);
                
                // If explosive, find blast targets
                if let Some(explosive_proj) = explosive {
                    for &(nearby_entity, nearby_pos, _) in &enemy_positions {
                        if nearby_entity != enemy_entity {
                            let blast_distance = proj_transform.translation.distance(nearby_pos);
                            if blast_distance <= explosive_proj.blast_radius {
                                blast_targets.push((nearby_entity, explosive_proj.blast_damage));
                            }
                        }
                    }
                }
                break;
            }
        }
        
        // Apply damage if we hit something
        if let Some(target_entity) = hit_target {
            // Apply direct damage
            if let Ok((_, target_transform, _, mut target_health)) = enemy_query.get_mut(target_entity) {
                target_health.0 -= projectile.damage;
                
                if target_health.0 <= 0 {
                    explosion_events.write(SpawnExplosion {
                        position: target_transform.translation,
                        intensity: 1.0,
                        enemy_type: None,
                    });
                    commands.entity(target_entity).despawn();
                }
            }
            
            // Handle explosive damage
            if let Some(explosive_proj) = explosive {
                explosion_events.write(SpawnExplosion {
                    position: proj_transform.translation,
                    intensity: 1.5,
                    enemy_type: None,
                });
                
                for (blast_entity, blast_damage) in blast_targets {
                    if let Ok((_, blast_transform, _, mut blast_health)) = enemy_query.get_mut(blast_entity) {
                        blast_health.0 -= blast_damage;
                        if blast_health.0 <= 0 {
                            explosion_events.write(SpawnExplosion {
                                position: blast_transform.translation,
                                intensity: 0.8,
                                enemy_type: None,
                            });
                            commands.entity(blast_entity).despawn();
                        }
                    }
                }
            }
            
            // Handle armor piercing
            if let Some(ref mut piercing) = armor_piercing {
                piercing.pierce_count += 1;
                if piercing.pierce_count >= piercing.max_pierce {
                    commands.entity(proj_entity).despawn();
                }
            } else {
                commands.entity(proj_entity).despawn();
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
    mut member_query: Query<(Entity, &mut Enemy, &ColonyMember, &Transform), Without<ColonyCommander>>,
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
                        CoordinationPattern::ChemicalSignaling { interval } => {
                            // All members shoot in coordinated burst
                            for &member_entity in &colony.members {
                                if let Ok((_, _, member, member_transform)) = member_query.get(member_entity) {
                                    let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
                                    
                                    commands.spawn((
                                        Sprite {
                                            image: assets.projectile_texture.clone(),
                                            color: Color::srgb(0.8, 0.9, 0.3),
                                            ..default()
                                        },
                                        Transform::from_translation(member_transform.translation),
                                        Projectile {
                                            velocity: direction * 420.0,
                                            damage: 25,
                                            friendly: false,
                                            organic_trail: true,
                                        },
                                        Collider { radius: 5.0 },
                                    ));
                                }
                            }
                        }
                        
                        CoordinationPattern::SwarmBehavior { swarm_size, .. } => {
                            // Rapid-fire barrage from swarm members
                            let workers: Vec<_> = colony.members.iter().take(*swarm_size as usize).collect();
                            for &member_entity in workers {
                                if let Ok((_, _, _, member_transform)) = member_query.get(member_entity) {
                                    let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
                                    
                                    // Spawn 3 projectiles per swarm member
                                    for i in 0..3 {
                                        let spread = (i as f32 - 1.0) * 0.3;
                                        let spread_dir = Vec2::new(
                                            direction.x * spread.cos() - direction.y * spread.sin(),
                                            direction.x * spread.sin() + direction.y * spread.cos(),
                                        );
                                        
                                        commands.spawn((
                                            Sprite {
                                                image: assets.projectile_texture.clone(),
                                                color: Color::srgb(0.7, 0.5, 1.0),
                                                ..default()
                                            },
                                            Transform::from_translation(member_transform.translation),
                                            Projectile {
                                                velocity: spread_dir * 400.0,
                                                damage: 18,
                                                friendly: false,
                                                organic_trail: true,
                                            },
                                            Collider { radius: 4.0 },
                                        ));
                                    }
                                }
                            }
                        }
                        
                        _ => {} // Other patterns already implemented
                    }
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