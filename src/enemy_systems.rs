use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;

// Enhanced enemy movement system
pub fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    formation_leader_query: Query<&Transform, (With<FormationLeader>, Without<Enemy>, Without<Player>)>,
    time: Res<Time>,
) {
    let player_pos = player_query.single().ok().map(|t| t.translation.truncate());
    
    for (mut transform, mut enemy) in enemy_query.iter_mut() {
        let mut enemy_clone = enemy.clone();

        match &mut enemy_clone.ai_type {
            EnemyAI::Static => {},
            
            EnemyAI::Linear { direction } => {
                transform.translation += (direction.extend(0.0) * enemy.speed * time.delta_secs());
            },
            
            EnemyAI::Sine { amplitude, frequency, phase } => {
                *phase += time.delta_secs() * *frequency;
                transform.translation.y -= enemy.speed * time.delta_secs();
                transform.translation.x += *amplitude * phase.sin() * time.delta_secs();
            },
            
            EnemyAI::MiniBoss { pattern: _, timer } => {
                *timer += time.delta_secs();
                transform.translation.y -= enemy.speed * 0.5 * time.delta_secs();
            },
            
            EnemyAI::Kamikaze { target_pos, dive_speed, acquired_target } => {
                if let Some(player_position) = player_pos {
                    if !*acquired_target {
                        *target_pos = player_position;
                        *acquired_target = true;
                    }
                    
                    let direction = (*target_pos - transform.translation.truncate()).normalize_or_zero();
                    transform.translation += direction.extend(0.0) * *dive_speed * time.delta_secs();
                    
                    // Rotate to face movement direction
                    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    transform.rotation = Quat::from_rotation_z(angle);
                } else {
                    // No player, move down slowly
                    transform.translation.y -= 50.0 * time.delta_secs();
                }
            },
            
            EnemyAI::Turret { rotation, shoot_timer: _, detection_range: _ } => {
                // Turrets don't move, but rotate to track player
                if let Some(player_position) = player_pos {
                    let direction = player_position - transform.translation.truncate();
                    let target_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    let angle_diff = (target_angle - *rotation + std::f32::consts::PI) % std::f32::consts::TAU - std::f32::consts::PI;
                    *rotation += angle_diff.clamp(-2.0 * time.delta_secs(), 2.0 * time.delta_secs());
                    transform.rotation = Quat::from_rotation_z(*rotation);
                }
            },
            
            EnemyAI::Formation { formation_id, position_in_formation, leader_offset, formation_timer } => {
                *formation_timer += time.delta_secs();
                
                // Find formation leader position
                let leader_pos = formation_leader_query.iter()
                    .find(|leader_transform| {
                        // This is a simplification - in practice you'd track leaders properly
                        true
                    })
                    .map(|t| t.translation.truncate())
                    .unwrap_or_else(|| Vec2::new(0.0, 400.0));
                
                let target_pos = leader_pos + *leader_offset + *position_in_formation;
                let direction = (target_pos - transform.translation.truncate()).normalize_or_zero();
                transform.translation += direction.extend(0.0) * enemy.speed * time.delta_secs();
            },
            
            EnemyAI::Spawner { spawn_timer: _, spawn_rate: _, minions_spawned: _, max_minions: _ } => {
                // Spawners move slowly downward
                transform.translation.y -= enemy.speed * time.delta_secs();
            },
        }
    }
}

// Spawner enemy behavior
pub fn update_spawner_enemies(
    mut commands: Commands,
    mut spawner_query: Query<(Entity, &Transform, &mut Enemy)>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    for (entity, transform, mut enemy) in spawner_query.iter_mut() {
        if let EnemyAI::Spawner { spawn_timer, spawn_rate, minions_spawned, max_minions } = &mut enemy.ai_type {
            *spawn_timer -= time.delta_secs();
            
            if *spawn_timer <= 0.0 && *minions_spawned < *max_minions {
                // Spawn a minion
                let angle = (*minions_spawned as f32 * 0.8) - 1.6; // Spread out spawn angles
                let spawn_offset = Vec2::new(angle.cos() * 30.0, angle.sin() * 30.0);
                
                spawn_events.write(SpawnEnemy {
                    position: transform.translation + spawn_offset.extend(0.0),
                    ai_type: EnemyAI::Linear { direction: Vec2::new(angle.cos() * 0.3, -1.0).normalize() },
                    enemy_type: EnemyType::SpawnerMinion,
                });
                
                *minions_spawned += 1;
                *spawn_timer = *spawn_rate;
            }
        }
    }
}

// Turret shooting system
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
                if let EnemyAI::Turret { rotation, shoot_timer, detection_range } = &mut enemy.ai_type {
                    *shoot_timer -= time.delta_secs();
                    
                    let distance = turret_transform.translation.distance(player_transform.translation);
                    
                    if distance <= *detection_range && *shoot_timer <= 0.0 {
                        let direction = (player_transform.translation.truncate() - turret_transform.translation.truncate()).normalize();
                        
                        // Spawn projectile
                        commands.spawn((
                            Sprite::from_image(assets.projectile_texture.clone()),
                            Transform::from_translation(turret_transform.translation + Vec3::new(0.0, -15.0, 0.0))
                                .with_rotation(Quat::from_rotation_z(*rotation)),
                            Projectile {
                                velocity: direction * 400.0,
                                damage: 20,
                                friendly: false,
                            },
                            Collider { radius: 4.0 },
                        ));
                        
                        *shoot_timer = 1.0; // Shoot every second
                    }
                }
            }
        }
    }
}

// Formation system
pub fn update_formations(
    mut formation_leader_query: Query<(&mut Transform, &mut FormationLeader)>,
    mut formation_member_query: Query<(&mut Enemy, &Transform), Without<FormationLeader>>,
    time: Res<Time>,
) {
    for (mut leader_transform, mut formation) in formation_leader_query.iter_mut() {
        formation.pattern_timer += time.delta_secs();
        
        // Move formation leader
        leader_transform.translation.y -= 100.0 * time.delta_secs();
        
        // Update formation pattern
        let wave_offset = match formation.pattern_type {
            FormationPattern::CircleFormation => {
                Vec2::new((formation.pattern_timer * 0.5).sin() * 50.0, 0.0)
            },
            FormationPattern::VFormation => {
                Vec2::new((formation.pattern_timer * 0.8).sin() * 30.0, 0.0)
            },
            _ => Vec2::ZERO,
        };
        
        leader_transform.translation += wave_offset.extend(0.0) * time.delta_secs();
        
        // Update member positions
        for (member_index, member_entity) in formation.members.iter().enumerate() {
            if let Ok((mut member_enemy, _)) = formation_member_query.get_mut(*member_entity) {
                if let EnemyAI::Formation { position_in_formation, leader_offset, .. } = &mut member_enemy.ai_type {
                    let new_pos = formation.pattern_type.get_position(
                        member_index, 
                        formation.members.len(), 
                        formation.pattern_timer
                    );
                    *position_in_formation = new_pos;
                    *leader_offset = wave_offset;
                }
            }
        }
    }
}

// Enhanced enemy spawning with variety
pub fn spawn_enemies_enhanced(
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    mut commands: Commands,
    time: Res<Time>,
) {
    enemy_spawner.spawn_timer -= time.delta_secs();
    enemy_spawner.wave_timer += time.delta_secs();
    
    if enemy_spawner.spawn_timer <= 0.0 {
        let wave_time = enemy_spawner.wave_timer;
        let spawn_count = (enemy_spawner.enemies_spawned % 20) as usize;
        
        match wave_time as u32 / 15 % 6 {
            0 => spawn_basic_wave(&mut spawn_events, spawn_count),
            1 => spawn_kamikaze_wave(&mut spawn_events, spawn_count),
            2 => spawn_turret_wave(&mut spawn_events, spawn_count),
            3 => spawn_formation_wave(&mut spawn_events, &mut commands, spawn_count),
            4 => spawn_spawner_wave(&mut spawn_events, spawn_count),
            _ => spawn_mixed_wave(&mut spawn_events, spawn_count),
        }
        
        enemy_spawner.enemies_spawned += 1;
        
        let spawn_rate = (2.5 - (wave_time * 0.02)).max(0.4);
        enemy_spawner.spawn_timer = spawn_rate;
    }
}

fn spawn_basic_wave(spawn_events: &mut EventWriter<SpawnEnemy>, spawn_count: usize) {
    let x_pos = [-300.0, -150.0, 0.0, 150.0, 300.0][spawn_count % 5];
    
    spawn_events.write(SpawnEnemy {
        position: Vec3::new(x_pos, 400.0, 0.0),
        ai_type: EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) },
        enemy_type: EnemyType::Basic,
    });
}

fn spawn_kamikaze_wave(spawn_events: &mut EventWriter<SpawnEnemy>, spawn_count: usize) {
    let x_pos = [-400.0, -200.0, 0.0, 200.0, 400.0][spawn_count % 5];
    
    spawn_events.write(SpawnEnemy {
        position: Vec3::new(x_pos, 450.0, 0.0),
        ai_type: EnemyAI::Kamikaze { 
            target_pos: Vec2::ZERO, 
            dive_speed: 350.0, 
            acquired_target: false 
        },
        enemy_type: EnemyType::Kamikaze,
    });
}

fn spawn_turret_wave(spawn_events: &mut EventWriter<SpawnEnemy>, spawn_count: usize) {
    let x_pos = [-250.0, 0.0, 250.0][spawn_count % 3];
    
    spawn_events.write(SpawnEnemy {
        position: Vec3::new(x_pos, 350.0, 0.0),
        ai_type: EnemyAI::Turret { 
            rotation: 0.0, 
            shoot_timer: 0.0, 
            detection_range: 300.0 
        },
        enemy_type: EnemyType::Turret,
    });
}

fn spawn_formation_wave(spawn_events: &mut EventWriter<SpawnEnemy>, commands: &mut Commands, spawn_count: usize) {
    let formation_id = spawn_count as u32;
    let pattern = match spawn_count % 4 {
        0 => FormationPattern::VFormation,
        1 => FormationPattern::LineFormation,
        2 => FormationPattern::CircleFormation,
        _ => FormationPattern::DiamondFormation,
    };
    
    let member_count = match pattern {
        FormationPattern::VFormation => 5,
        FormationPattern::LineFormation => 4,
        FormationPattern::CircleFormation => 6,
        FormationPattern::DiamondFormation => 4,
    };
    
    // Spawn formation leader (invisible)
    let leader_entity = commands.spawn((
        Transform::from_xyz(0.0, 450.0, 0.0),
        FormationLeader {
            formation_id,
            members: Vec::new(),
            pattern_timer: 0.0,
            pattern_type: pattern.clone(),
        },
    )).id();
    
    // Spawn formation members
    for i in 0..member_count {
        let pos = pattern.get_position(i, member_count, 0.0);
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(pos.x, 450.0 + pos.y, 0.0),
            ai_type: EnemyAI::Formation {
                formation_id,
                position_in_formation: pos,
                leader_offset: Vec2::ZERO,
                formation_timer: 0.0,
            },
            enemy_type: EnemyType::FormationFighter,
        });
    }
}

fn spawn_spawner_wave(spawn_events: &mut EventWriter<SpawnEnemy>, spawn_count: usize) {
    let x_pos = [-200.0, 200.0][spawn_count % 2];
    
    spawn_events.write(SpawnEnemy {
        position: Vec3::new(x_pos, 400.0, 0.0),
        ai_type: EnemyAI::Spawner { 
            spawn_timer: 2.0, 
            spawn_rate: 1.5, 
            minions_spawned: 0, 
            max_minions: 4 
        },
        enemy_type: EnemyType::Spawner,
    });
}

fn spawn_mixed_wave(spawn_events: &mut EventWriter<SpawnEnemy>, spawn_count: usize) {
    match spawn_count % 4 {
        0 => spawn_basic_wave(spawn_events, spawn_count),
        1 => spawn_kamikaze_wave(spawn_events, spawn_count),
        2 => spawn_turret_wave(spawn_events, spawn_count),
        _ => spawn_spawner_wave(spawn_events, spawn_count),
    }
}

// Enhanced formation spawning with coordination
pub fn spawn_coordinated_formation(
    mut commands: Commands,
    mut spawn_events: EventWriter<SpawnEnemy>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    time: Res<Time>,
) {
    // Spawn coordinated formations every 30 seconds after wave 20
    if enemy_spawner.wave_timer > 20.0 && enemy_spawner.wave_timer % 30.0 < 0.1 {
        let formation_id = (enemy_spawner.wave_timer as u32 / 30) + 1000;
        
        // Choose formation pattern based on time
        let (pattern_type, attack_pattern, member_count) = match (formation_id % 4) {
            0 => (
                FormationPattern::VFormation,
                AttackPattern::SynchronizedShoot { interval: 2.0 },
                5
            ),
            1 => (
                FormationPattern::CircleFormation,
                AttackPattern::CircularBarrage { projectile_count: 8, rotation_speed: 1.0 },
                6
            ),
            2 => (
                FormationPattern::LineFormation,
                AttackPattern::WaveAttack { wave_size: 3, wave_delay: 1.5 },
                4
            ),
            _ => (
                FormationPattern::DiamondFormation,
                AttackPattern::FocusedAssault { target_focus: true },
                4
            ),
        };
        
        // Spawn formation commander (invisible coordinator)
        let commander_entity = commands.spawn((
            Transform::from_xyz(0.0, 450.0, 0.0),
            FormationCommander {
                formation_id,
                members: Vec::new(),
                attack_pattern,
                coordination_timer: 0.0,
            },
        )).id();
        
        // Spawn formation members
        let mut member_entities = Vec::new();
        for i in 0..member_count {
            let position = pattern_type.get_position(i, member_count, 0.0);
            let role = match i {
                0 => FormationRole::Leader,
                1..=2 => FormationRole::Attacker,
                _ => FormationRole::Support,
            };
            
            let member_entity = commands.spawn((
                // Placeholder transform, will be set by spawn_enemy_system
                Transform::from_xyz(position.x, 450.0 + position.y, 0.0),
                FormationMember {
                    formation_id,
                    role,
                    last_command_time: 0.0,
                },
            )).id();
            
            member_entities.push(member_entity);
            
            // Send spawn event for the actual enemy
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(position.x, 450.0 + position.y, 0.0),
                ai_type: EnemyAI::Formation {
                    formation_id,
                    position_in_formation: position,
                    leader_offset: Vec2::ZERO,
                    formation_timer: 0.0,
                },
                enemy_type: match role {
                    FormationRole::Leader => EnemyType::Heavy,
                    FormationRole::Attacker => EnemyType::FormationFighter,
                    _ => EnemyType::Basic,
                },
            });
        }
        
        // Update commander with member list
        if let Ok(mut commander) = commands.entity(commander_entity).get_mut::<FormationCommander>() {
            commander.members = member_entities;
        }
    }
}

// Advanced formation coordination with tactical behaviors
pub fn advanced_formation_coordination(
    mut commands: Commands,
    mut formation_query: Query<(Entity, &Transform, &mut FormationCommander)>,
    mut member_query: Query<(&mut Enemy, &FormationMember, &Transform, Entity), Without<FormationCommander>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>, Without<FormationCommander>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        if let Ok(player_transform) = player_query.single() {
            for (commander_entity, commander_transform, mut formation) in formation_query.iter_mut() {
                formation.coordination_timer += time.delta_secs();
                
                // Clean up dead members
                formation.members.retain(|&member_entity| {
                    member_query.get(member_entity).is_ok()
                });
                
                // Disband formation if too few members
                if formation.members.len() < 2 {
                    commands.entity(commander_entity).despawn();
                    continue;
                }
                
                // Execute formation attack patterns
                if formation.attack_pattern.execute(formation.coordination_timer) {
                    match &formation.attack_pattern {
                        AttackPattern::SynchronizedShoot { interval } => {
                            execute_synchronized_attack(&mut commands, &assets, &formation, &member_query, player_transform);
                        }
                        
                        AttackPattern::WaveAttack { wave_size, wave_delay } => {
                            execute_wave_attack(&mut commands, &assets, &formation, &member_query, player_transform, *wave_size);
                        }
                        
                        AttackPattern::CircularBarrage { projectile_count, rotation_speed } => {
                            execute_circular_barrage(&mut commands, &assets, commander_transform, *projectile_count, formation.coordination_timer * rotation_speed);
                        }
                        
                        AttackPattern::FocusedAssault { target_focus } => {
                            execute_focused_assault(&mut commands, &assets, &formation, &member_query, player_transform);
                        }
                    }
                }
                
                // Formation-specific behaviors
                execute_formation_maneuvers(&mut commands, &formation, &mut member_query, time.delta_secs());
            }
        }
    }
}

fn execute_synchronized_attack(
    commands: &mut Commands,
    assets: &GameAssets,
    formation: &FormationCommander,
    member_query: &Query<(&mut Enemy, &FormationMember, &Transform, Entity), Without<FormationCommander>>,
    player_transform: &Transform,
) {
    for &member_entity in &formation.members {
        if let Ok((_, member, member_transform, _)) = member_query.get(member_entity) {
            let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
            
            // Enhanced projectiles based on role
            let (velocity, damage, color) = match member.role {
                FormationRole::Leader => (450.0, 30, Color::srgb(1.0, 0.3, 0.3)),
                FormationRole::Attacker => (400.0, 25, Color::srgb(1.0, 0.8, 0.2)),
                _ => (350.0, 20, Color::WHITE),
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
                },
                Collider { radius: 4.0 },
            ));
        }
    }
}

fn execute_wave_attack(
    commands: &mut Commands,
    assets: &GameAssets,
    formation: &FormationCommander,
    member_query: &Query<(&mut Enemy, &FormationMember, &Transform, Entity), Without<FormationCommander>>,
    player_transform: &Transform,
    wave_size: u32,
) {
    let attackers: Vec<_> = formation.members.iter()
        .filter_map(|&entity| member_query.get(entity).ok())
        .filter(|(_, member, _, _)| matches!(member.role, FormationRole::Attacker | FormationRole::Leader))
        .take(wave_size as usize)
        .collect();
    
    for (_, member, member_transform, _) in attackers {
        let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
        
        // Spawn multiple projectiles per attacker in wave
        for i in 0..3 {
            let spread_angle = (i as f32 - 1.0) * 0.2;
            let spread_direction = Vec2::new(
                direction.x * spread_angle.cos() - direction.y * spread_angle.sin(),
                direction.x * spread_angle.sin() + direction.y * spread_angle.cos(),
            );
            
            commands.spawn((
                Sprite {
                    image: assets.projectile_texture.clone(),
                    color: Color::srgb(0.8, 0.4, 1.0),
                    ..default()
                },
                Transform::from_translation(member_transform.translation),
                Projectile {
                    velocity: spread_direction * 380.0,
                    damage: 22,
                    friendly: false,
                },
                Collider { radius: 4.0 },
            ));
        }
    }
}

fn execute_circular_barrage(
    commands: &mut Commands,
    assets: &GameAssets,
    commander_transform: &Transform,
    projectile_count: u32,
    rotation_offset: f32,
) {
    let angle_step = std::f32::consts::TAU / projectile_count as f32;
    
    for i in 0..projectile_count {
        let angle = angle_step * i as f32 + rotation_offset;
        let direction = Vec2::new(angle.cos(), angle.sin());
        
        commands.spawn((
            Sprite {
                image: assets.projectile_texture.clone(),
                color: Color::srgb(1.0, 0.5, 0.8),
                ..default()
            },
            Transform::from_translation(commander_transform.translation),
            Projectile {
                velocity: direction * 320.0,
                damage: 18,
                friendly: false,
            },
            Collider { radius: 4.0 },
        ));
    }
}

fn execute_focused_assault(
    commands: &mut Commands,
    assets: &GameAssets,
    formation: &FormationCommander,
    member_query: &Query<(&mut Enemy, &FormationMember, &Transform, Entity), Without<FormationCommander>>,
    player_transform: &Transform,
) {
    // All members focus fire on player with prediction
    for &member_entity in &formation.members {
        if let Ok((_, member, member_transform, _)) = member_query.get(member_entity) {
            // Predict player position
            let distance = member_transform.translation.distance(player_transform.translation);
            let time_to_hit = distance / 400.0;
            let predicted_position = player_transform.translation; // Simplified - could add velocity prediction
            
            let direction = (predicted_position.truncate() - member_transform.translation.truncate()).normalize_or_zero();
            
            // High damage focused shot
            commands.spawn((
                Sprite {
                    image: assets.projectile_texture.clone(),
                    color: Color::srgb(1.0, 0.2, 0.2),
                    custom_size: Some(Vec2::splat(8.0)),
                    ..default()
                },
                Transform::from_translation(member_transform.translation),
                Projectile {
                    velocity: direction * 500.0,
                    damage: 35,
                    friendly: false,
                },
                Collider { radius: 6.0 },
            ));
        }
    }
}

fn execute_formation_maneuvers(
    commands: &mut Commands,
    formation: &FormationCommander,
    member_query: &mut Query<(&mut Enemy, &FormationMember, &Transform, Entity), Without<FormationCommander>>,
    delta_time: f32,
) {
    // Give special orders to different roles
    for &member_entity in &formation.members {
        if let Ok((mut enemy, member, transform, entity)) = member_query.get_mut(member_entity) {
            match member.role {
                FormationRole::Leader => {
                    // Leader moves more aggressively
                    if let EnemyAI::Formation { formation_timer, .. } = &mut enemy.ai_type {
                        *formation_timer += delta_time;
                        // Leaders can break formation to chase player occasionally
                        if *formation_timer > 10.0 && (*formation_timer % 15.0) < 3.0 {
                            // Temporarily increase speed
                            enemy.speed = 250.0;
                        } else {
                            enemy.speed = 180.0;
                        }
                    }
                }
                
                FormationRole::Attacker => {
                    // Attackers maintain optimal firing positions
                    if let EnemyAI::Formation { position_in_formation, .. } = &mut enemy.ai_type {
                        // Adjust position for better line of sight
                        let adjustment = Vec2::new(
                            (formation.coordination_timer * 0.5).sin() * 10.0,
                            (formation.coordination_timer * 0.3).cos() * 5.0,
                        );
                        *position_in_formation += adjustment * delta_time;
                    }
                }
                
                FormationRole::Support => {
                    // Support units protect the formation
                    if formation.members.len() < 4 {
                        // Become more aggressive when formation is weakened
                        enemy.speed = 200.0;
                    }
                }
                
                _ => {}
            }
        }
    }
}

// Formation-based enemy spawning enhancement
pub fn spawn_elite_formations(
    mut commands: Commands,
    mut spawn_events: EventWriter<SpawnEnemy>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    time: Res<Time>,
) {
    // Spawn elite formations every 45 seconds in late game
    if enemy_spawner.wave_timer > 60.0 && enemy_spawner.wave_timer % 45.0 < 0.1 {
        let formation_id = (enemy_spawner.wave_timer as u32 / 45) + 2000;
        
        // Elite formation with mixed enemy types and advanced tactics
        let commander_entity = commands.spawn((
            Transform::from_xyz(0.0, 500.0, 0.0),
            FormationCommander {
                formation_id,
                members: Vec::new(),
                attack_pattern: AttackPattern::FocusedAssault { target_focus: true },
                coordination_timer: 0.0,
            },
        )).id();
        
        // Spawn 1 boss, 2 heavies, 3 attackers
        let elite_composition = [
            (EnemyType::Boss, FormationRole::Leader, Vec2::new(0.0, 0.0)),
            (EnemyType::Heavy, FormationRole::Defender, Vec2::new(-60.0, -40.0)),
            (EnemyType::Heavy, FormationRole::Defender, Vec2::new(60.0, -40.0)),
            (EnemyType::FormationFighter, FormationRole::Attacker, Vec2::new(-100.0, -80.0)),
            (EnemyType::FormationFighter, FormationRole::Attacker, Vec2::new(0.0, -80.0)),
            (EnemyType::FormationFighter, FormationRole::Attacker, Vec2::new(100.0, -80.0)),
        ];
        
        let mut member_entities = Vec::new();
        for (enemy_type, role, position) in elite_composition {
            let member_entity = commands.spawn((
                Transform::from_xyz(position.x, 500.0 + position.y, 0.0),
                FormationMember {
                    formation_id,
                    role,
                    last_command_time: 0.0,
                },
            )).id();
            
            member_entities.push(member_entity);
            
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(position.x, 500.0 + position.y, 0.0),
                ai_type: EnemyAI::Formation {
                    formation_id,
                    position_in_formation: position,
                    leader_offset: Vec2::ZERO,
                    formation_timer: 0.0,
                },
                enemy_type,
            });
        }
        
        // Update commander with member list
        if let Ok(mut commander) = commands.entity(commander_entity).get_mut::<FormationCommander>() {
            commander.members = member_entities;
        }
    }
}