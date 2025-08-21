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