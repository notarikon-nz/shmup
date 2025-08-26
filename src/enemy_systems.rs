use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;
use crate::physics::{world_to_grid_pos, sample_current, sample_ph, sample_oxygen};
use std::collections::HashMap;

// ===== CONSTANTS =====
const UNDULATION_AMPLITUDE: f32 = 8.0;
const CURRENT_INFLUENCE_WEAK: f32 = 0.2;
const CURRENT_INFLUENCE_STRONG: f32 = 0.8;
const CHEMICAL_AVOIDANCE_THRESHOLD: f32 = 1.2;
const CHEMICAL_AVOIDANCE_STRENGTH: f32 = 40.0;
const DETECTION_RANGE_DEFAULT: f32 = 250.0;
const SPAWN_DISTANCE_BASE: f32 = 25.0;
const FORMATION_SPEED_NORMAL: f32 = 90.0;

// ===== HELPER FUNCTIONS =====
fn apply_organic_undulation(transform: &mut Transform, time: f32, amplitude: f32) {
    let undulation = Vec2::new(
        (time * 1.8 + transform.translation.y * 0.008).sin() * amplitude,
        0.0,
    );
    transform.translation += undulation.extend(0.0);
}

fn apply_current_influence(transform: &mut Transform, fluid_env: &FluidEnvironment, influence: f32, dt: f32) {
    let grid_pos = world_to_grid_pos(transform.translation.truncate(), fluid_env);
    let current = sample_current(fluid_env, grid_pos);
    transform.translation += (current * influence).extend(0.0) * dt;
}

fn get_chemical_avoidance(pos: Vec2, chemical_env: &ChemicalEnvironment, enemy: &Enemy) -> Vec2 {
    let local_ph = sample_ph(chemical_env, pos);
    let ph_diff = (local_ph - enemy.chemical_signature.ph_preference).abs();
    
    if ph_diff > CHEMICAL_AVOIDANCE_THRESHOLD && enemy.chemical_signature.responds_to_pheromones {
        let direction = if local_ph > enemy.chemical_signature.ph_preference {
            Vec2::new(-0.5, 0.2)
        } else {
            Vec2::new(0.5, 0.2)
        };
        direction * (ph_diff - CHEMICAL_AVOIDANCE_THRESHOLD) * CHEMICAL_AVOIDANCE_STRENGTH
    } else {
        Vec2::ZERO
    }
}

// ===== MAIN SYSTEMS =====

pub fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    colony_leader_query: Query<&Transform, (With<ColonyLeader>, Without<Enemy>, Without<Player>)>,
    fluid_environment: Res<FluidEnvironment>,
    chemical_environment: Res<ChemicalEnvironment>,
    time: Res<Time>,
) {
    let player_pos = player_query.single().ok().map(|t| t.translation.truncate());
    let dt = time.delta_secs();
    
    for (mut transform, mut enemy) in enemy_query.iter_mut() {
        
        let enemy_clone = enemy.clone();

        match &mut enemy.ai_type {
            EnemyAI::Static => {}
            
            EnemyAI::Linear { direction } => {
                apply_organic_undulation(&mut transform, time.elapsed_secs(), UNDULATION_AMPLITUDE);
                let movement = direction.extend(0.0);
                transform.translation += movement * enemy.speed * dt;
                apply_current_influence(&mut transform, &fluid_environment, 0.3, dt);
            }
            
            EnemyAI::Sine { amplitude, frequency, phase } => {
                *phase += dt * *frequency;
                transform.translation.y -= enemy_clone.speed * dt;
                
                let organic_var = (time.elapsed_secs() * 0.4).sin() * 0.15;
                let actual_amp = *amplitude * (1.0 + organic_var);
                transform.translation.x += actual_amp * phase.sin() * dt;
                apply_current_influence(&mut transform, &fluid_environment, CURRENT_INFLUENCE_WEAK, dt);
            }
            
            EnemyAI::MiniBoss { timer, .. } => {
                *timer += dt;
                transform.translation.y -= enemy_clone.speed * 0.6 * dt;
                let pattern = Vec2::new((*timer * 0.8).sin() * 100.0, (*timer * 0.5).cos() * 30.0);
                transform.translation += pattern.extend(0.0) * dt;
            }
            
            EnemyAI::Kamikaze { target_pos, dive_speed, acquired_target } => {
                if let Some(player_pos) = player_pos {
                    if !*acquired_target {
                        *target_pos = player_pos;
                        *acquired_target = true;
                    }
                    let direction = (*target_pos - transform.translation.truncate()).normalize_or_zero();
                    transform.translation += direction.extend(0.0) * *dive_speed * dt;
                    
                    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    let wobble = (time.elapsed_secs() * 8.0).sin() * 0.1;
                    transform.rotation = Quat::from_rotation_z(angle + wobble);
                } else {
                    apply_current_influence(&mut transform, &fluid_environment, 0.5, dt);
                    transform.translation.y -= 50.0 * dt;
                }
            }
            
            EnemyAI::Turret { rotation, detection_range, .. } => {
                let sway = sample_current(&fluid_environment, world_to_grid_pos(transform.translation.truncate(), &fluid_environment)).x * 0.001;
                if let Some(player_pos) = player_pos {
                    let direction = player_pos - transform.translation.truncate();
                    let target_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    let angle_diff = (target_angle - *rotation + std::f32::consts::PI) % std::f32::consts::TAU - std::f32::consts::PI;
                    *rotation += angle_diff.clamp(-1.8 * dt, 1.8 * dt);
                    transform.rotation = Quat::from_rotation_z(*rotation + sway);
                }
            }
            
            EnemyAI::Formation { position_in_formation, leader_offset, .. } => {
                let leader_pos = colony_leader_query.iter().next()
                    .map(|t| t.translation.truncate())
                    .unwrap_or_else(|| Vec2::new(0.0, 400.0));
                
                let target_pos = leader_pos + *leader_offset + *position_in_formation;
                let direction = (target_pos - transform.translation.truncate()).normalize_or_zero();
                apply_current_influence(&mut transform, &fluid_environment, 0.4, dt);
                transform.translation += direction.extend(0.0) * enemy.speed * dt;
            }
            
            EnemyAI::Spawner { .. } => {
                apply_current_influence(&mut transform, &fluid_environment, CURRENT_INFLUENCE_STRONG, dt);
                transform.translation.y -= enemy.speed * 0.7 * dt;
            }
            
            EnemyAI::Chemotaxis { target_chemical, sensitivity, current_direction } => {
                if let Some(player_pos) = player_pos {
                    let distance = transform.translation.distance(player_pos.extend(0.0));
                    if distance < 350.0 {
                        let dir_to_player = (player_pos - transform.translation.truncate()).normalize_or_zero();
                        let chemical_strength = match target_chemical {
                            ChemicalType::PlayerPheromones => 1.0 / (distance * 0.01 + 1.0),
                            ChemicalType::OxygenSeeker => sample_oxygen(&chemical_environment, transform.translation.truncate()),
                            _ => 0.5,
                        };
                        
                        let influence = chemical_strength * *sensitivity;
                        *current_direction = current_direction.lerp(dir_to_player, influence * dt);
                        
                        let random_influence = Vec2::new(
                            (time.elapsed_secs() * 3.2 + transform.translation.x * 0.01).sin() * 0.2,
                            (time.elapsed_secs() * 2.7 + transform.translation.y * 0.01).cos() * 0.2,
                        );
                        *current_direction = (*current_direction + random_influence).normalize_or_zero();
                        transform.translation += current_direction.extend(0.0) * enemy.speed * dt;
                    } else {
                        let random_turn = (time.elapsed_secs() * 2.5 + transform.translation.x * 0.005).sin();
                        *current_direction = Vec2::from_angle(current_direction.to_angle() + random_turn * 0.8 * dt);
                        transform.translation += current_direction.extend(0.0) * enemy.speed * 0.6 * dt;
                    }
                }
            }
            
            EnemyAI::CellDivision { division_timer, .. } => {
                transform.translation.y -= enemy_clone.speed * 0.8 * dt;
                if *division_timer > 0.0 {
                    let wobble = (*division_timer * 10.0).sin() * 5.0;
                    transform.translation.x += wobble * dt;
                }
            }
            
            EnemyAI::SymbioticPair { sync_timer, bond_distance, .. } => {
                *sync_timer += dt;
                let sync_movement = Vec2::new(
                    (*sync_timer * 1.8).sin() * *bond_distance * 0.3,
                    (*sync_timer * 1.2).cos() * *bond_distance * 0.2,
                );
                let base_movement = Vec2::new(0.0, -enemy_clone.speed * 0.9);
                apply_current_influence(&mut transform, &fluid_environment, 0.5, dt);
                transform.translation += (base_movement + sync_movement).extend(0.0) * dt;
            }
            
            EnemyAI::FluidFlow { flow_sensitivity, base_direction } => {
                let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
                let current = sample_current(&fluid_environment, grid_pos);
                let flow_influence = current * *flow_sensitivity * dt * 3.0;
                *base_direction = (*base_direction + flow_influence).normalize_or_zero();
                
                let movement = *base_direction * enemy_clone.speed * 0.3 + current * 1.5;
                transform.translation += movement.extend(0.0) * dt;
                
                let angle = (current.x * 0.7 + base_direction.x * 0.3).atan2(current.y * 0.7 + base_direction.y * 0.3) - std::f32::consts::FRAC_PI_2;
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }
        
        // Apply chemical effects
        let avoidance = get_chemical_avoidance(transform.translation.truncate(), &chemical_environment, &enemy);
        if avoidance != Vec2::ZERO {
            transform.translation += avoidance.extend(0.0) * dt;
        }
        
        // Oxygen response
        if enemy.chemical_signature.responds_to_pheromones {
            let oxygen = sample_oxygen(&chemical_environment, transform.translation.truncate());
            if oxygen < enemy.chemical_signature.oxygen_tolerance {
                transform.translation.y += 20.0 * dt;
            }
        }
    }
}

pub fn update_spawner_enemies(
    mut commands: Commands,
    mut spawner_query: Query<(Entity, &Transform, &mut Enemy)>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    time: Res<Time>,
) {
    for (_, transform, mut enemy) in spawner_query.iter_mut() {
        if let EnemyAI::Spawner { spawn_timer, spawn_rate, minions_spawned, max_minions } = &mut enemy.ai_type {
            *spawn_timer -= time.delta_secs();
            
            if *spawn_timer <= 0.0 && *minions_spawned < *max_minions {
                let spawn_angle = (*minions_spawned as f32 * 1.2) + (time.elapsed_secs() * 0.5).sin();
                let spawn_distance = SPAWN_DISTANCE_BASE + (time.elapsed_secs() * 2.0).cos() * 10.0;
                let spawn_offset = Vec2::from_angle(spawn_angle) * spawn_distance;
                
                let ai_type = if *minions_spawned % 2 == 0 {
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
                };
                
                spawn_events.write(SpawnEnemy {
                    position: transform.translation + spawn_offset.extend(0.0),
                    ai_type,
                    enemy_type: EnemyType::Offspring,
                });
                
                *minions_spawned += 1;
                *spawn_timer = *spawn_rate * (0.8 + (time.elapsed_secs() * 0.3).sin() * 0.2);
            }
        }
    }
}

pub fn turret_shooting(
    mut commands: Commands,
    mut turret_query: Query<(&Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    let Some(assets) = assets else { return };
    let Ok(player_transform) = player_query.single() else { return };
    
    for (turret_transform, mut enemy) in turret_query.iter_mut() {
        let enemy_clone = enemy.clone();
        if let EnemyAI::Turret { shoot_timer, detection_range, rotation } = &mut enemy.ai_type {

            *shoot_timer -= time.delta_secs();
            
            let distance = turret_transform.translation.distance(player_transform.translation);
            if distance <= *detection_range && *shoot_timer <= 0.0 {
                let direction = (player_transform.translation.truncate() - turret_transform.translation.truncate()).normalize();
                
                let (color, damage, velocity, count) = match enemy_clone.enemy_type {
                    EnemyType::BiofilmColony => (Color::srgb(0.6, 0.8, 0.3), 25, 350.0, 3),
                    _ => (Color::srgb(0.8, 0.4, 0.4), 20, 400.0, 1),
                };
                
                for i in 0..count {
                    let spread_angle = if count > 1 { (i as f32 - 1.0) * 0.3 } else { 0.0 };
                    let spread_dir = Vec2::new(
                        direction.x * spread_angle.cos() - direction.y * spread_angle.sin(),
                        direction.x * spread_angle.sin() + direction.y * spread_angle.cos(),
                    );
                    
                    commands.spawn((
                        Sprite { image: assets.projectile_texture.clone(), color, ..default() },
                        Transform::from_translation(turret_transform.translation + Vec3::new(0.0, -15.0, 0.0))
                            .with_rotation(Quat::from_rotation_z(*rotation)),
                        Projectile {
                            velocity: spread_dir * velocity,
                            damage,
                            friendly: false,
                            organic_trail: enemy_clone.chemical_signature.releases_toxins,
                        },
                        Collider { radius: 4.0 },
                    ));
                }
                
                *shoot_timer = 1.2 + (time.elapsed_secs() * 0.8).sin() * 0.3;
            }
        }
    }
}

pub fn update_formations(
    mut colony_leader_query: Query<(&mut Transform, &mut ColonyLeader)>,
    mut colony_member_query: Query<(&mut Enemy, &Transform), Without<ColonyLeader>>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (mut leader_transform, mut colony) in colony_leader_query.iter_mut() {
        colony.pattern_timer += time.delta_secs();
        
        apply_current_influence(&mut leader_transform, &fluid_environment, 0.6, time.delta_secs());
        leader_transform.translation.y -= FORMATION_SPEED_NORMAL * time.delta_secs();
        
        let colony_movement = match colony.pattern_type {
            ColonyPattern::BiofilmFormation => Vec2::new(
                (colony.pattern_timer * 0.4).sin() * 30.0,
                (colony.pattern_timer * 0.6).cos() * 15.0,
            ),
            ColonyPattern::LinearChain => Vec2::new(
                (colony.pattern_timer * 1.2).sin() * 40.0,
                0.0,
            ),
            ColonyPattern::CircularCluster => {
                let radius_pulse = 20.0 + (colony.pattern_timer * 1.5).sin() * 10.0;
                Vec2::new(
                    (colony.pattern_timer * 0.8).cos() * radius_pulse,
                    (colony.pattern_timer * 0.8).sin() * radius_pulse * 0.5,
                ) * 0.5
            }
            ColonyPattern::SymbioticPair => Vec2::new(
                (colony.pattern_timer * 2.0).sin() * 25.0,
                (colony.pattern_timer * 1.5).cos() * 12.0,
            ),
        };
        
        leader_transform.translation += colony_movement.extend(0.0) * time.delta_secs();
        
        // Update member positions
        for (member_index, member_entity) in colony.members.iter().enumerate() {
            if let Ok((mut member_enemy, _)) = colony_member_query.get_mut(*member_entity) {
                if let EnemyAI::Formation { position_in_formation, leader_offset, .. } = &mut member_enemy.ai_type {
                    let new_pos = colony.pattern_type.get_position(member_index, colony.members.len(), colony.pattern_timer);
                    *position_in_formation = new_pos;
                    *leader_offset = colony_movement * 0.5;
                }
            }
        }
        
        colony.members.retain(|&member_entity| colony_member_query.get(member_entity).is_ok());
    }
}

pub fn formation_coordination_system(
    mut commands: Commands,
    mut colony_query: Query<(Entity, &Transform, &mut ColonyCommander)>,
    member_query: Query<(&Enemy, &ColonyMember, &Transform), Without<ColonyCommander>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>, Without<ColonyCommander>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    let Some(assets) = assets else { return };
    let Ok(player_transform) = player_query.single() else { return };
    
    for (_, commander_transform, mut colony) in colony_query.iter_mut() {
        colony.chemical_timer += time.delta_secs();
        
        if colony.coordination_pattern.execute(colony.chemical_timer) {
            match &colony.coordination_pattern {
                CoordinationPattern::ChemicalSignaling { .. } => {
                    for &member_entity in &colony.members {
                        if let Ok((_, member, member_transform)) = member_query.get(member_entity) {
                            let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
                            let (velocity, damage, color) = match member.role {
                                ColonyRole::Queen => (480.0, 35, Color::srgb(1.0, 0.3, 0.8)),
                                ColonyRole::Worker => (420.0, 25, Color::srgb(0.8, 0.9, 0.3)),
                                ColonyRole::Guardian => (380.0, 30, Color::srgb(0.3, 0.8, 0.9)),
                                ColonyRole::Symbiont => (350.0, 20, Color::srgb(0.9, 0.6, 1.0)),
                            };
                            
                            commands.spawn((
                                Sprite { image: assets.projectile_texture.clone(), color, ..default() },
                                Transform::from_translation(member_transform.translation),
                                Projectile { velocity: direction * velocity, damage, friendly: false, organic_trail: true },
                                Collider { radius: 5.0 },
                            ));
                        }
                    }
                }
                
                CoordinationPattern::BiofilmFormation { member_count, rotation_speed } => {
                    let rotation_offset = colony.chemical_timer * rotation_speed;
                    let angle_step = std::f32::consts::TAU / *member_count as f32;
                    
                    for i in 0..*member_count {
                        let angle = angle_step * i as f32 + rotation_offset;
                        let direction = Vec2::new(angle.cos(), angle.sin());
                        
                        commands.spawn((
                            Sprite { image: assets.projectile_texture.clone(), color: Color::srgb(0.6, 0.8, 0.4), ..default() },
                            Transform::from_translation(commander_transform.translation),
                            Projectile { velocity: direction * 340.0, damage: 22, friendly: false, organic_trail: true },
                            Collider { radius: 4.0 },
                        ));
                    }
                }
                
                _ => {} // Other patterns simplified
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
                    
                    // Spawn two offspring
                    let split_positions = [
                        transform.translation + Vec3::new(-30.0, 0.0, 0.0),
                        transform.translation + Vec3::new(30.0, 0.0, 0.0),
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
                    
                    commands.entity(enemy_entity).insert(AlreadyDespawned).despawn();
                }
            }
        }
    }
}

pub fn symbiotic_pair_system(
    mut commands: Commands,
    pair_query: Query<(Entity, &Transform, &Enemy), Without<AlreadyDespawned>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
) {
    let pair_data: Vec<(Entity, Vec3, Option<Entity>)> = pair_query.iter()
        .filter_map(|(entity, transform, enemy)| {
            if let EnemyAI::SymbioticPair { partner_entity, .. } = &enemy.ai_type {
                Some((entity, transform.translation, *partner_entity))
            } else {
                None
            }
        })
        .collect();
    
    for (entity, position, partner_entity) in pair_data {
        if let Some(partner) = partner_entity {
            if pair_query.get(partner).is_err() {
                explosion_events.write(SpawnExplosion { position, intensity: 1.2, enemy_type: None });
                commands.entity(entity).insert(AlreadyDespawned).despawn();
            }
        }
    }
}

pub fn link_symbiotic_pairs(mut pair_query: Query<(Entity, &Transform, &mut Enemy)>) {
    let unlinked: Vec<(Entity, Vec3)> = pair_query.iter()
        .filter_map(|(entity, transform, enemy)| {
            if let EnemyAI::SymbioticPair { partner_entity: None, .. } = &enemy.ai_type {
                Some((entity, transform.translation))
            } else {
                None
            }
        })
        .collect();
    
    for chunk in unlinked.chunks(2) {
        if chunk.len() == 2 {
            let (entity1, _) = chunk[0];
            let (entity2, _) = chunk[1];
            
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
    time: Res<Time>,
    mut colony_timer: Local<f32>,
) {
    *colony_timer += time.delta_secs();
    
    if enemy_spawner.wave_timer > 60.0 && *colony_timer >= 25.0 {
        *colony_timer = 0.0;
        
        let base_x = (time.elapsed_secs() * 40.0).sin() * 200.0;
        let spawned = match (time.elapsed_secs() as u32 / 25) % 4 {
            0 => spawn_colony_pattern(&mut commands, &mut spawn_events, base_x, "biofilm"),
            1 => spawn_colony_pattern(&mut commands, &mut spawn_events, base_x, "chain"),
            2 => spawn_colony_pattern(&mut commands, &mut spawn_events, base_x, "cluster"),
            _ => spawn_colony_pattern(&mut commands, &mut spawn_events, base_x, "hunters"),
        };
        
        enemy_spawner.enemies_spawned += spawned;
    }
}

fn spawn_colony_pattern(
    commands: &mut Commands,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    base_x: f32,
    pattern: &str,
) -> u32 {
    match pattern {
        "biofilm" => {
            let colony_id = (base_x * 1000.0) as u32;
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(base_x, 420.0, 0.0),
                ai_type: EnemyAI::Turret { rotation: 0.0, shoot_timer: 0.0, detection_range: DETECTION_RANGE_DEFAULT },
                enemy_type: EnemyType::BiofilmColony,
            });
            
            commands.spawn((
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
            ));
            
            for layer in 0..3 {
                let cells = 3 + layer;
                for i in 0..cells {
                    let angle = (i as f32 / cells as f32) * std::f32::consts::TAU;
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
            12
        }
        "chain" => {
            for i in 0..6 {
                spawn_events.write(SpawnEnemy {
                    position: Vec3::new(base_x, 420.0 - i as f32 * 35.0, 0.0),
                    ai_type: EnemyAI::Formation {
                        formation_id: (base_x * 1001.0) as u32,
                        position_in_formation: Vec2::new(0.0, -(i as f32 * 35.0)),
                        leader_offset: Vec2::ZERO,
                        formation_timer: i as f32 * 0.2,
                    },
                    enemy_type: EnemyType::AggressiveBacteria,
                });
            }
            6
        }
        "cluster" => {
            for i in 0..8 {
                let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                let pos = Vec2::from_angle(angle) * 60.0;
                
                spawn_events.write(SpawnEnemy {
                    position: Vec3::new(base_x + pos.x, 420.0 + pos.y, 0.0),
                    ai_type: EnemyAI::Formation {
                        formation_id: (base_x * 1002.0) as u32,
                        position_in_formation: pos,
                        leader_offset: Vec2::ZERO,
                        formation_timer: 0.0,
                    },
                    enemy_type: EnemyType::SwarmCell,
                });
            }
            8
        }
        "hunters" => {
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
        _ => 0,
    }
}

// ===== ENHANCED AI SYSTEMS =====

pub fn predator_prey_system(
    mut predator_query: Query<(&mut Transform, &mut Enemy, &PredatorPreyBehavior), Without<AlreadyDespawned>>,
    prey_query: Query<(Entity, &Transform, &Enemy), (With<Enemy>, Without<PredatorPreyBehavior>, Without<AlreadyDespawned>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    
    for (mut predator_transform, predator_enemy, behavior) in predator_query.iter_mut() {
        let mut hunting_target: Option<Vec3> = None;
        let mut fleeing_from: Option<Vec3> = None;
        
        for (_, prey_transform, prey_enemy) in prey_query.iter() {
            let distance = predator_transform.translation.distance(prey_transform.translation);
            
            if behavior.prey_types.contains(&prey_enemy.enemy_type) && distance < behavior.hunt_range {
                if hunting_target.is_none() || distance < predator_transform.translation.distance(hunting_target.unwrap()) {
                    hunting_target = Some(prey_transform.translation);
                }
            }
            
            if behavior.predator_types.contains(&prey_enemy.enemy_type) && distance < behavior.flee_range {
                fleeing_from = Some(prey_transform.translation);
                break;
            }
        }
        
        let dt = time.delta_secs();
        if let Some(flee_pos) = fleeing_from {
            let flee_direction = (predator_transform.translation - flee_pos).normalize();
            let panic_speed = predator_enemy.speed * (1.0 + behavior.fear_intensity);
            predator_transform.translation += flee_direction * panic_speed * dt;
            predator_transform.rotation *= Quat::from_rotation_z(dt * 8.0);
        } else if let Some(hunt_pos) = hunting_target {
            let hunt_direction = (hunt_pos - predator_transform.translation).normalize();
            let hunt_speed = predator_enemy.speed * behavior.hunting_speed_bonus;
            predator_transform.translation += hunt_direction * hunt_speed * dt;
            
            let angle = hunt_direction.y.atan2(hunt_direction.x) - std::f32::consts::FRAC_PI_2;
            predator_transform.rotation = Quat::from_rotation_z(angle);
        } else {
            let player_distance = predator_transform.translation.distance(player_transform.translation);
            if player_distance < behavior.hunt_range * 1.5 {
                let player_direction = (player_transform.translation - predator_transform.translation).normalize();
                predator_transform.translation += player_direction * predator_enemy.speed * dt;
            }
        }
    }
}

pub fn chemical_trail_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut trail_query: Query<(Entity, &mut ChemicalTrail, &Transform), Without<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut player_trail_timer: Local<f32>,
) {
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
    
    for (trail_entity, mut trail, _) in trail_query.iter_mut() {
        trail.strength -= trail.decay_rate * time.delta_secs();
        if trail.strength <= 0.0 {
            commands.entity(trail_entity).insert(AlreadyDespawned).despawn();
        }
    }
}

pub fn chemical_trail_following(
    mut enemy_query: Query<(&mut Transform, &mut Enemy), (With<Enemy>, Without<ChemicalTrail>)>,
    trail_query: Query<(&Transform, &ChemicalTrail), (With<ChemicalTrail>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut enemy_transform, mut enemy) in enemy_query.iter_mut() {
        match &enemy.ai_type {
            EnemyAI::Chemotaxis { .. } | EnemyAI::Linear { .. } => {
                let mut strongest_trail: Option<(Vec3, f32)> = None;
                
                for (trail_transform, trail) in trail_query.iter() {
                    let distance = enemy_transform.translation.distance(trail_transform.translation);
                    if distance < 80.0 {
                        let influence = trail.strength / (distance + 1.0);
                        if strongest_trail.is_none() || influence > strongest_trail.unwrap().1 {
                            strongest_trail = Some((trail_transform.translation, influence));
                        }
                    }
                }
                
                if let Some((trail_pos, influence)) = strongest_trail {
                    if influence > 0.1 {
                        let trail_direction = (trail_pos - enemy_transform.translation).normalize();
                        let follow_strength = (influence * 60.0).min(enemy.speed * 0.7);
                        enemy_transform.translation += trail_direction * follow_strength * time.delta_secs();
                        
                        let trail_2d = trail_direction.truncate();
                        match &mut enemy.ai_type {
                            EnemyAI::Chemotaxis { current_direction, .. } => {
                                *current_direction = (*current_direction + trail_2d * 0.3).normalize();
                            }
                            EnemyAI::Linear { direction } => {
                                *direction = (*direction + trail_2d * 0.1).normalize();
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

pub fn ecosystem_balance_system(
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut ecosystem: ResMut<EcosystemState>,
    enemy_query: Query<(&Enemy, &EcosystemRole)>,
    time: Res<Time>,
) {
    let mut role_counts = HashMap::new();
    let mut total_balance = 0.0;
    
    for (_, role) in enemy_query.iter() {
        let count = role_counts.entry(role.role.clone()).or_insert(0u32);
        *count += 1;
        total_balance += role.balance_factor;
    }
    
    let apex_count = *role_counts.get(&EcosystemRoleType::Apex).unwrap_or(&0);
    let primary_count = *role_counts.get(&EcosystemRoleType::Primary).unwrap_or(&0);
    let secondary_count = *role_counts.get(&EcosystemRoleType::Secondary).unwrap_or(&0);
    
    if apex_count > 3 {
        enemy_spawner.spawn_timer *= 1.5;
    }
    if primary_count < 2 && enemy_spawner.wave_timer > 10.0 {
        enemy_spawner.spawn_timer *= 0.7;
    }
    
    let total_enemies = enemy_query.iter().count() as f32 + 1.0;
    let balance_score = 1.0 - (
        (apex_count as f32 / total_enemies - 0.1).abs() +
        (primary_count as f32 / total_enemies - 0.4).abs() +
        (secondary_count as f32 / total_enemies - 0.5).abs()
    ) / 3.0;
    
    ecosystem.health = balance_score.clamp(0.0, 1.0);
    ecosystem.population_balance.pathogenic_threats = primary_count + apex_count;
    ecosystem.population_balance.beneficial_microbes = secondary_count;
}

pub fn adaptive_difficulty_system(
    mut adaptive_query: Query<(&mut AdaptiveDifficulty, &mut Enemy)>,
    player_query: Query<(&Player, &EvolutionSystem, &Health)>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    time: Res<Time>,
) {
    let Ok((player, evolution, health)) = player_query.single() else { return };
    
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
    
    for (mut adaptive, mut enemy) in adaptive_query.iter_mut() {
        let target_threat = player_strength * 0.8;
        let threat_diff = target_threat - adaptive.threat_level;
        adaptive.threat_level += threat_diff * adaptive.adaptation_rate * time.delta_secs();
        
        if adaptive.threat_level > 1.5 {
            enemy.speed *= 1.0 + (adaptive.threat_level - 1.5) * 0.1;
            enemy.health = (enemy.health as f32 * (1.0 + (adaptive.threat_level - 1.5) * 0.15)) as i32;
        }
        
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

pub fn pheromone_communication_system(
    mut commands: Commands,
    mut colony_query: Query<(&Transform, &mut ColonyCommander)>,
    mut member_query: Query<(&mut Enemy, &Transform, &ColonyMember)>,
    player_query: Query<&Transform, With<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    let Some(assets) = assets else { return };
    
    for (colony_transform, mut colony) in colony_query.iter_mut() {
        colony.chemical_timer += time.delta_secs();
        
        if colony.chemical_timer % 2.0 < 0.1 {
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
            
            if let Ok(player_transform) = player_query.single() {
                for &member_entity in &colony.members {
                    if let Ok((mut enemy, _, member)) = member_query.get_mut(member_entity) {
                        match member.role {
                            ColonyRole::Worker => enemy.speed = 200.0,
                            ColonyRole::Guardian => {
                                if let EnemyAI::Formation { position_in_formation, .. } = &mut enemy.ai_type {
                                    let intercept = player_transform.translation.truncate() - colony_transform.translation.truncate();
                                    *position_in_formation = intercept * 0.8;
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

// ===== ENEMY TYPE EXTENSIONS =====

impl EnemyType {
    pub fn get_predator_prey_behavior(&self) -> Option<PredatorPreyBehavior> {
        match self {
            EnemyType::InfectedMacrophage => Some(PredatorPreyBehavior {
                predator_types: vec![],
                prey_types: vec![EnemyType::ViralParticle, EnemyType::AggressiveBacteria, EnemyType::Offspring],
                hunt_range: 180.0,
                flee_range: 50.0,
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