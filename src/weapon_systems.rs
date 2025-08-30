use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;
use crate::input::*;

// ===== CONSTANTS =====
const WING_CANNON_OFFSET: f32 = 25.0; // Distance from center
const WING_CANNON_Y_OFFSET: f32 = 10.0;
const MISSILE_LAUNCH_OFFSET: f32 = 20.0;
const MISSILE_Y_OFFSET: f32 = -15.0;

// Wing Cannon stats per level
const WING_CANNON_STATS: [(f32, i32, f32, u32); 5] = [
    (1.2, 25, 12.0, 2), // fire_rate, damage, size, pierce
    (1.0, 35, 14.0, 3),
    (0.8, 50, 16.0, 4),
    (0.6, 70, 18.0, 5),
    (0.5, 95, 20.0, 6),
];

// Missile system stats per level
const MISSILE_STATS: [(f32, i32, f32, f32, bool); 5] = [
    (2.5, 80, 400.0, 200.0, false), // fire_rate, damage, speed, range, dual
    (2.0, 110, 450.0, 250.0, false),
    (1.8, 150, 500.0, 300.0, false),
    (1.5, 200, 550.0, 350.0, true),
    (1.2, 260, 600.0, 400.0, true),
];

// New components for biological weapons
#[derive(Component)]
pub struct ToxinCloudEffect {
    pub timer: f32,
    pub max_duration: f32,
    pub damage_per_second: i32,
    pub radius: f32,
    pub intensity: f32,
}

#[derive(Component)]
pub struct ElectricArc {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub damage: i32,
    pub chain_index: u32,
    pub timer: f32,
    pub max_duration: f32,
    pub target_entity: Option<Entity>,
}


pub fn setup_player_weapons(
    mut commands: Commands,
    player_query: Query<(Entity, &UpgradeLimits), With<Player>>,
) {
    if let Ok((player_entity, limits)) = player_query.single() {
        // Setup Wing Cannons if purchased
        if limits.wing_cannon_level > 0 {
            let (fire_rate, damage, size, pierce) = WING_CANNON_STATS[(limits.wing_cannon_level - 1) as usize];
            
            commands.entity(player_entity).insert((
                WingCannon {
                    level: limits.wing_cannon_level,
                    fire_timer: 0.0,
                    fire_rate,
                    damage,
                    projectile_size: size,
                    side: WingCannonSide::Left,
                },
            ));
        }

        // Setup Missile System if purchased
        if limits.missile_level > 0 {
            let (fire_rate, damage, speed, range, dual) = MISSILE_STATS[(limits.missile_level - 1) as usize];
            
            commands.entity(player_entity).insert((
                MissileSystem {
                    level: limits.missile_level,
                    fire_timer: 0.0,
                    fire_rate,
                    damage,
                    missile_speed: speed,
                    homing_range: range,
                    dual_launch: dual,
                },
            ));
        }
    }
}


// UNIFIED SYSTEM

pub fn unified_weapon_update_system(
    mut commands: Commands,
    // Use ParamSet to separate conflicting weapon queries
    mut weapon_queries: ParamSet<(
        // Query 0: Missile projectiles
        Query<(
            Entity,
            &mut Transform, 
            &mut Projectile, 
            &mut MissileProjectile
        ), (With<MissileProjectile>, Without<LaserBeam>, Without<SporeWave>, Without<ToxinCloudEffect>)>,
        
        // Query 1: Laser beams
        Query<(
            Entity, 
            &mut LaserBeam, 
            &mut Sprite, 
            &Transform, 
            Option<&BioluminescentParticle>
        ), (With<LaserBeam>, Without<MissileProjectile>, Without<SporeWave>, Without<ToxinCloudEffect>)>,
        
        // Query 2: Emergency spores 
        Query<(
            Entity, 
            &mut SporeWave, 
            &mut Transform, 
            &mut Sprite
        ), (With<SporeWave>, Without<LaserBeam>, Without<MissileProjectile>, Without<ToxinCloudEffect>)>,
        
        // Query 3: Toxin clouds
        Query<(
            Entity,
            &mut ToxinCloudEffect,
            &mut Transform,
            &mut Sprite
        ), (With<ToxinCloudEffect>, Without<LaserBeam>, Without<MissileProjectile>, Without<SporeWave>)>,
        
        // Query 4: Electric arcs (no Transform conflicts)
        Query<(Entity, &mut ElectricArc)>,
    )>,
    
    // Separate enemy queries to avoid conflicts
    enemy_query: Query<(Entity, &Transform), (With<Enemy>, Without<MissileProjectile>, Without<SporeWave>, Without<LaserBeam>, Without<ToxinCloudEffect>)>,
    mut enemy_health_query: Query<(Entity, &Transform, &Collider, &mut Health), (With<Enemy>, Without<ToxinCloudEffect>, Without<LaserBeam>, Without<SporeWave>, Without<MissileProjectile>)>,
    
    // Events and resources
    mut explosion_events: EventWriter<SpawnExplosion>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    // 1. UPDATE HOMING MISSILES
    {
        let mut missiles = weapon_queries.p0();
        for (missile_entity, mut transform, mut projectile, mut missile) in missiles.iter_mut() {
            missile.seek_timer += time.delta_secs();
            
            if missile.seek_timer > 0.3 {
                if let Some(target_entity) = missile.target {
                    if let Ok((_, target_transform)) = enemy_query.get(target_entity) {
                        let direction_to_target = (target_transform.translation.truncate() - transform.translation.truncate()).normalize_or_zero();
                        let current_direction = projectile.velocity.normalize_or_zero();
                        
                        // Enhanced homing with organic smoothness
                        let homing_rate = if missile.symbiotic { 
                            missile.homing_strength * 1.5 
                        } else { 
                            missile.homing_strength 
                        };
                        
                        let new_direction = (current_direction + direction_to_target * homing_rate * time.delta_secs()).normalize_or_zero();
                        projectile.velocity = new_direction * projectile.velocity.length();
                        
                        // Organic rotation with wobble
                        let angle = new_direction.y.atan2(new_direction.x) - std::f32::consts::FRAC_PI_2;
                        let wobble = if missile.symbiotic { 
                            (time.elapsed_secs() * 6.0).sin() * 0.05 
                        } else { 
                            0.0 
                        };
                        transform.rotation = Quat::from_rotation_z(angle + wobble);
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

    // 2. UPDATE LASER BEAMS
    if let Some(assets) = &assets {
        let mut lasers = weapon_queries.p1();
        for (entity, mut laser, mut sprite, transform, bio_particle) in lasers.iter_mut() {
            laser.timer += time.delta_secs();
            
            if laser.timer >= laser.max_duration {
                commands.entity(entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                continue;
            }
            
            // Enhanced fade with bioluminescent pulsing
            let base_alpha = 1.0 - (laser.timer / laser.max_duration);
            
            if laser.bioluminescent {
                let pulse = (time.elapsed_secs() * 8.0).sin() * 0.3 + 0.7;
                sprite.color.set_alpha(base_alpha * pulse);
                
                // Spawn bioluminescent particles along beam
                if (time.elapsed_secs() * 20.0) % 1.0 < 0.1 {
                    for i in 0..5 {
                        let y_offset = (i as f32 - 2.0) * laser.length / 5.0;
                        commands.spawn((
                            Sprite {
                                image: assets.particle_texture.clone(),
                                color: Color::srgba(0.3, 1.0, 0.8, 0.8),
                                custom_size: Some(Vec2::splat(3.0)),
                                ..default()
                            },
                            Transform::from_translation(transform.translation + Vec3::new(0.0, y_offset, 0.1)),
                            Particle {
                                velocity: Vec2::new(
                                    (time.elapsed_secs() * 234.56).sin() * 20.0,
                                    (time.elapsed_secs() * 345.67).cos() * 15.0,
                                ),
                                lifetime: 0.0,
                                max_lifetime: 0.8,
                                size: 3.0,
                                fade_rate: 1.0,
                                bioluminescent: true,
                                drift_pattern: DriftPattern::Floating,
                            },
                        ));
                    }
                }
            } else {
                sprite.color.set_alpha(base_alpha);
            }
        }
    }

    // 3. UPDATE EMERGENCY SPORES (Smart Bombs)
    {
        let mut spores = weapon_queries.p2();
        for (spore_entity, mut spore, mut spore_transform, mut sprite) in spores.iter_mut() {
            spore.timer += time.delta_secs();
            
            if spore.timer >= spore.max_time {
                commands.entity(spore_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                continue;
            }
            
            let progress = spore.timer / spore.max_time;
            spore.current_radius = spore.max_radius * progress;
            
            // Enhanced organic visual effects
            let scale = progress * 12.0;
            let pulse = (time.elapsed_secs() * 4.0).sin() * 0.1 + 0.9;
            spore_transform.scale = Vec3::splat(scale * pulse);
            
            // Organic color transition
            let alpha = (1.0 - progress) * 0.8;
            let color_shift = progress * 0.3;
            sprite.color = Color::srgba(1.0 - color_shift, 0.8, 0.3 + color_shift, alpha);
            
            // Damage enemies within radius
            for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health) in enemy_health_query.iter_mut() {
                let distance = spore_transform.translation.distance(enemy_transform.translation);
                if distance <= spore.current_radius {
                    enemy_health.0 -= spore.damage;
                    
                    // Spawn organic destruction particles
                    if let Some(assets) = &assets {
                        for i in 0..3 {
                            let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
                            let offset = Vec2::from_angle(angle) * 15.0;
                            
                            commands.spawn((
                                Sprite {
                                    image: assets.particle_texture.clone(),
                                    color: Color::srgb(1.0, 0.6, 0.2),
                                    custom_size: Some(Vec2::splat(4.0)),
                                    ..default()
                                },
                                Transform::from_translation(enemy_transform.translation + offset.extend(0.0)),
                                Particle {
                                    velocity: offset * 3.0,
                                    lifetime: 0.0,
                                    max_lifetime: 1.2,
                                    size: 4.0,
                                    fade_rate: 1.0,
                                    bioluminescent: true,
                                    drift_pattern: DriftPattern::Floating,
                                },
                            ));
                        }
                    }
                    
                    if enemy_health.0 <= 0 {
                        explosion_events.write(SpawnExplosion {
                            position: enemy_transform.translation,
                            intensity: 1.8,
                            enemy_type: None,
                        });
                        commands.entity(enemy_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                    }
                }
            }
        }
    }

    // 4. UPDATE TOXIN CLOUDS
    {
        let mut toxin_clouds = weapon_queries.p3();
        for (cloud_entity, mut cloud, mut cloud_transform, mut sprite) in toxin_clouds.iter_mut() {
            cloud.timer += time.delta_secs();
            
            if cloud.timer >= cloud.max_duration {
                commands.entity(cloud_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                continue;
            }
            
            let progress = cloud.timer / cloud.max_duration;
            cloud.intensity = 1.0 - progress * 0.3;
            
            // Organic pulsing effect
            let pulse = (time.elapsed_secs() * 3.0).sin() * 0.1 + 0.9;
            cloud_transform.scale = Vec3::splat(pulse);
            sprite.color.set_alpha(cloud.intensity * 0.6);
            
            // Damage enemies in cloud
            for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health) in enemy_health_query.iter_mut() {
                let distance = cloud_transform.translation.distance(enemy_transform.translation);
                if distance <= cloud.radius {
                    let damage = (cloud.damage_per_second as f32 * time.delta_secs()) as i32;
                    enemy_health.0 -= damage;
                    
                    if enemy_health.0 <= 0 {
                        explosion_events.write(SpawnExplosion {
                            position: enemy_transform.translation,
                            intensity: 0.8,
                            enemy_type: None,
                        });
                        commands.entity(enemy_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                    }
                }
            }
        }
    }

    // 5. UPDATE ELECTRIC ARCS
    {
        let mut electric_arcs = weapon_queries.p4();
        for (arc_entity, mut arc) in electric_arcs.iter_mut() {
            arc.timer += time.delta_secs();
            
            if arc.timer >= arc.max_duration {
                commands.entity(arc_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                continue;
            }
            
            // Apply damage to target
            if let Some(target_entity) = arc.target_entity {
                if let Ok((_, target_transform, _, mut target_health)) = enemy_health_query.get_mut(target_entity) {
                    target_health.0 -= arc.damage;
                    
                    // Spawn arc visual effect
                    if let Some(assets) = &assets {
                        let segments = 8;
                        for i in 0..segments {
                            let t = i as f32 / segments as f32;
                            let pos = arc.start_pos.lerp(arc.end_pos, t);
                            let jitter = Vec2::new(
                                (time.elapsed_secs() * 20.0 + i as f32).sin() * 5.0,
                                (time.elapsed_secs() * 25.0 + i as f32).cos() * 5.0,
                            );
                            
                            commands.spawn((
                                Sprite {
                                    image: assets.particle_texture.clone(),
                                    color: Color::srgb(0.8, 0.9, 1.0),
                                    custom_size: Some(Vec2::splat(3.0)),
                                    ..default()
                                },
                                Transform::from_translation((pos + jitter).extend(0.0)),
                                Particle {
                                    velocity: Vec2::ZERO,
                                    lifetime: 0.0,
                                    max_lifetime: 0.1,
                                    size: 3.0,
                                    fade_rate: 10.0,
                                    bioluminescent: true,
                                    drift_pattern: DriftPattern::Pulsing,
                                },
                            ));
                        }
                    }
                    
                    if target_health.0 <= 0 {
                        explosion_events.write(SpawnExplosion {
                            position: target_transform.translation,
                            intensity: 1.2,
                            enemy_type: None,
                        });
                        commands.entity(target_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                    }
                }
            }
        }
    }
}


// Helper function to find nearest enemy
fn find_nearest_enemy(
    enemy_query: &Query<(Entity, &Transform), (With<Enemy>, Without<MissileProjectile>, Without<SporeWave>, Without<LaserBeam>, Without<ToxinCloudEffect>)>,
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

// ===== PREVIOUS FILE =====

pub fn enhanced_shooting_system(
    mut commands: Commands,
    input_manager: Res<InputManager>,
    mut player_query: Query<(
        &Transform, 
        &mut EvolutionSystem, 
        &CellularUpgrades,
        Option<&mut WingCannon>,
        Option<&mut MissileSystem>
    ), With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Enemy), (Without<AutoMissile>, Without<Player>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut main_cannon_timer: Local<f32>,
) {
    let Some(assets) = assets else { return };
    
    *main_cannon_timer -= time.delta_secs();
    
    if let Ok((player_transform, mut evolution_system, upgrades, wing_cannon, missile_system)) = player_query.single_mut() {
        let shooting = input_manager.pressed(InputAction::Shoot);
        
        // ===== MAIN CANNON (Enhanced) =====
        if shooting && *main_cannon_timer <= 0.0 {
            spawn_enhanced_main_cannon_projectiles(
                &mut commands,
                &assets,
                player_transform,
                &evolution_system,
                upgrades
            );
            
            let base_fire_rate = evolution_system.primary_evolution.get_fire_rate();
            *main_cannon_timer = base_fire_rate / upgrades.metabolic_rate;
        }

        // ===== WING CANNONS =====
        if let Some(mut wing_cannon) = wing_cannon {
            wing_cannon.fire_timer -= time.delta_secs();
            
            if shooting && wing_cannon.fire_timer <= 0.0 {
                spawn_wing_cannon_projectiles(
                    &mut commands,
                    &assets,
                    player_transform,
                    &wing_cannon,
                    upgrades
                );
                wing_cannon.fire_timer = wing_cannon.fire_rate / upgrades.metabolic_rate;
            }
        }

        // ===== MISSILE SYSTEM =====
        if let Some(mut missile_system) = missile_system {
            missile_system.fire_timer -= time.delta_secs();
            
            if missile_system.fire_timer <= 0.0 {
                let target = find_strongest_enemy_in_range(
                    &enemy_query,
                    player_transform.translation,
                    missile_system.homing_range
                );
                
                if target.is_some() {
                    spawn_auto_missiles(
                        &mut commands,
                        &assets,
                        player_transform,
                        &missile_system,
                        target
                    );
                    missile_system.fire_timer = missile_system.fire_rate;
                }
            }
        }

        // ===== EMERGENCY SPORE =====
        if input_manager.just_pressed(InputAction::EmergencySpore) && evolution_system.emergency_spores > 0 {
            spawn_emergency_spore(&mut commands, &assets, player_transform.translation);
            evolution_system.emergency_spores -= 1;
        }
    }
}

fn spawn_enhanced_main_cannon_projectiles(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    evolution_system: &EvolutionSystem,
    upgrades: &CellularUpgrades,
) {
    let damage_level = upgrades.damage_amplification;
    let base_damage = evolution_system.primary_evolution.get_base_damage();
    let final_damage = (base_damage as f32 * damage_level) as i32;
    
    // Determine number of projectiles based on damage level
    let projectile_count = match damage_level as u32 {
        1..=1 => 1,
        2..=2 => 2, // Add second projectile
        3..=3 => 3, // Add third projectile
        4..=4 => 4, // Four projectiles
        _ => 5,     // Maximum five projectiles
    } as f32;
    
    let base_size = 8.0 + (damage_level - 1.0) * 2.0; // Size increases with level
    
    // Spawn multiple projectiles in a tight spread
    for i in 0..(projectile_count as u32) {
        let offset_x = if projectile_count == 1.0 {
            0.0
        } else {
            (i as f32 - (projectile_count - 1.0) / 2.0) * 12.0
        };
        
        commands.spawn((
            Sprite {
                image: assets.projectile_texture.clone(),
                color: Color::srgb(0.4, 0.9, 0.7),
                custom_size: Some(Vec2::splat(base_size)),
                ..default()
            },
            Transform::from_translation(
                player_transform.translation + Vec3::new(offset_x, 30.0, 0.0)
            ),
            Projectile {
                velocity: Vec2::new(0.0, 850.0),
                damage: final_damage,
                friendly: true,
                organic_trail: true,
            },
            Collider { radius: base_size / 2.0 },
            BioluminescentParticle {
                base_color: Color::srgb(0.4, 0.9, 0.7),
                pulse_frequency: 3.0,
                pulse_intensity: 0.6,
                organic_motion: OrganicMotion {
                    undulation_speed: 2.0,
                    response_to_current: 0.3,
                },
            },
        ));
    }
}

fn spawn_wing_cannon_projectiles(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    wing_cannon: &WingCannon,
    upgrades: &CellularUpgrades,
) {
    let enhanced_damage = (wing_cannon.damage as f32 * upgrades.damage_amplification) as i32;
    
    // Color progression: Yellow (level 1) to Green (level 5)
    let color = match wing_cannon.level {
        1 => Color::srgb(1.0, 0.9, 0.2),      // Yellow
        2 => Color::srgb(0.9, 0.9, 0.3),      // Yellow-Green
        3 => Color::srgb(0.7, 0.9, 0.4),      // Light Green
        4 => Color::srgb(0.5, 0.9, 0.5),      // Green
        _ => Color::srgb(0.3, 1.0, 0.6),      // Bright Green
    };
    
    // Spawn left wing projectile
    commands.spawn((
        Sprite {
            image: assets.projectile_texture.clone(),
            color,
            custom_size: Some(Vec2::splat(wing_cannon.projectile_size)),
            ..default()
        },
        Transform::from_translation(
            player_transform.translation + Vec3::new(-WING_CANNON_OFFSET, WING_CANNON_Y_OFFSET, 0.0)
        ),
        Projectile {
            velocity: Vec2::new(0.0, 750.0),
            damage: enhanced_damage,
            friendly: true,
            organic_trail: false,
        },
        WingCannonProjectile {
            pierce_count: 0,
            max_pierce: wing_cannon.level + 1,
            damage_falloff: 0.8, // 20% damage reduction per pierce
        },
        Collider { radius: wing_cannon.projectile_size / 2.0 },
    ));
    
    // Spawn right wing projectile
    commands.spawn((
        Sprite {
            image: assets.projectile_texture.clone(),
            color,
            custom_size: Some(Vec2::splat(wing_cannon.projectile_size)),
            ..default()
        },
        Transform::from_translation(
            player_transform.translation + Vec3::new(WING_CANNON_OFFSET, WING_CANNON_Y_OFFSET, 0.0)
        ),
        Projectile {
            velocity: Vec2::new(0.0, 750.0),
            damage: enhanced_damage,
            friendly: true,
            organic_trail: false,
        },
        WingCannonProjectile {
            pierce_count: 0,
            max_pierce: wing_cannon.level + 1,
            damage_falloff: 0.8,
        },
        Collider { radius: wing_cannon.projectile_size / 2.0 },
    ));
}

fn spawn_auto_missiles(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    missile_system: &MissileSystem,
    target: Option<Entity>,
) {
    let missile_count = if missile_system.dual_launch { 2 } else { 1 };
    
    for i in 0..missile_count {
        let offset_x = if missile_count == 1 { 0.0 } else { (i as f32 - 0.5) * 30.0 };
        
        commands.spawn((
            Sprite {
                image: assets.projectile_texture.clone(),
                color: Color::srgb(0.9, 0.5, 0.3), // Orange missile color
                custom_size: Some(Vec2::splat(10.0)),
                ..default()
            },
            Transform::from_translation(
                player_transform.translation + Vec3::new(offset_x, MISSILE_Y_OFFSET, 0.0)
            ),
            Projectile {
                velocity: Vec2::new(0.0, missile_system.missile_speed),
                damage: missile_system.damage,
                friendly: true,
                organic_trail: true,
            },
            AutoMissile {
                target,
                homing_strength: 2.0,
                speed: missile_system.missile_speed,
                damage: missile_system.damage,
                retarget_timer: 0.0,
            },
            ExplosiveProjectile {
                blast_radius: 40.0,
                blast_damage: missile_system.damage / 2,
                organic_explosion: true,
            },
            Collider { radius: 8.0 },
            BioluminescentParticle {
                base_color: Color::srgb(0.9, 0.5, 0.3),
                pulse_frequency: 4.0,
                pulse_intensity: 0.7,
                organic_motion: OrganicMotion {
                    undulation_speed: 3.0,
                    response_to_current: 0.2,
                },
            },
        ));
    }
}

fn find_strongest_enemy_in_range(
    enemy_query: &Query<(Entity, &Transform, &Enemy), (Without<AutoMissile>, Without<Player>)>,
    player_pos: Vec3,
    range: f32,
) -> Option<Entity> {
    enemy_query
        .iter()
        .filter(|(_, transform, _)| {
            player_pos.distance(transform.translation) <= range
        })
        .max_by(|(_, _, a), (_, _, b)| {
            // Prioritize by enemy type strength (boss > heavy > normal)
            let strength_a = match a.enemy_type {
                EnemyType::InfectedMacrophage => 100,
                EnemyType::ReproductiveVesicle => 80,
                EnemyType::ParasiticProtozoa => 60,
                EnemyType::BiofilmColony => 50,
                EnemyType::SwarmCell => 40,
                EnemyType::AggressiveBacteria => 30,
                EnemyType::SuicidalSpore => 20,
                EnemyType::ViralParticle => 10,
                EnemyType::Offspring => 5,
            };
            let strength_b = match b.enemy_type {
                EnemyType::InfectedMacrophage => 100,
                EnemyType::ReproductiveVesicle => 80,
                EnemyType::ParasiticProtozoa => 60,
                EnemyType::BiofilmColony => 50,
                EnemyType::SwarmCell => 40,
                EnemyType::AggressiveBacteria => 30,
                EnemyType::SuicidalSpore => 20,
                EnemyType::ViralParticle => 10,
                EnemyType::Offspring => 5,
            };
            strength_a.cmp(&strength_b)
        })
        .map(|(entity, _, _)| entity)
}

pub fn wing_cannon_collision_system(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &Transform, &Collider, &mut Projectile, &mut WingCannonProjectile)>,
    mut enemy_query: Query<(Entity, &Transform, &Collider, &mut Health), (With<Enemy>, Without<WingCannonProjectile>)>,
    mut explosion_events: EventWriter<SpawnExplosion>,
) {
    for (proj_entity, proj_transform, proj_collider, mut projectile, mut wing_cannon) in projectile_query.iter_mut() {
        for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health) in enemy_query.iter_mut() {
            let distance = proj_transform.translation.distance(enemy_transform.translation);
            
            if distance < proj_collider.radius + enemy_collider.radius {
                // Apply damage with falloff
                let actual_damage = (projectile.damage as f32 * wing_cannon.damage_falloff.powi(wing_cannon.pierce_count as i32)) as i32;
                enemy_health.0 -= actual_damage;
                
                // Spawn hit effect
                explosion_events.write(SpawnExplosion {
                    position: enemy_transform.translation,
                    intensity: 0.6,
                    enemy_type: None,
                });
                
                wing_cannon.pierce_count += 1;
                
                // Check if projectile should be destroyed
                if wing_cannon.pierce_count >= wing_cannon.max_pierce {
                    commands.entity(proj_entity)
                        .insert(AlreadyDespawned)
                        .despawn();
                    break;
                } else {
                    // Reduce damage for next hit
                    projectile.damage = actual_damage;
                }
                
                // Check if enemy died
                if enemy_health.0 <= 0 {
                    explosion_events.write(SpawnExplosion {
                        position: enemy_transform.translation,
                        intensity: 1.0,
                        enemy_type: None,
                    });
                    commands.entity(enemy_entity)
                        .insert(AlreadyDespawned)
                        .despawn();
                }
                
                break; // Only hit one enemy per frame per projectile
            }
        }
    }
}

pub fn auto_missile_system(
    mut missile_query: Query<(Entity, &mut Transform, &mut Projectile, &mut AutoMissile)>,
    enemy_query: Query<(Entity, &Transform, &Enemy), (Without<AutoMissile>, Without<Player>)>,
    time: Res<Time>,
) {
    for (missile_entity, mut missile_transform, mut projectile, mut auto_missile) in missile_query.iter_mut() {
        auto_missile.retarget_timer += time.delta_secs();
        
        // Retarget every 0.5 seconds or if target is lost
        if auto_missile.retarget_timer > 0.5 {
            auto_missile.retarget_timer = 0.0;
            
            if let Some(target_entity) = auto_missile.target {
                // Check if current target still exists
                if enemy_query.get(target_entity).is_err() {
                    auto_missile.target = find_strongest_enemy_in_range(
                        &enemy_query,
                        missile_transform.translation,
                        300.0 // Retarget range
                    );
                }
            } else {
                auto_missile.target = find_strongest_enemy_in_range(
                    &enemy_query,
                    missile_transform.translation,
                    300.0
                );
            }
        }
        
        // Homing behavior
        if let Some(target_entity) = auto_missile.target {
            if let Ok((_, target_transform, _)) = enemy_query.get(target_entity) {
                let direction_to_target = (target_transform.translation - missile_transform.translation).normalize_or_zero();
                let current_direction = projectile.velocity.normalize_or_zero();
                
                // Smooth homing
                let homing_rate = auto_missile.homing_strength * time.delta_secs();
                let new_direction = (current_direction + direction_to_target.truncate() * homing_rate).normalize_or_zero();
                projectile.velocity = new_direction * auto_missile.speed;
                
                // Update rotation
                let angle = new_direction.y.atan2(new_direction.x) - std::f32::consts::FRAC_PI_2;
                missile_transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

// Spawn cytoplasmic spray projectiles
fn spawn_cytoplasmic_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    adaptations: &CellularAdaptations,
) {
    let damage = (base_damage as f32 * adaptations.membrane_permeability) as i32;
    
    let mut entity_commands = commands.spawn((
        Sprite {
            image: assets.projectile_texture.clone(),
            color: Color::srgb(0.4, 0.9, 0.7), // Organic cyan-green
            ..default()
        },
        Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0)),
        Projectile {
            velocity: Vec2::new(0.0, 850.0),
            damage,
            friendly: true,
            organic_trail: true,
        },
        Collider { radius: 5.0 },
    ));
    
    // Add special properties based on adaptations
    if adaptations.extremophile_traits {
        entity_commands.insert(ArmorPiercing {
            pierce_count: 0,
            max_pierce: 2,
            enzyme_based: true,
        });
    }
    
    if adaptations.biofilm_formation {
        entity_commands.insert(ExplosiveProjectile {
            blast_radius: 30.0,
            blast_damage: damage / 3,
            organic_explosion: true,
        });
    }
}

// Spawn pseudopod network (spread shot)
fn spawn_pseudopod_network(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    tendril_count: u32,
    spread_angle: f32,
    adaptations: &CellularAdaptations,
) {
    let damage = (base_damage as f32 * adaptations.membrane_permeability) as i32;
    let angle_step = spread_angle / (tendril_count - 1) as f32;
    let start_angle = -spread_angle / 2.0;
    
    for i in 0..tendril_count {
        let angle = start_angle + angle_step * i as f32;
        let direction = Vec2::new(angle.sin(), angle.cos());
        
        commands.spawn((
            Sprite {
                image: assets.projectile_texture.clone(),
                color: Color::srgb(0.8, 0.6, 1.0), // Purple pseudopod
                custom_size: Some(Vec2::splat(4.0)),
                ..default()
            },
            Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            Projectile {
                velocity: direction * 720.0,
                damage,
                friendly: true,
                organic_trail: true,
            },
            Collider { radius: 4.0 },
            BioluminescentParticle {
                base_color: Color::srgb(0.8, 0.6, 1.0),
                pulse_frequency: 4.0,
                pulse_intensity: 0.5,
                organic_motion: OrganicMotion {
                    undulation_speed: 2.0,
                    response_to_current: 0.3,
                },
            },
        ));
    }
}

// Spawn bioluminescent beam
fn spawn_bioluminescent_beam(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    duration: f32,
    width: f32,
    adaptations: &CellularAdaptations,
) {
    let damage = (base_damage as f32 * adaptations.membrane_permeability) as i32;
    
    commands.spawn((
        Sprite {
            image: assets.projectile_texture.clone(),
            color: Color::srgb(0.3, 1.0, 0.8), // Bright bioluminescent
            custom_size: Some(Vec2::new(width, 900.0)),
            ..default()
        },
        Transform::from_translation(player_transform.translation + Vec3::new(0.0, 450.0, 0.0)),
        LaserBeam {
            timer: 0.0,
            max_duration: duration * adaptations.metabolic_efficiency,
            damage_per_second: damage * 12,
            width,
            length: 900.0,
            bioluminescent: true,
        },
        Collider { radius: width / 2.0 },
        BioluminescentParticle {
            base_color: Color::srgb(0.3, 1.0, 0.8),
            pulse_frequency: 8.0,
            pulse_intensity: 0.8,
            organic_motion: OrganicMotion {
                undulation_speed: 0.5,
                response_to_current: 0.0,
            },
        },
    ));
}

// Spawn symbiotic hunter (homing missile)
fn spawn_symbiotic_hunter(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    target: Option<Entity>,
    homing_strength: f32,
    blast_radius: f32,
    adaptations: &CellularAdaptations,
) {
    let damage = (base_damage as f32 * adaptations.membrane_permeability) as i32;
    
    commands.spawn((
        Sprite {
            image: assets.projectile_texture.clone(),
            color: Color::srgb(1.0, 0.7, 0.3), // Warm symbiotic orange
            custom_size: Some(Vec2::splat(8.0)),
            ..default()
        },
        Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0)),
        Projectile {
            velocity: Vec2::new(0.0, 450.0),
            damage,
            friendly: true,
            organic_trail: true,
        },
        MissileProjectile {
            target,
            homing_strength: homing_strength * adaptations.chemoreceptor_sensitivity,
            blast_radius,
            seek_timer: 0.0,
            symbiotic: true,
        },
        ExplosiveProjectile {
            blast_radius,
            blast_damage: damage,
            organic_explosion: true,
        },
        Collider { radius: 6.0 },
        BioluminescentParticle {
            base_color: Color::srgb(1.0, 0.7, 0.3),
            pulse_frequency: 3.0,
            pulse_intensity: 0.6,
            organic_motion: OrganicMotion {
                undulation_speed: 1.5,
                response_to_current: 0.4,
            },
        },
    ));
}

// Spawn enzyme burst
fn spawn_enzyme_burst(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    base_damage: i32,
    acid_damage: f32,
    adaptations: &CellularAdaptations,
) {
    let damage = (base_damage as f32 * adaptations.membrane_permeability) as i32;
    
    // Spawn multiple enzyme projectiles in a burst
    for i in 0..5 {
        let angle = (i as f32 - 2.0) * 0.2;
        let direction = Vec2::new(angle.sin(), angle.cos());
        
        commands.spawn((
            Sprite {
                image: assets.projectile_texture.clone(),
                color: Color::srgb(0.9, 1.0, 0.4), // Acidic yellow-green
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0)),
            Projectile {
                velocity: direction * 800.0,
                damage,
                friendly: true,
                organic_trail: true,
            },
            Collider { radius: 5.0 },
            ArmorPiercing {
                pierce_count: 0,
                max_pierce: 3,
                enzyme_based: true,
            },
        ));
    }
}

// Spawn toxin cloud
fn spawn_toxin_cloud(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    damage_per_second: i32,
    cloud_radius: f32,
    duration: f32,
    adaptations: &CellularAdaptations,
) {
    let enhanced_damage = (damage_per_second as f32 * adaptations.membrane_permeability) as i32;
    
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgba(0.7, 0.9, 0.3, 0.6), // Translucent toxic green
            custom_size: Some(Vec2::splat(cloud_radius * 2.0)),
            ..default()
        },
        Transform::from_translation(player_transform.translation + Vec3::new(0.0, 50.0, 0.0)),
        ToxinCloudEffect {
            timer: 0.0,
            max_duration: duration,
            damage_per_second: enhanced_damage,
            radius: cloud_radius,
            intensity: 1.0,
        },
        Collider { radius: cloud_radius },
    ));
}

// Spawn electric discharge
fn spawn_electric_discharge(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    enemy_query: &Query<(Entity, &Transform), (With<Enemy>, Without<MissileProjectile>, Without<SporeWave>, Without<LaserBeam>, Without<ToxinCloudEffect>)>,
    base_damage: i32,
    chain_count: u32,
    range: f32,
    adaptations: &CellularAdaptations,
) {
    let damage = (base_damage as f32 * adaptations.membrane_permeability) as i32;
    
    // Find nearby enemies for electric chain
    let mut targets = Vec::new();
    for (enemy_entity, enemy_transform) in enemy_query.iter() {
        let distance = player_transform.translation.distance(enemy_transform.translation);
        if distance <= range && targets.len() < chain_count as usize {
            targets.push((enemy_entity, enemy_transform.translation));
        }
    }
    
    // Create electric discharge effects
    for (i, (target_entity, target_pos)) in targets.iter().enumerate() {
        commands.spawn((
            ElectricArc {
                start_pos: player_transform.translation.truncate(),
                end_pos: target_pos.truncate(),
                damage,
                chain_index: i as u32,
                timer: 0.0,
                max_duration: 0.3,
                target_entity: Some(*target_entity),
            },
        ));
    }
}

// Spawn clone projectile for binary fission
fn spawn_clone_projectile(
    commands: &mut Commands,
    assets: &GameAssets,
    player_transform: &Transform,
    evolution: &EvolutionType,
    adaptations: &CellularAdaptations,
) {
    // Create a slightly offset clone projectile
    let offset = Vec2::new(15.0, 0.0);
    let clone_transform = Transform::from_translation(player_transform.translation + offset.extend(0.0));
    
    match evolution {
        EvolutionType::CytoplasmicSpray { damage, .. } => {
            spawn_cytoplasmic_projectile(commands, assets, &clone_transform, *damage, adaptations);
        }
        _ => {
            // Default clone projectile for other weapon types
            spawn_cytoplasmic_projectile(commands, assets, &clone_transform, 8, adaptations);
        }
    }
}

// Spawn emergency spore (replaces smart bomb)
fn spawn_emergency_spore(commands: &mut Commands, assets: &GameAssets, position: Vec3) {
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgb(1.0, 0.8, 0.3),
            ..default()
        },
        Transform::from_translation(position),
        SporeWave {
            timer: 0.0,
            max_time: 2.5,
            current_radius: 0.0,
            max_radius: 600.0,
            damage: 150, // Increased damage for new system
        },
        BioluminescentParticle {
            base_color: Color::srgb(1.0, 0.8, 0.3),
            pulse_frequency: 6.0,
            pulse_intensity: 1.0,
            organic_motion: OrganicMotion {
                undulation_speed: 3.0,
                response_to_current: 0.0,
            },
        },
    ));
}

