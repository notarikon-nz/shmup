use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;


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

// Enhanced biological shooting system
pub fn enhanced_shooting_system(
    mut commands: Commands,
    input_state: Res<InputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut EvolutionSystem), With<Player>>,
    enemy_query: Query<(Entity, &Transform), (With<Enemy>, Without<MissileProjectile>, Without<SporeWave>, Without<LaserBeam>, Without<ToxinCloudEffect>)>,
    assets: Option<Res<GameAssets>>,
    mitochondria_query: Query<&MitochondriaOvercharge>,
    chemotaxis_query: Query<&ChemotaxisActive>,
    binary_fission_query: Query<&BinaryFissionActive>,
    time: Res<Time>,
    mut shoot_timer: Local<f32>,
    mut beam_charging: Local<bool>,
    mut beam_charge_timer: Local<f32>,
    mut toxin_cloud_timer: Local<f32>,
) {
    if let Some(assets) = assets {
        *shoot_timer -= time.delta_secs();
        *toxin_cloud_timer -= time.delta_secs();
        
        if let Ok((player_transform, mut evolution_system)) = player_query.single_mut() {
            let evolution = evolution_system.primary_evolution.clone();
            let adaptations = &evolution_system.cellular_adaptations;
            
            // Get rate multiplier from mitochondria overcharge
            let rate_multiplier = mitochondria_query.iter().next()
                .map(|mito| mito.rate_multiplier)
                .unwrap_or(1.0);
            
            // Get homing enhancement from chemotaxis
            let homing_bonus = chemotaxis_query.iter().next()
                .map(|chemo| chemo.homing_strength)
                .unwrap_or(0.0);
            
            match evolution {
                EvolutionType::CytoplasmicSpray { damage, fire_rate } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        spawn_cytoplasmic_projectile(&mut commands, &assets, player_transform, damage, adaptations);
                        *shoot_timer = fire_rate / (adaptations.metabolic_efficiency * rate_multiplier);
                    }
                }
                
                EvolutionType::PseudopodNetwork { damage, fire_rate, tendril_count, spread_angle } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        spawn_pseudopod_network(&mut commands, &assets, player_transform, damage, tendril_count, spread_angle, adaptations);
                        *shoot_timer = fire_rate / (adaptations.metabolic_efficiency * rate_multiplier);
                    }
                }
                
                EvolutionType::BioluminescentBeam { damage, charge_time, duration, width } => {
                    if input_state.shooting {
                        if !*beam_charging {
                            *beam_charging = true;
                            *beam_charge_timer = 0.0;
                        }
                        *beam_charge_timer += time.delta_secs();
                        
                        if *beam_charge_timer >= charge_time {
                            spawn_bioluminescent_beam(&mut commands, &assets, player_transform, damage, duration, width, adaptations);
                            *beam_charging = false;
                            *beam_charge_timer = 0.0;
                        }
                    } else {
                        *beam_charging = false;
                        *beam_charge_timer = 0.0;
                    }
                }
                
                EvolutionType::SymbioticHunters { damage, fire_rate, homing_strength, blast_radius } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        let target = find_nearest_enemy(&enemy_query, player_transform.translation);
                        let enhanced_homing = homing_strength + homing_bonus;
                        spawn_symbiotic_hunter(&mut commands, &assets, player_transform, damage, target, enhanced_homing, blast_radius, adaptations);
                        *shoot_timer = fire_rate / (adaptations.metabolic_efficiency * rate_multiplier);
                    }
                }
                
                EvolutionType::EnzymeBurst { damage, fire_rate, acid_damage } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        spawn_enzyme_burst(&mut commands, &assets, player_transform, damage, acid_damage, adaptations);
                        *shoot_timer = fire_rate / (adaptations.metabolic_efficiency * rate_multiplier);
                    }
                }
                
                EvolutionType::ToxinCloud { damage_per_second, cloud_radius, duration } => {
                    if input_state.shooting && *toxin_cloud_timer <= 0.0 {
                        spawn_toxin_cloud(&mut commands, &assets, player_transform, damage_per_second, cloud_radius, duration, adaptations);
                        *toxin_cloud_timer = 3.0; // Cooldown for toxin clouds
                    }
                }
                
                EvolutionType::ElectricDischarge { damage, chain_count, range } => {
                    if input_state.shooting && *shoot_timer <= 0.0 {
                        spawn_electric_discharge(&mut commands, &assets, player_transform, &enemy_query, damage, chain_count, range, adaptations);
                        *shoot_timer = 1.5 / adaptations.metabolic_efficiency;
                    }
                }
            }
            
            // Binary fission - spawn clone projectiles
            if let Some(binary_fission) = binary_fission_query.iter().next() {
                if input_state.shooting && *shoot_timer <= 0.0 {
                    // Spawn additional projectile for binary fission
                    spawn_clone_projectile(&mut commands, &assets, player_transform, &evolution, adaptations);
                }
            }
            
            // Emergency spore activation (replaces smart bomb)
            if keyboard.just_pressed(KeyCode::Space) && evolution_system.emergency_spores > 0 {
                spawn_emergency_spore(&mut commands, &assets, player_transform.translation);
                evolution_system.emergency_spores -= 1;
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
fn spawn_emergency_spore(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
) {
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgb(1.0, 0.8, 0.3), // Warm spore color
            ..default()
        },
        Transform::from_translation(position),
        SporeWave {
            timer: 0.0,
            max_time: 2.5,
            current_radius: 0.0,
            max_radius: 600.0,
            damage: 120,
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



