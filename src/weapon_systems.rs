use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;

// Enhanced weapon shooting system
pub fn enhanced_shooting_system(
    mut commands: Commands,
    input_state: Res<InputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut WeaponSystem), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut shoot_timer: Local<f32>,
    mut laser_charging: Local<bool>,
    mut laser_charge_timer: Local<f32>,
) {
    if let Some(assets) = assets {
        *shoot_timer -= time.delta_secs();
        
        if let Ok((player_transform, mut weapon_system)) = player_query.single_mut() {
            let weapon = weapon_system.primary_weapon.clone();
            let upgrades = &weapon_system.weapon_upgrades;
            
            match weapon {
                WeaponType::Basic { damage, fire_rate } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        spawn_basic_projectile(&mut commands, &assets, player_transform, damage, upgrades);
                        *shoot_timer = fire_rate / upgrades.fire_rate_multiplier;
                    }
                }
                
                WeaponType::SpreadShot { damage, fire_rate, spread_count, spread_angle } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        spawn_spread_shot(&mut commands, &assets, player_transform, damage, spread_count, spread_angle, upgrades);
                        *shoot_timer = fire_rate / upgrades.fire_rate_multiplier;
                    }
                }
                
                WeaponType::Laser { damage, charge_time, duration, width } => {
                    if input_state.shooting {
                        if !*laser_charging {
                            *laser_charging = true;
                            *laser_charge_timer = 0.0;
                        }
                        *laser_charge_timer += time.delta_secs();
                        
                        if *laser_charge_timer >= charge_time {
                            spawn_laser_beam(&mut commands, &assets, player_transform, damage, duration, width, upgrades);
                            *laser_charging = false;
                            *laser_charge_timer = 0.0;
                        }
                    } else {
                        *laser_charging = false;
                        *laser_charge_timer = 0.0;
                    }
                }
                
                WeaponType::Missile { damage, fire_rate, homing_strength, blast_radius } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        let target = find_nearest_enemy(&enemy_query, player_transform.translation);
                        spawn_missile(&mut commands, &assets, player_transform, damage, target, homing_strength, blast_radius, upgrades);
                        *shoot_timer = fire_rate / upgrades.fire_rate_multiplier;
                    }
                }
                
                WeaponType::RapidFire { damage, fire_rate, .. } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        spawn_basic_projectile(&mut commands, &assets, player_transform, damage, upgrades);
                        *shoot_timer = fire_rate / upgrades.fire_rate_multiplier;
                    }
                }
            }
            
            // Smart bomb activation
            if keyboard.just_pressed(KeyCode::Space) && weapon_system.smart_bombs > 0 {
                spawn_smart_bomb(&mut commands, &assets, player_transform.translation);
                weapon_system.smart_bombs -= 1;
            }
        }
    }
}

fn spawn_basic_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    upgrades: &WeaponUpgrades,
) {
    let damage = (base_damage as f32 * upgrades.damage_multiplier) as i32;
    
    let mut projectile_bundle = (
        Sprite::from_image(assets.projectile_texture.clone()),
        Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0)),
        Projectile {
            velocity: Vec2::new(0.0, 800.0),
            damage,
            friendly: true,
        },
        Collider { radius: 4.0 },
    );
    
    let mut entity_commands = commands.spawn(projectile_bundle);
    
    if upgrades.explosive_rounds {
        entity_commands.insert(ExplosiveProjectile {
            blast_radius: 40.0,
            blast_damage: damage / 2,
        });
    }
    
    if upgrades.armor_piercing {
        entity_commands.insert(ArmorPiercing {
            pierce_count: 0,
            max_pierce: 3,
        });
    }
}

fn spawn_spread_shot(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    spread_count: u32,
    spread_angle: f32,
    upgrades: &WeaponUpgrades,
) {
    let damage = (base_damage as f32 * upgrades.damage_multiplier) as i32;
    let angle_step = spread_angle / (spread_count - 1) as f32;
    let start_angle = -spread_angle / 2.0;
    
    for i in 0..spread_count {
        let angle = start_angle + angle_step * i as f32;
        let direction = Vec2::new(angle.sin(), angle.cos());
        
        let mut entity_commands = commands.spawn((
            Sprite::from_image(assets.projectile_texture.clone()),
            Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            Projectile {
                velocity: direction * 700.0,
                damage,
                friendly: true,
            },
            Collider { radius: 4.0 },
        ));
        
        if upgrades.explosive_rounds {
            entity_commands.insert(ExplosiveProjectile {
                blast_radius: 30.0,
                blast_damage: damage / 3,
            });
        }
    }
}

fn spawn_laser_beam(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    duration: f32,
    width: f32,
    upgrades: &WeaponUpgrades,
) {
    let damage = (base_damage as f32 * upgrades.damage_multiplier) as i32;
    
    commands.spawn((
        Sprite {
            image: assets.projectile_texture.clone(),
            color: Color::srgb(1.0, 0.2, 0.2),
            custom_size: Some(Vec2::new(width, 800.0)),
            ..default()
        },
        Transform::from_translation(player_transform.translation + Vec3::new(0.0, 400.0, 0.0)),
        LaserBeam {
            timer: 0.0,
            max_duration: duration,
            damage_per_second: damage * 10,
            width,
            length: 800.0,
        },
        Collider { radius: width / 2.0 },
    ));
}

fn spawn_missile(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    target: Option<Entity>,
    homing_strength: f32,
    blast_radius: f32,
    upgrades: &WeaponUpgrades,
) {
    let damage = (base_damage as f32 * upgrades.damage_multiplier) as i32;
    let enhanced_homing = if upgrades.homing_enhancement { homing_strength * 1.5 } else { homing_strength };
    
    commands.spawn((
        Sprite {
            image: assets.projectile_texture.clone(),
            color: Color::srgb(1.0, 0.8, 0.2),
            ..default()
        },
        Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0)),
        Projectile {
            velocity: Vec2::new(0.0, 400.0),
            damage,
            friendly: true,
        },
        MissileProjectile {
            target,
            homing_strength: enhanced_homing,
            blast_radius,
            seek_timer: 0.0,
        },
        ExplosiveProjectile {
            blast_radius,
            blast_damage: damage,
        },
        Collider { radius: 6.0 },
    ));
}

fn spawn_smart_bomb(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
) {
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgb(0.8, 0.8, 1.0),
            ..default()
        },
        Transform::from_translation(position),
        SmartBombWave {
            timer: 0.0,
            max_time: 2.0,
            current_radius: 0.0,
            max_radius: 500.0,
            damage: 100,
        },
    ));
}

fn find_nearest_enemy(
    enemy_query: &Query<(Entity, &Transform), With<Enemy>>,
    player_pos: Vec3,
) -> Option<Entity> {
    enemy_query
        .iter()
        .min_by(|(_, a), (_, b)| {
            a.translation.distance(player_pos)
                .partial_cmp(&b.translation.distance(player_pos))
                .unwrap()
        })
        .map(|(entity, _)| entity)
}

// Missile homing system
pub fn update_missiles(
    mut missile_query: Query<(&mut Transform, &mut Projectile, &mut MissileProjectile)>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
) {
    for (mut transform, mut projectile, mut missile) in missile_query.iter_mut() {
        missile.seek_timer += time.delta_secs();
        
        if missile.seek_timer > 0.5 { // Start seeking after 0.5 seconds
            if let Some(target_entity) = missile.target {
                if let Ok((_, target_transform)) = enemy_query.get(target_entity) {
                    let direction_to_target = (target_transform.translation.truncate() - transform.translation.truncate()).normalize_or_zero();
                    let current_direction = projectile.velocity.normalize_or_zero();
                    
                    let new_direction = (current_direction + direction_to_target * missile.homing_strength * time.delta_secs()).normalize_or_zero();
                    projectile.velocity = new_direction * projectile.velocity.length();
                    
                    let angle = new_direction.y.atan2(new_direction.x) - std::f32::consts::FRAC_PI_2;
                    transform.rotation = Quat::from_rotation_z(angle);
                } else {
                    // Target destroyed, find new target
                    missile.target = find_nearest_enemy(&enemy_query, transform.translation);
                }
            } else {
                // No target, find one
                missile.target = find_nearest_enemy(&enemy_query, transform.translation);
            }
        }
    }
}

// Laser beam system
pub fn update_laser_beams(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &mut LaserBeam, &mut Sprite, &Transform)>,
    time: Res<Time>,
) {
    for (entity, mut laser, mut sprite, _) in laser_query.iter_mut() {
        laser.timer += time.delta_secs();
        
        if laser.timer >= laser.max_duration {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Fade out over time
        let alpha = 1.0 - (laser.timer / laser.max_duration);
        sprite.color.set_alpha(alpha);
    }
}

// Smart bomb system
pub fn update_smart_bombs(
    mut commands: Commands,
    mut smart_bomb_query: Query<(Entity, &mut SmartBombWave, &mut Transform, &mut Sprite)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<SmartBombWave>)>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    time: Res<Time>,
) {
    for (bomb_entity, mut bomb, mut bomb_transform, mut sprite) in smart_bomb_query.iter_mut() {
        bomb.timer += time.delta_secs();
        
        if bomb.timer >= bomb.max_time {
            commands.entity(bomb_entity).despawn();
            continue;
        }
        
        let progress = bomb.timer / bomb.max_time;
        bomb.current_radius = bomb.max_radius * progress;
        
        // Update visual
        let scale = progress * 10.0;
        bomb_transform.scale = Vec3::splat(scale);
        sprite.color.set_alpha(1.0 - progress);
        
        // Damage enemies within radius
        for (enemy_entity, enemy_transform, mut enemy_health) in enemy_query.iter_mut() {
            let distance = bomb_transform.translation.distance(enemy_transform.translation);
            if distance <= bomb.current_radius {
                enemy_health.0 -= bomb.damage;
                
                if enemy_health.0 <= 0 {
                    explosion_events.write(SpawnExplosion {
                        position: enemy_transform.translation,
                        intensity: 1.5,
                        enemy_type: None,
                    });
                    commands.entity(enemy_entity).despawn();
                }
            }
        }
    }
}

// Enhanced collision system for new projectile types
pub fn enhanced_projectile_collisions(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Collider, &Projectile, Option<&ExplosiveProjectile>, Option<&mut ArmorPiercing>)>,
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
                            intensity: 1.0,
                            enemy_type: None,
                        });
                        commands.entity(enemy_entity).despawn();
                    }
                }
            }
        }
        *last_laser_damage = 0.0;
    }
    
    // Regular projectile collisions with explosive and armor piercing
    for (proj_entity, proj_transform, proj_collider, projectile, explosive, armor_piercing) in projectile_query.iter() {
        if !projectile.friendly { continue; }
        
        for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health) in enemy_query.iter_mut() {

            let distance = proj_transform.translation.distance(enemy_transform.translation);
            if distance < proj_collider.radius + enemy_collider.radius {
                enemy_health.0 -= projectile.damage;
                
                // Handle explosive projectiles
                if let Some(explosive_proj) = explosive {
                    explosion_events.write(SpawnExplosion {
                        position: proj_transform.translation,
                        intensity: 1.2,
                        enemy_type: None,
                    });
                    
                    // Damage nearby enemies
                    for (nearby_entity, nearby_transform, _, mut nearby_health) in enemy_query.iter_mut() {
                        if nearby_entity != enemy_entity {
                            let blast_distance = proj_transform.translation.distance(nearby_transform.translation);
                            if blast_distance <= explosive_proj.blast_radius {
                                nearby_health.0 -= explosive_proj.blast_damage;
                            }
                        }
                    }
                }
                
                // Handle armor piercing
                if let Some(mut piercing) = armor_piercing {
                    piercing.pierce_count += 1;
                    if piercing.pierce_count < piercing.max_pierce {
                        // Don't despawn, continue through enemy
                        break;
                    }
                }
                
                commands.entity(proj_entity).despawn();
                
                if enemy_health.0 <= 0 {
                    explosion_events.write(SpawnExplosion {
                        position: enemy_transform.translation,
                        intensity: 1.0,
                        enemy_type: None,
                    });
                    commands.entity(enemy_entity).despawn();
                }
                break;
            }
        }
    }
}

// Formation coordination system
pub fn formation_coordination_system(
    mut commands: Commands,
    mut formation_query: Query<(Entity, &Transform, &mut FormationCommander)>,
    mut member_query: Query<(&mut Enemy, &FormationMember, &Transform), Without<FormationCommander>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>, Without<FormationCommander>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        if let Ok(player_transform) = player_query.single() {
            for (commander_entity, commander_transform, mut formation) in formation_query.iter_mut() {
                formation.coordination_timer += time.delta_secs();
                
                if formation.attack_pattern.execute(formation.coordination_timer) {
                    match &formation.attack_pattern {
                        AttackPattern::SynchronizedShoot { .. } => {
                            // All members shoot at once
                            for member_entity in &formation.members {
                                if let Ok((_, member, member_transform)) = member_query.get(*member_entity) {
                                    spawn_coordinated_projectile(&mut commands, &assets, member_transform, player_transform);
                                }
                            }
                        }
                        
                        AttackPattern::CircularBarrage { projectile_count, .. } => {
                            let angle_step = std::f32::consts::TAU / *projectile_count as f32;
                            for i in 0..*projectile_count {
                                let angle = angle_step * i as f32;
                                let direction = Vec2::new(angle.cos(), angle.sin());
                                spawn_barrage_projectile(&mut commands, &assets, commander_transform, direction);
                            }
                        }
                        
                        AttackPattern::FocusedAssault { .. } => {
                            // Multiple members focus fire on player
                            let member_count = formation.members.len().min(3);
                            for i in 0..member_count {
                                if let Some(member_entity) = formation.members.get(i) {
                                    if let Ok((_, _, member_transform)) = member_query.get(*member_entity) {
                                        spawn_focused_projectile(&mut commands, &assets, member_transform, player_transform);
                                    }
                                }
                            }
                        }
                        
                        _ => {}
                    }
                }
            }
        }
    }
}

fn spawn_coordinated_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    member_transform: &Transform,
    player_transform: &Transform,
) {
    let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
    
    commands.spawn((
        Sprite::from_image(assets.projectile_texture.clone()),
        Transform::from_translation(member_transform.translation),
        Projectile {
            velocity: direction * 350.0,
            damage: 20,
            friendly: false,
        },
        Collider { radius: 4.0 },
    ));
}

fn spawn_barrage_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    commander_transform: &Transform,
    direction: Vec2,
) {
    commands.spawn((
        Sprite::from_image(assets.projectile_texture.clone()),
        Transform::from_translation(commander_transform.translation),
        Projectile {
            velocity: direction * 300.0,
            damage: 15,
            friendly: false,
        },
        Collider { radius: 4.0 },
    ));
}

fn spawn_focused_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    member_transform: &Transform,
    player_transform: &Transform,
) {
    let direction = (player_transform.translation.truncate() - member_transform.translation.truncate()).normalize_or_zero();
    
    commands.spawn((
        Sprite::from_image(assets.projectile_texture.clone()),
        Transform::from_translation(member_transform.translation),
        Projectile {
            velocity: direction * 400.0,
            damage: 25,
            friendly: false,
        },
        Collider { radius: 4.0 },
    ));
}
