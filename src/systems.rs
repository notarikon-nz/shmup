use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::achievements::*;
use crate::enemy_types::*;
use crate::physics::*;
use crate::wave_systems::*;
use crate::despawn::{SafeDespawn};

// ===== PERFORMANCE CONSTANTS =====
const MAX_PARTICLES: usize = 200;
const CLEANUP_PARTICLES_TO: usize = 150;
const CLEANUP_INTERVAL: f32 = 2.0;
const MAX_AUDIO_ENTITIES: usize = 10;
const OFFSCREEN_BOUNDS: f32 = 600.0;
const COLLISION_GRID_SIZE: f32 = 64.0;
const SPAWN_RATE_MIN: f32 = 0.3;
const SPAWN_RATE_DECAY: f32 = 0.02;
const ENEMY_SHOOT_INTERVAL: f32 = 1.5;

// ===== WAVE CONSTANTS =====
const WAVE_1_DURATION: f32 = 20.0;
const WAVE_2_DURATION: f32 = 40.0;
const WAVE_3_DURATION: f32 = 60.0;

// ===== OPTIMIZED COLLISION HELPERS =====
#[inline(always)]
fn check_collision_fast(pos1: Vec3, radius1: f32, pos2: Vec3, radius2: f32) -> bool {
    let dx = pos1.x - pos2.x;
    let dy = pos1.y - pos2.y;
    let distance_sq = dx * dx + dy * dy;
    let combined_radius = radius1 + radius2;
    distance_sq < combined_radius * combined_radius
}

#[inline(always)]
fn calculate_crit_hit(damage: i32, crit_stats: &CriticalHitStats, seed: f32) -> (i32, bool) {
    let is_crit = seed.sin().abs() < crit_stats.chance;
    if is_crit {
        ((damage as f32 * crit_stats.damage_multiplier) as i32, true)
    } else {
        (damage, false)
    }
}

fn spawn_damage_text_fast(commands: &mut Commands, position: Vec3, damage: i32, is_crit: bool, fonts: &GameFonts) {
    let (color, size) = if is_crit { 
        (Color::srgb(1.0, 1.0, 0.3), 16.0) 
    } else { 
        (Color::WHITE, 12.0) 
    };
    
    commands.spawn((
        Text2d::new(format!("{}", damage)),
        TextFont { font: fonts.default_font.clone(), font_size: size, ..default() },
        TextColor(color),
        Transform::from_translation(position + Vec3::new(0.0, 25.0, 1.0)),
        DamageText { timer: 1.5, velocity: Vec2::new(0.0, 80.0) },
    ));
}

// ===== OPTIMIZED WAVE SYSTEM =====
#[inline]
fn get_enemy_config_fast(wave_timer: f32, enemies_spawned: u32) -> (EnemyAI, EnemyType) {
    match wave_timer {
        t if t < WAVE_1_DURATION => (
            EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }, 
            EnemyType::ViralParticle
        ),
        t if t < WAVE_2_DURATION => (
            EnemyAI::Sine { amplitude: 100.0, frequency: 2.0, phase: 0.0 }, 
            EnemyType::AggressiveBacteria
        ),
        t if t < WAVE_3_DURATION => {
            if enemies_spawned & 1 == 0 {
                (EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }, EnemyType::ParasiticProtozoa)
            } else {
                (EnemyAI::Chemotaxis { 
                    target_chemical: ChemicalType::PlayerPheromones,
                    sensitivity: 1.5,
                    current_direction: Vec2::new(0.0, -1.0),
                }, EnemyType::AggressiveBacteria)
            }
        },
        _ => {
            if enemies_spawned % 10 == 0 {
                (EnemyAI::MiniBoss { pattern: 0, timer: 0.0 }, EnemyType::InfectedMacrophage)
            } else {
                (EnemyAI::FluidFlow { 
                    flow_sensitivity: 2.0,
                    base_direction: Vec2::new(0.0, -1.0),
                }, EnemyType::SwarmCell)
            }
        }
    }
}

// ===== CORE SYSTEMS (unchanged but optimized) =====

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { viewport_height: 720.0 },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

pub fn init_particle_pool(mut commands: Commands) {
    commands.insert_resource(ParticlePool { entities: Vec::with_capacity(2000), index: 0 });
    commands.insert_resource(ShootingState { rate_multiplier: 1.0, base_rate: 0.1 });
}

// ===== OPTIMIZED MOVEMENT SYSTEMS =====

pub fn move_projectiles(
    mut projectile_query: Query<(&mut Transform, &Projectile)>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    projectile_query.par_iter_mut().for_each(|(mut transform, projectile)| {
        // Move projectile
        let velocity_dt = projectile.velocity * dt;
        transform.translation.x += velocity_dt.x;
        transform.translation.y += velocity_dt.y;
        
        // Apply fluid effects only to friendly projectiles
        if projectile.friendly {
            let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
            let current = sample_current(&fluid_environment, grid_pos);
            let current_dt = current * 0.05 * dt;
            transform.translation.x += current_dt.x;
            transform.translation.y += current_dt.y;
        }
    });
}

pub fn update_parallax(
    mut parallax_query: Query<(&mut Transform, &ParallaxLayer)>,
    time: Res<Time>,
) {
    let movement = 100.0 * time.delta_secs();
    
    parallax_query.par_iter_mut().for_each(|(mut transform, layer)| {
        transform.translation.y -= layer.speed * movement;
        if transform.translation.y < -400.0 {
            transform.translation.y = 400.0;
        }
    });
}

// ===== OPTIMIZED CLEANUP SYSTEM =====

pub fn cleanup_offscreen(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (
        Without<Player>, Without<ParallaxLayer>, Without<HealthBarFill>, 
        Without<ScoreText>, Without<HighScoreText>, Without<HealthBar>,
        Without<LivesText>, Without<MultiplierText>, Without<CellWallVisual>,
        Without<AlreadyDespawned>,
    )>,
) {
    let bounds_sq = OFFSCREEN_BOUNDS * OFFSCREEN_BOUNDS;
    
    for (entity, transform) in query.iter() {
        let pos = transform.translation;
        let distance_sq = pos.x * pos.x + pos.y * pos.y;
        
        if distance_sq > bounds_sq {
            commands.entity(entity).try_insert(AlreadyDespawned).despawn();
        }
    }
}

// ===== OPTIMIZED ENEMY SYSTEMS =====

pub fn spawn_enemies(
    _commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    time: Res<Time>,
) {
    enemy_spawner.spawn_timer -= time.delta_secs();
    enemy_spawner.wave_timer += time.delta_secs();
    
    if enemy_spawner.spawn_timer <= 0.0 {
        // Use bit manipulation for faster position selection
        let spawn_x = match enemy_spawner.enemies_spawned & 0b11 {
            0 => -400.0,
            1 => -200.0, 
            2 => 0.0,
            3 => 200.0,
            _ => 400.0,
        };
        
        let (ai_type, enemy_type) = get_enemy_config_fast(enemy_spawner.wave_timer, enemy_spawner.enemies_spawned);
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(spawn_x, 400.0, 0.0),
            ai_type,
            enemy_type,
        });
        
        enemy_spawner.enemies_spawned += 1;
        enemy_spawner.spawn_timer = (2.0 - enemy_spawner.wave_timer * SPAWN_RATE_DECAY).max(SPAWN_RATE_MIN);
    }
}

// ===== OPTIMIZED ENEMY SHOOTING =====

pub fn enemy_shooting(
    mut commands: Commands,
    enemy_query: Query<(&Transform, &Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut shoot_timer: Local<f32>,
    mut enemy_index: Local<usize>,
) {
    let Some(assets) = assets else { return };
    
    *shoot_timer -= time.delta_secs();
    if *shoot_timer > 0.0 { return; }
    
    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation.truncate();
    
    // Cycle through enemies instead of random selection
    let enemies: Vec<_> = enemy_query.iter().collect();
    if enemies.is_empty() { return; }
    
    *enemy_index = (*enemy_index + 1) % enemies.len();
    let (enemy_transform, enemy) = enemies[*enemy_index];
    
    let direction = (player_pos - enemy_transform.translation.truncate()).normalize_or_zero();
    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
    
    let color = match enemy.enemy_type {
        EnemyType::ViralParticle => Color::srgb(0.9, 0.9, 1.0),
        EnemyType::AggressiveBacteria => Color::srgb(1.0, 0.4, 0.4),
        EnemyType::ParasiticProtozoa => Color::srgb(0.7, 0.9, 0.4),
        EnemyType::BiofilmColony => Color::srgb(0.6, 0.8, 0.3),
        EnemyType::InfectedMacrophage => Color::srgb(1.0, 0.3, 0.8),
        _ => Color::WHITE,
    };
    
    commands.spawn((
        Sprite { image: assets.projectile_texture.clone(), color, ..default() },
        Transform::from_translation(enemy_transform.translation - Vec3::new(0.0, 20.0, 0.0))
            .with_rotation(Quat::from_rotation_z(angle)),
        Projectile {
            velocity: direction * 300.0,
            damage: 15,
            friendly: false,
            organic_trail: enemy.chemical_signature.releases_toxins,
        },
        Collider { radius: 4.0 },
    ));
    
    *shoot_timer = ENEMY_SHOOT_INTERVAL;
}

// ===== SPAWN ENEMY SYSTEM (simplified) =====

pub fn old_spawn_enemy_system(
    mut commands: Commands,
    mut enemy_events: EventReader<SpawnEnemy>,
    assets: Option<Res<GameAssets>>,
) {
    let Some(assets) = assets else { return };
    
    for event in enemy_events.read() {
        let (health, size, speed, color) = event.enemy_type.get_stats();
        let chemical_signature = event.enemy_type.get_chemical_signature();
        
        let texture = match event.enemy_type {
            EnemyType::ViralParticle => &assets.viral_particle_texture,
            EnemyType::AggressiveBacteria => &assets.aggressive_bacteria_texture,
            EnemyType::ParasiticProtozoa => &assets.parasitic_protozoa_texture,
            EnemyType::InfectedMacrophage => &assets.infected_macrophage_texture,
            EnemyType::SuicidalSpore => &assets.suicidal_spore_texture,
            EnemyType::BiofilmColony => &assets.biofilm_colony_texture,
            EnemyType::SwarmCell => &assets.swarm_cell_texture,
            EnemyType::ReproductiveVesicle => &assets.reproductive_vesicle_texture,
            EnemyType::Offspring => &assets.offspring_texture,
        };

        let entity = commands.spawn((
            Sprite { image: texture.clone(), color, ..default() },
            Transform::from_translation(event.position),
            Enemy {
                ai_type: event.ai_type.clone(),
                health, speed,
                enemy_type: event.enemy_type.clone(),
                colony_id: None,
                chemical_signature: chemical_signature.clone(),
            },
            Collider { radius: size },
            Health(health),
            ChemicalSensitivity {
                ph_tolerance_min: chemical_signature.ph_preference - 1.0,
                ph_tolerance_max: chemical_signature.ph_preference + 1.0,
                oxygen_requirement: chemical_signature.oxygen_tolerance,
                damage_per_second_outside_range: 3,
            },
        )).id();

        // Add specialized components efficiently
        match event.enemy_type {
            EnemyType::ViralParticle => { commands.entity(entity).insert(PulsingAnimation { frequency: 4.0, intensity: 0.2 }); },
            EnemyType::AggressiveBacteria => { commands.entity(entity).insert(FlagellaAnimation { undulation_speed: 6.0, amplitude: 2.0 }); },
            EnemyType::ParasiticProtozoa => { commands.entity(entity).insert(PseudopodAnimation { extension_speed: 3.0, max_extension: 8.0 }); },
            EnemyType::InfectedMacrophage => { commands.entity(entity).insert(CorruptionEffect { intensity: 1.0, color_shift_speed: 2.0 }); },
            EnemyType::SuicidalSpore => { commands.entity(entity).insert(WarningFlash { flash_frequency: 8.0, warning_color: Color::srgb(1.0, 0.3, 0.3) }); },
            EnemyType::BiofilmColony => { commands.entity(entity).insert(ToxicAura { radius: 40.0, pulse_speed: 2.0 }); },
            EnemyType::SwarmCell => { commands.entity(entity).insert(CoordinationIndicator { signal_strength: 1.0, communication_range: 80.0 }); },
            EnemyType::ReproductiveVesicle => { commands.entity(entity).insert(GestationAnimation { pulse_frequency: 1.5, growth_factor: 0.3 }); },
            EnemyType::Offspring => { commands.entity(entity).insert(JuvenileWiggle { wiggle_speed: 10.0, amplitude: 3.0 }); },
        };
    }
}

pub fn spawn_enemy_system(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnEnemy>,
    wave_manager: ResMut<WaveManager>,
    assets: Option<Res<GameAssets>>,
) {
    let Some(assets) = assets else { return };
    
    for event in spawn_events.read() {
        let (base_health, _damage, base_speed, base_color) = event.enemy_type.get_stats();
        let chemical_signature = event.enemy_type.get_chemical_signature();
        
        // Apply wave difficulty scaling
        let (health_mult, speed_mult) = wave_manager.calculate_difficulty_multipliers();
        let final_health = (base_health as f32 * health_mult) as i32;
        let final_speed = base_speed * speed_mult;
        
        // Select appropriate texture
        let texture = match event.enemy_type {
            EnemyType::ViralParticle => assets.viral_particle_texture.clone(),
            EnemyType::AggressiveBacteria => assets.aggressive_bacteria_texture.clone(),
            EnemyType::ParasiticProtozoa => assets.parasitic_protozoa_texture.clone(),
            EnemyType::InfectedMacrophage => assets.infected_macrophage_texture.clone(),
            EnemyType::SuicidalSpore => assets.suicidal_spore_texture.clone(),
            EnemyType::BiofilmColony => assets.biofilm_colony_texture.clone(),
            EnemyType::SwarmCell => assets.swarm_cell_texture.clone(),
            EnemyType::ReproductiveVesicle => assets.reproductive_vesicle_texture.clone(),
            EnemyType::Offspring => assets.offspring_texture.clone(),
        };

        let enemy_entity = commands.spawn((
            Sprite {
                image: texture,
                color: base_color,
                ..default()
            },
            Transform::from_translation(event.position),
            Enemy {
                ai_type: event.ai_type.clone(),
                health: final_health,
                speed: final_speed,
                enemy_type: event.enemy_type.clone(),
                colony_id: None,
                chemical_signature,
            },
            Health(final_health),
            Collider { radius: get_enemy_collision_radius(event.enemy_type.clone()) },
        )).id();

        // Add special components based on enemy type
        match event.enemy_type {
            EnemyType::InfectedMacrophage => {
                commands.entity(enemy_entity).insert(CriticalHitStats::default());
            }
            EnemyType::SuicidalSpore => {
                commands.entity(enemy_entity).insert(ExplosiveProjectile {
                    blast_radius: 60.0,
                    blast_damage: 40,
                    organic_explosion: true,
                });
            }
            EnemyType::ReproductiveVesicle => {
                commands.entity(enemy_entity).insert(ParticleEmitter {
                    spawn_rate: 2.0,
                    spawn_timer: 0.0,
                    particle_config: ParticleConfig::default(),
                    active: true,
                });
            }
            _ => {}
        }

        // Add ecosystem role and predator-prey behavior
        if let Some(behavior) = event.enemy_type.get_predator_prey_behavior() {
            commands.entity(enemy_entity).insert(behavior);
        }
        
        commands.entity(enemy_entity).insert(event.enemy_type.get_ecosystem_role());
        
        // Add adaptive difficulty component for later waves
        if wave_manager.current_wave >= 10 {
            commands.entity(enemy_entity).insert(AdaptiveDifficulty {
                threat_level: 1.0,
                adaptation_rate: 0.5,
                player_evolution_response: 1.0,
            });
        }
    }
}


// ===== MASSIVELY OPTIMIZED COLLISION SYSTEM =====

pub fn collision_system(
    mut commands: Commands,
    mut player_hit_events: EventWriter<PlayerHit>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    mut shake_events: EventWriter<AddScreenShake>,
    mut enemy_hit_events: EventWriter<EnemyHit>,
    mut game_score: ResMut<GameScore>,
    time: Res<Time>,
    fonts: Res<GameFonts>,
    projectile_query: Query<(Entity, &Transform, &Collider, &Projectile), Without<AlreadyDespawned>>,
    mut enemy_query: Query<(Entity, &Transform, &Collider, &mut Health, Option<&Enemy>), (Without<Projectile>, Without<Player>, Without<AlreadyDespawned>)>,
    player_query: Query<(Entity, &Transform, &Collider, &Player, &CriticalHitStats), (With<Player>, Without<Enemy>, Without<AlreadyDespawned>)>,
    mut achievement_events: EventWriter<AchievementEvent>,
) {
    let Ok((_, player_transform, player_collider, player, crit_stats)) = player_query.single() else { return };
    if player.invincible_timer > 0.0 { return; }
    
    let player_pos = player_transform.translation;
    let player_radius = player_collider.radius;
    let time_seed = time.elapsed_secs();
    
    // Track entities to remove to avoid double-processing
    let mut projectiles_to_remove = std::collections::HashSet::new();
    let mut enemies_to_remove = std::collections::HashSet::new();
    
    // Enemy projectiles vs player
    for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
        if projectiles_to_remove.contains(&proj_entity) { continue; }
        if projectile.friendly { continue; }
        
        if check_collision_fast(player_pos, player_radius, proj_transform.translation, proj_collider.radius) {
            player_hit_events.write(PlayerHit { 
                position: proj_transform.translation, 
                damage: projectile.damage 
            });
            shake_events.write(AddScreenShake { amount: 0.5 });
            explosion_events.write(SpawnExplosion { 
                position: proj_transform.translation, 
                intensity: 0.8, 
                enemy_type: None 
            });
            
            commands.entity(proj_entity).safe_despawn();
            projectiles_to_remove.insert(proj_entity);
        }
    }
    
    // Player projectiles vs enemies - ONE projectile per enemy per frame
    for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
        if projectiles_to_remove.contains(&proj_entity) { continue; }
        if !projectile.friendly { continue; }
        
        let proj_pos = proj_transform.translation;
        let proj_radius = proj_collider.radius;
        
        // Find closest enemy that this projectile can hit
        let mut closest_enemy: Option<(Entity, f32)> = None;
        
        for (enemy_entity, enemy_transform, enemy_collider, enemy_health, enemy_opt) in enemy_query.iter() {
            if enemies_to_remove.contains(&enemy_entity) { continue; }
            if enemy_opt.is_none() { continue; }
            
            if check_collision_fast(proj_pos, proj_radius, enemy_transform.translation, enemy_collider.radius) {
                let distance_sq = proj_pos.distance_squared(enemy_transform.translation);
                
                if let Some((_, current_distance)) = closest_enemy {
                    if distance_sq < current_distance {
                        closest_enemy = Some((enemy_entity, distance_sq));
                    }
                } else {
                    closest_enemy = Some((enemy_entity, distance_sq));
                }
            }
        }
        
        // Process hit with closest enemy
        if let Some((enemy_entity, _)) = closest_enemy {
            if let Ok((_, enemy_transform, _, mut enemy_health, enemy_opt)) = enemy_query.get_mut(enemy_entity) {
                if let Some(enemy) = enemy_opt {
                    let seed = proj_pos.x * 0.1 + time_seed;
                    let (final_damage, is_crit) = calculate_crit_hit(projectile.damage, crit_stats, seed);
                    
                    enemy_health.0 -= final_damage;
                    enemy_hit_events.write(EnemyHit { 
                        entity: enemy_entity, 
                        position: enemy_transform.translation 
                    });
                    
                    explosion_events.write(SpawnExplosion { 
                        position: proj_pos, 
                        intensity: 0.6, 
                        enemy_type: None 
                    });
                    
                    spawn_damage_text_fast(&mut commands, enemy_transform.translation, final_damage, is_crit, &fonts);
                    
                    // Remove projectile
                    commands.entity(proj_entity).safe_despawn();
                    projectiles_to_remove.insert(proj_entity);
                    
                    // Check if enemy died
                    if enemy_health.0 <= 0 {
                        let enemy_type = &enemy.enemy_type;
                        game_score.current += enemy_type.get_points();
                        
                        let shake = match enemy_type {
                            EnemyType::InfectedMacrophage => 0.8,
                            EnemyType::ParasiticProtozoa => 0.4,
                            _ => 0.2,
                        };
                        shake_events.write(AddScreenShake { amount: shake });
                        explosion_events.write(SpawnExplosion { 
                            position: enemy_transform.translation, 
                            intensity: 1.0, 
                            enemy_type: Some(enemy_type.clone()) 
                        });
                        
                        commands.entity(enemy_entity).try_insert(AlreadyDespawned).try_despawn();
                        enemies_to_remove.insert(enemy_entity);
                        game_score.enemies_defeated += 1;
                        achievement_events.write(AchievementEvent::EnemyKilled(enemy_type.get_biological_description().to_string()));
                    }
                }
            }
        }
    }
    
    // Enemy vs player collision
    for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health, enemy_opt) in enemy_query.iter_mut() {
        if enemies_to_remove.contains(&enemy_entity) { continue; }
        if enemy_opt.is_none() { continue; }
        
        if check_collision_fast(player_pos, player_radius, enemy_transform.translation, enemy_collider.radius) {
            player_hit_events.write(PlayerHit { 
                position: enemy_transform.translation, 
                damage: 20 
            });
            shake_events.write(AddScreenShake { amount: 0.6 });
            
            // Damage enemy from collision
            enemy_health.0 -= 30;
            if enemy_health.0 <= 0 {
                game_score.current += 50;
                achievement_events.write(AchievementEvent::EnemyKilled("Collision Kill".to_string()));
                explosion_events.write(SpawnExplosion { 
                    position: enemy_transform.translation, 
                    intensity: 1.0, 
                    enemy_type: None 
                });
                commands.entity(enemy_entity).safe_despawn();
            }
        }
    }
}

// Also update the collision radius function to be more generous:
fn get_enemy_collision_radius(enemy_type: EnemyType) -> f32 {
    match enemy_type {
        EnemyType::ViralParticle | EnemyType::Offspring => 12.0, // Increased from 8.0
        EnemyType::AggressiveBacteria | EnemyType::SwarmCell => 16.0, // Increased from 12.0
        EnemyType::ParasiticProtozoa | EnemyType::SuicidalSpore => 20.0, // Increased from 16.0
        EnemyType::BiofilmColony | EnemyType::ReproductiveVesicle => 24.0, // Increased from 20.0
        EnemyType::InfectedMacrophage => 28.0, // Increased from 24.0
    }
}

// ===== REMAINING SYSTEMS (kept for completeness) =====

pub fn load_high_scores(mut game_score: ResMut<GameScore>) {
    game_score.high_scores = vec![10000, 7500, 5000, 2500, 1000];
}

pub fn save_high_score_system(mut game_score: ResMut<GameScore>) {
    let game_score_clone = game_score.clone();
    if game_score.current > 0 {
        game_score.high_scores.push(game_score_clone.current);
        game_score.high_scores.sort_by(|a, b| b.cmp(a));
        game_score.high_scores.truncate(10);
    }
}

pub fn check_game_over(
    mut commands: Commands,
    player_query: Query<(Entity, &Health, &Transform, &Player), (With<Player>, Without<AlreadyDespawned>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
) {
    if let Ok((entity, health, transform, player)) = player_query.single() {
        if health.0 <= 0 && player.lives <= 0 {
            explosion_events.write(SpawnExplosion {
                position: transform.translation,
                intensity: 2.5,
                enemy_type: None,
            });
            commands.entity(entity).try_insert(AlreadyDespawned).despawn();
            next_state.set(GameState::GameOver);
        }
    }
}

pub fn handle_restart_button(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<RestartButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => next_state.set(GameState::Playing),
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.25, 0.7, 0.25)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
        }
    }
}

pub fn handle_player_hit(
    mut commands: Commands,
    mut player_hit_events: EventReader<PlayerHit>,
    mut player_query: Query<(Entity, &mut Health, &mut Player, &CellularUpgrades, Option<&CellWallReinforcement>), With<Player>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in player_hit_events.read() {
        if let Ok((entity, mut health, mut player, upgrades, cell_wall)) = player_query.single_mut() {
            if cell_wall.is_some() || player.invincible_timer > 0.0 { continue; }

            health.0 -= event.damage;
            player.invincible_timer = 1.0;

            explosion_events.write(SpawnExplosion { position: event.position, intensity: 0.8, enemy_type: None });

            if health.0 <= 0 {
                player.lives -= 1;
                if player.lives > 0 {
                    health.0 = upgrades.max_health;
                    player.invincible_timer = 3.0;
                } else {
                    commands.entity(entity).try_insert(AlreadyDespawned).despawn();
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}

// ===== OPTIMIZED VISUAL EFFECT SYSTEMS =====

pub fn damage_text_system(
    mut commands: Commands,
    mut damage_query: Query<(Entity, &mut Transform, &mut DamageText, &mut TextColor), Without<AlreadyDespawned>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    damage_query.par_iter_mut().for_each(|(entity, mut transform, mut damage_text, mut text_color)| {
        damage_text.timer -= dt;
        transform.translation += damage_text.velocity.extend(0.0) * dt;
        
        let alpha = damage_text.timer / 1.5;
        text_color.0 = Color::srgba(1.0, 0.3, 0.3, alpha);
    });
    
    // Cleanup in separate pass
    for (entity, _, damage_text, _) in damage_query.iter() {
        if damage_text.timer <= 0.0 {
            commands.entity(entity).try_insert(AlreadyDespawned).despawn();
        }
    }
}

pub fn update_cell_wall_timer(
    mut commands: Commands,
    mut cell_wall_query: Query<(Entity, &mut CellWallReinforcement)>,
    mut ui_query: Query<&mut Text, With<CellWallTimerText>>,
    time: Res<Time>,
) {
    if let Ok((entity, mut cell_wall)) = cell_wall_query.single_mut() {
        cell_wall.timer -= time.delta_secs();
        if let Ok(mut text) = ui_query.single_mut() {
            **text = format!("Cell Wall: {:.1}s", cell_wall.timer.max(0.0));
        }
        if cell_wall.timer <= 0.0 {
            commands.entity(entity).remove::<CellWallReinforcement>();
        }
    } else if let Ok(mut text) = ui_query.single_mut() {
        **text = String::new();
    }
}

pub fn enemy_flash_system(
    mut commands: Commands,
    mut flash_query: Query<(Entity, &mut FlashEffect, &mut Sprite)>,
    mut enemy_hit_events: EventReader<EnemyHit>,
    // UPDATED: Now returns (Entity, &Sprite) and filters out despawned entities
    enemy_query: Query<(&Sprite), (
        With<Enemy>, 
        Without<FlashEffect>, 
        Without<AlreadyDespawned>, 
        Without<PendingDespawn>
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    // Add flash effects to hit enemies
    for event in enemy_hit_events.read() {
        if let Ok(sprite) = enemy_query.get(event.entity) {
            commands.entity(event.entity).try_insert(FlashEffect {
                timer: 0.0, duration: 0.15,
                original_color: sprite.color,
                flash_color: Color::WHITE,
            });
        }
    }
    
    // Update existing flash effects
    flash_query.par_iter_mut().for_each(|(entity, mut flash, mut sprite)| {
        flash.timer += dt;
    });
    
    // Process flash removal in separate pass
    for (entity, flash, mut sprite) in flash_query.iter_mut() {
        if flash.timer >= flash.duration {
            sprite.color = flash.original_color;
            commands.entity(entity).try_remove::<FlashEffect>();
        } else {
            let progress = flash.timer / flash.duration;
            let intensity = if progress < 0.3 { 
                1.0 - (progress / 0.3) 
            } else { 
                ((1.0 - progress) / 0.7).powi(2) 
            };
            sprite.color = flash.flash_color.mix(&flash.original_color, intensity);
        }
    }
}

pub fn screen_shake_system(
    mut shake_resource: ResMut<ScreenShakeResource>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut shake_events: EventReader<AddScreenShake>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    // Add trauma from events
    for event in shake_events.read() {
        shake_resource.trauma = (shake_resource.trauma + event.amount).min(shake_resource.max_trauma);
    }
    
    // Decay trauma
    shake_resource.trauma = (shake_resource.trauma - shake_resource.decay_rate * dt).max(0.0);
    
    // Apply shake to camera
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if shake_resource.trauma > 0.0 {
            let shake = shake_resource.trauma.powi(2);
            let time_factor = time.elapsed_secs();
            let shake_x = (time_factor * 47.3).sin() * shake * shake_resource.shake_intensity
                + (time_factor * 23.1).sin() * shake * shake_resource.shake_intensity * 0.5;
            let shake_y = (time_factor * 34.7).cos() * shake * shake_resource.shake_intensity
                + (time_factor * 18.9).cos() * shake * shake_resource.shake_intensity * 0.5;
            let rotation_shake = (time_factor * 15.6).sin() * shake * shake_resource.rotation_factor;
            
            camera_transform.translation.x = shake_x;
            camera_transform.translation.y = shake_y;
            camera_transform.rotation = Quat::from_rotation_z(rotation_shake);
        } else {
            camera_transform.translation.x = 0.0;
            camera_transform.translation.y = 0.0;
            camera_transform.rotation = Quat::IDENTITY;
        }
    }
}

pub fn performance_optimization_system(
    mut commands: Commands,
    particle_query: Query<Entity, (With<Particle>, Without<AlreadyDespawned>)>,
    explosion_query: Query<Entity, (With<Explosion>, Without<AlreadyDespawned>)>,
    audio_query: Query<Entity, (With<AudioPlayer>, Without<AlreadyDespawned>)>,
    time: Res<Time>,
    mut cleanup_timer: Local<f32>,
) {
    *cleanup_timer += time.delta_secs();
    
    if *cleanup_timer >= CLEANUP_INTERVAL {
        *cleanup_timer = 0.0;
        
        let total_entities = particle_query.iter().count() + explosion_query.iter().count();
        
        if total_entities > MAX_PARTICLES {
            let particles: Vec<Entity> = particle_query.iter().collect();
            let remove_count = (total_entities - CLEANUP_PARTICLES_TO).min(particles.len());
            
            for &entity in particles.iter().take(remove_count) {
                commands.entity(entity).try_insert(AlreadyDespawned).despawn();
            }
        }
        
        let audio_count = audio_query.iter().count();
        if audio_count > MAX_AUDIO_ENTITIES {
            warn!("High audio entity count: {}", audio_count);
        }
    }
}