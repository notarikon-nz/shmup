use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::achievements::*;
use crate::enemy_types::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

// ===== GENERIC SYSTEMS (unchanged) =====

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { 
                viewport_height: 720.0 
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

pub fn handle_input(
    mut input_state: ResMut<InputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<(Entity, &Gamepad)>,
) {
    if input_state.gamepad.is_none() {
        if let Some((entity, _)) = gamepads.iter().next() {
            input_state.gamepad = Some(entity);
        }
    }
    
    let mut movement = Vec2::ZERO;
    
    // Keyboard input
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        movement.y -= 1.0;
    }
    
    // Gamepad input
    if let Some(gamepad_entity) = input_state.gamepad {
        if let Ok((_, gamepad)) = gamepads.get(gamepad_entity) {
            let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
            let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
            
            movement += Vec2::new(left_stick_x, left_stick_y);
        }
    }
    
    input_state.movement = movement.clamp_length_max(1.0);
    
    // Shooting input
    let mut shooting = keyboard.pressed(KeyCode::Space) || mouse.pressed(MouseButton::Left);
    
    if let Some(gamepad_entity) = input_state.gamepad {
        if let Ok((_, gamepad)) = gamepads.get(gamepad_entity) {
            if gamepad.pressed(GamepadButton::RightTrigger2) {
                shooting = true;
            }
        }
    }
    
    input_state.shooting = shooting;
}

pub fn init_particle_pool(mut commands: Commands) {
    commands.insert_resource(ParticlePool {
        entities: Vec::with_capacity(2000),
        index: 0,
    });
    commands.insert_resource(ShootingState {
        rate_multiplier: 1.0,
        base_rate: 0.1,
    });
}

pub fn update_parallax(
    mut parallax_query: Query<(&mut Transform, &ParallaxLayer)>,
    time: Res<Time>,
) {
    for (mut transform, layer) in parallax_query.iter_mut() {
        transform.translation.y -= layer.speed * 100.0 * time.delta_secs();
        
        if transform.translation.y < -400.0 {
            transform.translation.y = 400.0;
        }
    }
}

pub fn cleanup_offscreen(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (
        Without<Player>, 
        Without<ParallaxLayer>, 
        Without<HealthBarFill>, 
        Without<ScoreText>, 
        Without<HighScoreText>, 
        Without<HealthBar>,
        Without<LivesText>,
        Without<MultiplierText>,
        Without<CellWallVisual>,
        Without<AlreadyDespawned>,
    )>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y < -800.0 || transform.translation.y > 800.0 ||
           transform.translation.x < -800.0 || transform.translation.x > 800.0 {
            commands.entity(entity)
                .try_insert(AlreadyDespawned)
                .despawn();
        }
    }
}

pub fn update_health_bar(
    player_query: Query<(&Health, &CellularUpgrades), With<Player>>,
    mut health_fill_query: Query<&mut Node, (With<HealthBarFill>, Without<HealthNumericText>)>,
    mut health_text_query: Query<&mut Text, (With<HealthNumericText>, Without<HealthBarFill>)>,
) {
    if let Ok((player_health, upgrades)) = player_query.single() {
        let max_health = upgrades.max_health;
        let current_health = player_health.0;
        let health_percent = (current_health as f32 / max_health as f32).clamp(0.0, 1.0);
        
        // Update health bar fill
        if let Ok(mut fill_node) = health_fill_query.single_mut() {
            fill_node.width = Val::Px(200.0 * health_percent);
        }
        
        // Update numeric display
        if let Ok(mut health_text) = health_text_query.single_mut() {
            **health_text = format!("{}/{}", current_health, max_health);
        }
    } else {
        // Player dead - show 0
        if let Ok(mut fill_node) = health_fill_query.single_mut() {
            fill_node.width = Val::Px(0.0);
        }
        if let Ok(mut health_text) = health_text_query.single_mut() {
            **health_text = "0/100".to_string();
        }
    }
}

pub fn load_high_scores(mut game_score: ResMut<GameScore>) {
    game_score.high_scores = vec![10000, 7500, 5000, 2500, 1000];
}

pub fn save_high_score_system(mut game_score: ResMut<GameScore>) {
    save_high_score(&mut game_score);
}

pub fn save_high_score(game_score: &mut GameScore) {
    if game_score.current > 0 {
        game_score.high_scores.push(game_score.current);
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
    if let Ok((player_entity, player_health, player_transform, player)) = player_query.single() {
        if player_health.0 <= 0 && player.lives <= 0 {
            explosion_events.write(SpawnExplosion {
                position: player_transform.translation,
                intensity: 2.5,
                enemy_type: None,
            });
            
            commands.entity(player_entity)
                .try_insert(AlreadyDespawned)
                .despawn();
            next_state.set(GameState::GameOver);
        }
    }
}



pub fn handle_restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}

pub fn handle_restart_button(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<RestartButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.7, 0.25));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.6, 0.2));
            }
        }
    }
}



// ===== SYSTEMS THAT NEED BIOLOGICAL UPDATES =====

// Enhanced projectile movement with fluid dynamics
pub fn move_projectiles(
    mut projectile_query: Query<(&mut Transform, &Projectile)>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (mut transform, projectile) in projectile_query.iter_mut() {
        // Basic projectile movement
        transform.translation += projectile.velocity.extend(0.0) * time.delta_secs();
        
        // Apply slight fluid resistance to projectiles
        if projectile.friendly {
            let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
            let current = sample_current(&fluid_environment, grid_pos);
            
            // Minimal current influence on projectiles
            transform.translation += (current * 0.05).extend(0.0) * time.delta_secs();
        }
    }
}

// Enhanced enemy shooting with biological projectiles
pub fn enemy_shooting(
    mut commands: Commands,
    enemy_query: Query<(&Transform, &Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut shoot_timer: Local<f32>,
) {
    if let Some(assets) = assets {
        *shoot_timer -= time.delta_secs();
        
        if *shoot_timer <= 0.0 && !enemy_query.is_empty() {
            if let Ok(player_transform) = player_query.single() {
                let enemy_count = enemy_query.iter().count();
                if enemy_count > 0 {
                    let random_index = (time.elapsed_secs() * 123.456) as usize % enemy_count;
                    if let Some((enemy_transform, enemy)) = enemy_query.iter().nth(random_index) {
                        let direction = (player_transform.translation.truncate() - enemy_transform.translation.truncate()).normalize_or_zero();
                        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                        
                        // Different projectile colors based on biological enemy type
                        let projectile_color = match enemy.enemy_type {
                            EnemyType::ViralParticle => Color::srgb(0.9, 0.9, 1.0),
                            EnemyType::AggressiveBacteria => Color::srgb(1.0, 0.4, 0.4),
                            EnemyType::ParasiticProtozoa => Color::srgb(0.7, 0.9, 0.4),
                            EnemyType::BiofilmColony => Color::srgb(0.6, 0.8, 0.3),
                            EnemyType::InfectedMacrophage => Color::srgb(1.0, 0.3, 0.8),
                            _ => Color::WHITE,
                        };
                        
                        commands.spawn((
                            Sprite {
                                image: assets.projectile_texture.clone(),
                                color: projectile_color,
                                ..default()
                            },
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
                    }
                }
            }
            
            *shoot_timer = 1.5;
        }
    }
}

// Enhanced player hit system with biological feedback
pub fn handle_player_hit(
    mut commands: Commands,
    mut player_hit_events: EventReader<PlayerHit>,
    mut player_query: Query<(Entity, &mut Health, &mut Player, &CellularUpgrades, Option<&CellWallReinforcement>), With<Player>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in player_hit_events.read() {
        if let Ok((player_entity, mut health, mut player, upgrades, cell_wall)) = player_query.single_mut() {
            // Skip damage if cell wall is active or invincible
            if cell_wall.is_some() || player.invincible_timer > 0.0 {
                continue;
            }

            health.0 -= event.damage;
            player.invincible_timer = 1.0;

            // Organic explosion effect
            explosion_events.write(SpawnExplosion {
                position: event.position,
                intensity: 0.8,
                enemy_type: None,
            });

            if health.0 <= 0 {
                player.lives -= 1;
                
                if player.lives > 0 {
                    // FIXED: Restore health to upgraded maximum, not just 100
                    health.0 = upgrades.max_health;
                    player.invincible_timer = 3.0;
                } else {
                    // Final cellular breakdown
                    commands.entity(player_entity)
                        .try_insert(AlreadyDespawned)
                        .despawn();
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}





// Spawning Systems
pub fn spawn_enemies(
    _commands: Commands,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    time: Res<Time>,
) {
    enemy_spawner.spawn_timer -= time.delta_secs();
    enemy_spawner.wave_timer += time.delta_secs();
    
    if enemy_spawner.spawn_timer <= 0.0 {

        let x_positions = [-400.0, -200.0, 0.0, 200.0, 400.0];
        let spawn_x = x_positions[enemy_spawner.enemies_spawned as usize % x_positions.len()];
        
        let (ai_type, enemy_type) = if enemy_spawner.wave_timer < 20.0 {
            (EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }, EnemyType::ViralParticle)
        } else if enemy_spawner.wave_timer < 40.0 {
            (EnemyAI::Sine { amplitude: 100.0, frequency: 2.0, phase: 0.0 }, EnemyType::AggressiveBacteria)
        } else if enemy_spawner.wave_timer < 60.0 {
            if enemy_spawner.enemies_spawned % 2 == 0 {
                (EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }, EnemyType::ParasiticProtozoa)
            } else {
                (EnemyAI::Chemotaxis { 
                    target_chemical: ChemicalType::PlayerPheromones,
                    sensitivity: 1.5,
                    current_direction: Vec2::new(0.0, -1.0),
                }, EnemyType::AggressiveBacteria)
            }
        } else {
            if enemy_spawner.enemies_spawned % 10 == 0 {
                (EnemyAI::MiniBoss { pattern: 0, timer: 0.0 }, EnemyType::InfectedMacrophage)
            } else {
                (EnemyAI::FluidFlow { 
                    flow_sensitivity: 2.0,
                    base_direction: Vec2::new(0.0, -1.0),
                }, EnemyType::SwarmCell)
            }
        };
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(spawn_x, 400.0, 0.0),
            ai_type,
            enemy_type,
        });

        // Symbiotic pair spawning (separate from main logic)
        if enemy_spawner.wave_timer > 40.0 && enemy_spawner.enemies_spawned % 8 == 0 {
            let pair_x = x_positions[2]; // Use center position
            
            // Spawn first pair member
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(pair_x - 25.0, 400.0, 0.0),
                ai_type: EnemyAI::SymbioticPair {
                    partner_entity: None, // Will be set by pair coordination system
                    bond_distance: 50.0,
                    sync_timer: 0.0,
                },
                enemy_type: EnemyType::SwarmCell,
            });
            
            // Spawn second pair member
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(pair_x + 25.0, 400.0, 0.0),
                ai_type: EnemyAI::SymbioticPair {
                    partner_entity: None, // Will be set by pair coordination system
                    bond_distance: 50.0,
                    sync_timer: 0.0,
                },
                enemy_type: EnemyType::SwarmCell,
            });
            
            enemy_spawner.enemies_spawned += 2;
        }
        
        // Cell division enemies
        if enemy_spawner.wave_timer > 30.0 && enemy_spawner.enemies_spawned % 5 == 0 {
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(spawn_x, 400.0, 0.0),
                ai_type: EnemyAI::CellDivision {
                    division_threshold: 10.0,
                    division_timer: 2.0,
                    has_divided: false,
                },
                enemy_type: EnemyType::AggressiveBacteria,
            });
            
            enemy_spawner.enemies_spawned += 1;
        }

        if enemy_spawner.wave_timer > 50.0 && enemy_spawner.enemies_spawned % 7 == 0 {
            spawn_events.write(SpawnEnemy {
                position: Vec3::new(spawn_x, 400.0, 0.0),
                ai_type: EnemyAI::FluidFlow {
                    flow_sensitivity: 2.5,
                    base_direction: Vec2::new(0.0, -1.0),
                },
                enemy_type: EnemyType::SwarmCell,
            });
            enemy_spawner.enemies_spawned += 1;
        }        

        enemy_spawner.enemies_spawned += 1;
        
        let spawn_rate = (2.0 - (enemy_spawner.wave_timer * 0.02)).max(0.3);
        enemy_spawner.spawn_timer = spawn_rate;
    }
}

pub fn spawn_enemy_system(
    mut commands: Commands,
    mut enemy_events: EventReader<SpawnEnemy>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in enemy_events.read() {
            let (health, size, speed, color) = event.enemy_type.get_stats();
            let chemical_signature = event.enemy_type.get_chemical_signature();
            let chemical_signature_clone = chemical_signature.clone();

            // GET UNIQUE TEXTURE based on enemy type
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

            let mut enemy_commands = commands.spawn((
                Sprite {
                    image: texture,
                    color,
                    ..default()
                },
                Transform::from_translation(event.position),
                Enemy {
                    ai_type: event.ai_type.clone(),
                    health,
                    speed,
                    enemy_type: event.enemy_type.clone(),
                    colony_id: None,
                    chemical_signature,
                },
                Collider { radius: size },
                Health(health),
                ChemicalSensitivity {
                    ph_tolerance_min: chemical_signature_clone.ph_preference - 1.0,
                    ph_tolerance_max: chemical_signature_clone.ph_preference + 1.0,
                    oxygen_requirement: chemical_signature_clone.oxygen_tolerance,
                    damage_per_second_outside_range: 3,
                },
            ));

            // ADD UNIQUE BEHAVIORS based on enemy type
            match event.enemy_type {
                EnemyType::ViralParticle => {
                    // Viruses have pulsing animation
                    enemy_commands.insert(PulsingAnimation {
                        frequency: 4.0,
                        intensity: 0.2,
                    });
                }
                EnemyType::AggressiveBacteria => {
                    // Bacteria have flagella movement
                    enemy_commands.insert(FlagellaAnimation {
                        undulation_speed: 6.0,
                        amplitude: 2.0,
                    });
                }
                EnemyType::ParasiticProtozoa => {
                    // Protozoa have pseudopod extensions
                    enemy_commands.insert(PseudopodAnimation {
                        extension_speed: 3.0,
                        max_extension: 8.0,
                    });
                }
                EnemyType::InfectedMacrophage => {
                    // Macrophages have corruption effects
                    enemy_commands.insert(CorruptionEffect {
                        intensity: 1.0,
                        color_shift_speed: 2.0,
                    });
                }
                EnemyType::SuicidalSpore => {
                    // Spores have warning flash
                    enemy_commands.insert(WarningFlash {
                        flash_frequency: 8.0,
                        warning_color: Color::srgb(1.0, 0.3, 0.3),
                    });
                }
                EnemyType::BiofilmColony => {
                    // Colonies have toxic aura
                    enemy_commands.insert(ToxicAura {
                        radius: 40.0,
                        pulse_speed: 2.0,
                    });
                }
                EnemyType::SwarmCell => {
                    // Swarm cells have coordination indicators
                    enemy_commands.insert(CoordinationIndicator {
                        signal_strength: 1.0,
                        communication_range: 80.0,
                    });
                }
                EnemyType::ReproductiveVesicle => {
                    // Vesicles have gestation animation
                    enemy_commands.insert(GestationAnimation {
                        pulse_frequency: 1.5,
                        growth_factor: 0.3,
                    });
                }
                EnemyType::Offspring => {
                    // Offspring have juvenile wiggle
                    enemy_commands.insert(JuvenileWiggle {
                        wiggle_speed: 10.0,
                        amplitude: 3.0,
                    });
                }
            }


        }
    }
}





// Helper functions for fluid dynamics
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
    mut enemy_query: Query<(Entity, &Transform, &Collider, &mut Health, Option<&Enemy>), (Without<Projectile>, Without<AlreadyDespawned>)>,
    player_query: Query<(Entity, &Transform, &Collider, &Player, &CriticalHitStats), (With<Player>, Without<Enemy>)>,

    mut achievement_events: EventWriter<AchievementEvent>,
) {
    if let Ok((_, player_transform, player_collider, player, crit_stats)) = player_query.single() {
        // Skip if invincible
        if player.invincible_timer > 0.0 { return; }
        
        // Enemy projectiles vs player
        for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
            if projectile.friendly { continue; }
            
            let distance = player_transform.translation.distance(proj_transform.translation);
            if distance < player_collider.radius + proj_collider.radius {
                player_hit_events.write(PlayerHit {
                    position: proj_transform.translation,
                    damage: projectile.damage,
                });
                
                // Screen shake for player damage
                shake_events.write(AddScreenShake { amount: 0.5 });
                
                // Mini explosion on hit
                explosion_events.write(SpawnExplosion {
                    position: proj_transform.translation,
                    intensity: 0.8,
                    enemy_type: None,
                });
                
                commands.entity(proj_entity)
                    .try_insert(AlreadyDespawned)
                    .despawn();
            }
        }
        
        // Player projectiles vs enemies
        for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
            if !projectile.friendly { continue; }
            
            for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health, enemy_opt) in enemy_query.iter_mut() {
                if enemy_opt.is_none() { continue; }
                
                let distance = proj_transform.translation.distance(enemy_transform.translation);
                if distance < proj_collider.radius + enemy_collider.radius {
                    // Critical hit calculation
                    let is_crit = (proj_transform.translation.x * 123.456 + time.elapsed_secs()).sin().abs() < crit_stats.chance;
                    let final_damage = if is_crit {
                        (projectile.damage as f32 * crit_stats.damage_multiplier) as i32
                    } else {
                        projectile.damage
                    };
                    
                    enemy_health.0 -= final_damage;
                    
                    // Flash enemy when hit
                    enemy_hit_events.write(EnemyHit {
                        entity: enemy_entity,
                        position: enemy_transform.translation,
                    });
                    
                    // Mini explosion on projectile hit
                    explosion_events.write(SpawnExplosion {
                        position: proj_transform.translation,
                        intensity: 0.6,
                        enemy_type: None,
                    });
                    
                    // Spawn damage text
                    let (text_color, font_size) = if is_crit {
                        (Color::srgb(1.0, 1.0, 0.3), 16.0)
                    } else {
                        (Color::srgb(1.0, 1.0, 1.0), 12.0)
                    };
                    
                    commands.spawn((
                        Text2d::new(format!("{}", final_damage)),
                        TextFont { 
                            font: fonts.default_font.clone(),
                            font_size, ..default() },
                        TextColor(text_color),
                        Transform::from_translation(enemy_transform.translation + Vec3::new(0.0, 25.0, 1.0)),
                        DamageText {
                            timer: 1.5,
                            velocity: Vec2::new(0.0, 80.0),
                        },
                    ));
                    
                    commands.entity(proj_entity)
                        .try_insert(AlreadyDespawned)
                        .despawn();
                    
                    if enemy_health.0 <= 0 {
                        let enemy_type = &enemy_opt.unwrap().enemy_type;
                        game_score.current += enemy_type.get_points();
                        
                        // Screen shake based on enemy size
                        let shake_amount = match enemy_type {
                            EnemyType::InfectedMacrophage => 0.8,
                            EnemyType::ParasiticProtozoa => 0.4,
                            _ => 0.2,
                        };
                        shake_events.write(AddScreenShake { amount: shake_amount });
                        
                        explosion_events.write(SpawnExplosion {
                            position: enemy_transform.translation,
                            intensity: 1.0,
                            enemy_type: Some(enemy_type.clone()),
                        });
                        commands.entity(enemy_entity)
                            .try_insert(AlreadyDespawned)
                            .despawn();

                        // stats tracking
                        game_score.enemies_defeated += 1;

                        achievement_events.write(AchievementEvent::EnemyKilled(
                            enemy_type.get_biological_description().to_string()
                        ));                     
                    }
                    break;
                }
            }
        }
        
        // Enemy vs player collision
        for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health, enemy_opt) in enemy_query.iter_mut() {
            if enemy_opt.is_none() { continue; }
            
            let distance = player_transform.translation.distance(enemy_transform.translation);
            if distance < player_collider.radius + enemy_collider.radius {
                player_hit_events.write(PlayerHit {
                    position: enemy_transform.translation,
                    damage: 20,
                });
                
                // Screen shake for enemy collision
                shake_events.write(AddScreenShake { amount: 0.6 });
                
                enemy_health.0 -= 30;
                if enemy_health.0 <= 0 {
                    game_score.current += 50;

                    // Achievement tracking for ramming kills
                    achievement_events.write(AchievementEvent::EnemyKilled(
                        "Collision Kill".to_string()
                    ));

                    explosion_events.write(SpawnExplosion {
                        position: enemy_transform.translation,
                        intensity: 1.0,
                        enemy_type: None,
                    });
                    commands.entity(enemy_entity)
                        .try_insert(AlreadyDespawned)
                        .despawn();
                }
            }
        }
    }
}


// BEGIN FPS
pub fn fps_system(
    diagnostics: Res<DiagnosticsStore>, 
    mut query: Query<&mut Text, With<FpsText>>
) {

    if let Ok(mut fps_text) = query.single_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **fps_text = format!("FPS: {:.0}", value);
            } else {
                **fps_text = format!("FPS: --");
            }
        } else {
                **fps_text = format!("FPS: N/A");
        }
    }
}



// END FPS

// floating combat text
pub fn damage_text_system(
    mut commands: Commands,
    mut damage_query: Query<(Entity, &mut Transform, &mut DamageText, &mut TextColor), Without<AlreadyDespawned>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut damage_text, mut text_color) in damage_query.iter_mut() {
        damage_text.timer -= time.delta_secs();
        transform.translation += damage_text.velocity.extend(0.0) * time.delta_secs();
        
        let alpha = damage_text.timer / 1.5;
        text_color.0 = Color::srgba(1.0, 0.3, 0.3, alpha);
        
        if damage_text.timer <= 0.0 {
            commands.entity(entity)
                .try_insert(AlreadyDespawned)
                .despawn();
                
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
        
        // Update UI countdown
        if let Ok(mut text) = ui_query.single_mut() {
            **text = format!("Cell Wall: {:.1}s", cell_wall.timer.max(0.0));
        }
        
        if cell_wall.timer <= 0.0 {
            commands.entity(entity).remove::<CellWallReinforcement>();
        }
    } else {
        // Clear timer text when no cell wall active
        if let Ok(mut text) = ui_query.single_mut() {
            **text = String::new();
        }
    }
}

pub fn enemy_flash_system(
    mut commands: Commands,
    mut flash_query: Query<(Entity, &mut FlashEffect, &mut Sprite)>,
    mut enemy_hit_events: EventReader<EnemyHit>,
    enemy_query: Query<&Sprite, (With<Enemy>, Without<FlashEffect>)>,
    time: Res<Time>,
) {
    // Handle new hits
    for event in enemy_hit_events.read() {
        if let Ok(enemy_sprite) = enemy_query.get(event.entity) {
            commands.entity(event.entity).insert(FlashEffect {
                timer: 0.0,
                duration: 0.15,
                original_color: enemy_sprite.color,
                flash_color: Color::srgb(1.0, 1.0, 1.0),
            });
        }
    }
    
    // Update existing flashes
    for (entity, mut flash, mut sprite) in flash_query.iter_mut() {
        flash.timer += time.delta_secs();
        
        if flash.timer >= flash.duration {
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<FlashEffect>();
        } else {
            let progress = flash.timer / flash.duration;
            let flash_intensity = if progress < 0.3 {
                1.0 - (progress / 0.3)
            } else {
                ((1.0 - progress) / 0.7).powi(2)
            };
            
            sprite.color = flash.flash_color.mix(&flash.original_color, flash_intensity);
        }
    }
}

pub fn screen_shake_system(
    mut shake_resource: ResMut<ScreenShakeResource>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut shake_events: EventReader<AddScreenShake>,
    time: Res<Time>,
) {
    // Add new trauma
    for event in shake_events.read() {
        shake_resource.trauma = (shake_resource.trauma + event.amount).min(shake_resource.max_trauma);
    }
    
    // Decay trauma with organic falloff
    shake_resource.trauma = (shake_resource.trauma - shake_resource.decay_rate * time.delta_secs()).max(0.0);
    
    // Apply enhanced shake to camera
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        if shake_resource.trauma > 0.0 {
            let shake = shake_resource.trauma.powi(2); // Quadratic falloff
            
            // Multi-frequency shake for more organic feel
            let time_factor = time.elapsed_secs();
            let shake_x = (time_factor * 47.3).sin() * shake * shake_resource.shake_intensity
                + (time_factor * 23.1).sin() * shake * shake_resource.shake_intensity * 0.5;
            let shake_y = (time_factor * 34.7).cos() * shake * shake_resource.shake_intensity
                + (time_factor * 18.9).cos() * shake * shake_resource.shake_intensity * 0.5;
            
            // Rotation shake for impact feel
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
    entity_query: Query<Entity, (Or<(With<Particle>, With<Enemy>, With<Projectile>)>, Without<AlreadyDespawned>)>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        let mut entities_processed = 0;
        const MAX_ENTITIES_PER_FRAME: usize = 100;
        
        for entity in entity_query.iter() {
            // Could add distance culling here for very far entities
            entities_processed += 1;
            if entities_processed >= MAX_ENTITIES_PER_FRAME {
                break;
            }
        }
    }
}