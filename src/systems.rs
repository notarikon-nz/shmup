use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::render::camera::ScalingMode;

use crate::components::*;
use crate::resources::*;
use crate::events::*;

// Setup Systems
pub fn startup_debug() {
    info!("=== STARTUP SYSTEMS RUNNING ===");
}

pub fn setup_camera(mut commands: Commands) {
    info!("Setting up camera...");
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { 
                viewport_height: 720.0 
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
    info!("Camera setup complete");
}

pub fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let layers = vec![
        ("textures/bg_layer1.png", 0.1, -100.0),
        ("textures/bg_layer2.png", 0.3, -50.0),
        ("textures/bg_layer3.png", 0.6, -25.0),
    ];
    
    for (path, speed, depth) in layers {
        let texture = asset_server.load(path);
        commands.spawn((
            Sprite::from_image(texture),
            Transform::from_xyz(0.0, 0.0, depth),
            ParallaxLayer { speed, depth },
        ));
    }
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("textures/player.png");
    
    commands.spawn((
        Sprite {
            image: texture,
            anchor: Anchor::Center,
            ..default()
        },
        Transform::from_xyz(0.0, -250.0, 0.0),
        Player {
            speed: 400.0,
            roll_factor: 0.3,
        },
        Collider { radius: 16.0 },
        Health(100),
    ));
}

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        player_texture: asset_server.load("textures/player.png"),
        enemy_texture: asset_server.load("textures/enemy.png"),
        projectile_texture: asset_server.load("textures/bullet.png"),
        explosion_texture: asset_server.load("textures/explosion.png"),
        health_powerup_texture: asset_server.load("textures/health_powerup.png"),
        background_layers: vec![
            asset_server.load("textures/bg_layer1.png"),
            asset_server.load("textures/bg_layer2.png"),
            asset_server.load("textures/bg_layer3.png"),
        ],
        sfx_shoot: asset_server.load("audio/shoot.ogg"),
        sfx_explosion: asset_server.load("audio/explosion.ogg"),
        sfx_powerup: asset_server.load("audio/powerup.ogg"),
        music: asset_server.load("audio/music.ogg"),
    };
    
    commands.insert_resource(assets);
    
    let pool = ProjectilePool {
        entities: Vec::with_capacity(1000),
        index: 0,
    };
    commands.insert_resource(pool);
}

pub fn setup_ui(mut commands: Commands) {
    info!("=== STARTING UI SETUP ===");
    
    // Health bar background (dark border)
    info!("Creating health bar background...");
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(204.0), // Slightly larger for border effect
            height: Val::Px(24.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        BorderColor(Color::srgb(0.8, 0.8, 0.8)),
        HealthBar,
    ));
    info!("Health bar background created");
    
    // Health bar fill (red bar)
    info!("Creating health bar fill...");
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(22.0),
            bottom: Val::Px(22.0),
            width: Val::Px(200.0),
            height: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
        HealthBarFill,
    ));
    info!("Health bar fill created");

    // Score text
    info!("Creating score text...");
    commands.spawn((
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        ScoreText,
    ));
    info!("Score text created");
    
    // High score text
    info!("Creating high score text...");
    commands.spawn((
        Text::new("High: 0"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(50.0),
            ..default()
        },
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        HighScoreText,
    ));
    info!("High score text created");
    
    info!("=== UI SETUP COMPLETE ===");
}




// Input Systems
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

// Movement Systems
pub fn move_player(
    mut player_query: Query<(&mut Transform, &Player)>,
    input_state: Res<InputState>,
    time: Res<Time>,
) {
    if let Ok((mut transform, player)) = player_query.single_mut() {
        let movement = input_state.movement * player.speed * time.delta_secs();
        transform.translation += movement.extend(0.0);
        
        transform.translation.x = transform.translation.x.clamp(-600.0, 600.0);
        transform.translation.y = transform.translation.y.clamp(-350.0, 350.0);
        
        let target_roll = -input_state.movement.x * player.roll_factor;
        transform.rotation = transform.rotation.lerp(
            Quat::from_rotation_z(target_roll),
            time.delta_secs() * 8.0
        );
    }
}

pub fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    time: Res<Time>,
) {
    for (mut transform, mut enemy) in enemy_query.iter_mut() {
        let mut enemy_c = enemy.clone();
        match &mut enemy_c.ai_type {
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
        }
    }
}

pub fn move_projectiles(
    mut projectile_query: Query<(&mut Transform, &Projectile)>,
    time: Res<Time>,
) {
    for (mut transform, projectile) in projectile_query.iter_mut() {
        transform.translation += projectile.velocity.extend(0.0) * time.delta_secs();
    }
}

// Spawning Systems
pub fn spawn_enemies(
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    time: Res<Time>,
) {
    enemy_spawner.spawn_timer -= time.delta_secs();
    enemy_spawner.wave_timer += time.delta_secs();
    
    if enemy_spawner.spawn_timer <= 0.0 {
        let x_positions = [-400.0, -200.0, 0.0, 200.0, 400.0];
        let spawn_x = x_positions[enemy_spawner.enemies_spawned as usize % x_positions.len()];
        
        let ai_type = if enemy_spawner.wave_timer < 20.0 {
            EnemyAI::Linear { 
                direction: Vec2::new(0.0, -1.0) 
            }
        } else if enemy_spawner.wave_timer < 40.0 {
            EnemyAI::Sine { 
                amplitude: 100.0, 
                frequency: 2.0, 
                phase: 0.0 
            }
        } else if enemy_spawner.wave_timer < 60.0 {
            if enemy_spawner.enemies_spawned % 2 == 0 {
                EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }
            } else {
                EnemyAI::Sine { amplitude: 150.0, frequency: 3.0, phase: 0.0 }
            }
        } else {
            if enemy_spawner.enemies_spawned % 10 == 0 {
                EnemyAI::MiniBoss { pattern: 0, timer: 0.0 }
            } else {
                EnemyAI::Sine { amplitude: 200.0, frequency: 4.0, phase: 0.0 }
            }
        };
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(spawn_x, 400.0, 0.0),
            ai_type,
        });
        
        enemy_spawner.enemies_spawned += 1;
        
        let spawn_rate = (2.0 - (enemy_spawner.wave_timer * 0.02)).max(0.5);
        enemy_spawner.spawn_timer = spawn_rate;
    }
}

pub fn spawn_projectiles(
    mut commands: Commands,
    input_state: Res<InputState>,
    mut player_query: Query<&Transform, With<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut shoot_timer: Local<f32>,
) {
    if let Some(assets) = assets {
        *shoot_timer -= time.delta_secs();
        
        if input_state.shooting && *shoot_timer <= 0.0 {
            if let Ok(player_transform) = player_query.single() {
                commands.spawn((
                    Sprite::from_image(assets.projectile_texture.clone()),
                    Transform::from_translation(player_transform.translation + Vec3::new(0.0, 30.0, 0.0)),
                    Projectile {
                        velocity: Vec2::new(0.0, 800.0),
                        damage: 10,
                        friendly: true,
                    },
                    Collider { radius: 4.0 },
                ));
                
                *shoot_timer = 0.1;
            }
        }
    }
}

pub fn enemy_shooting(
    mut commands: Commands,
    enemy_query: Query<&Transform, With<Enemy>>,
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
                    if let Some(enemy_transform) = enemy_query.iter().nth(random_index) {
                        let direction = (player_transform.translation.truncate() - enemy_transform.translation.truncate()).normalize_or_zero();
                        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                        
                        commands.spawn((
                            Sprite::from_image(assets.projectile_texture.clone()),
                            Transform::from_translation(enemy_transform.translation - Vec3::new(0.0, 20.0, 0.0))
                                .with_rotation(Quat::from_rotation_z(angle)),
                            Projectile {
                                velocity: direction * 300.0,
                                damage: 15,
                                friendly: false,
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

pub fn handle_collisions(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Collider, &Projectile)>,
    mut enemy_query: Query<(Entity, &Transform, &Collider, &mut Health, &Enemy), (With<Enemy>, Without<Projectile>, Without<Player>)>,
    mut player_query: Query<(Entity, &Transform, &Collider, &mut Health), (With<Player>, Without<Enemy>, Without<Projectile>)>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    mut game_score: ResMut<GameScore>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Projectile vs Enemy collisions
    for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
        if !projectile.friendly { continue; }
        
        for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health, enemy) in enemy_query.iter_mut() {
            let distance = proj_transform.translation.distance(enemy_transform.translation);
            if distance < proj_collider.radius + enemy_collider.radius {
                enemy_health.0 -= projectile.damage;
                commands.entity(proj_entity).despawn();
                
                if enemy_health.0 <= 0 {
                    commands.entity(enemy_entity).despawn();
                    
                    let points = match enemy.ai_type {
                        EnemyAI::MiniBoss { .. } => 500,
                        EnemyAI::Sine { .. } => 150,
                        EnemyAI::Linear { .. } => 100,
                        EnemyAI::Static => 50,
                    };
                    game_score.current += points;
                    
                    explosion_events.write(SpawnExplosion {
                        position: enemy_transform.translation,
                        intensity: 1.0,
                    });
                }
                break;
            }
        }
    }
    
    // Enemy projectiles vs Player
    for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
        if projectile.friendly { continue; }
        
        if let Ok((_, player_transform, player_collider, mut player_health)) = player_query.single_mut() {
            let distance = proj_transform.translation.distance(player_transform.translation);
            if distance < proj_collider.radius + player_collider.radius {
                player_health.0 -= projectile.damage;
                commands.entity(proj_entity).despawn();
                
                explosion_events.write(SpawnExplosion {
                    position: player_transform.translation,
                    intensity: 0.8,
                });
                
                if player_health.0 <= 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
    
    // Enemy vs Player direct collision
    if let Ok((_, player_transform, player_collider, mut player_health)) = player_query.single_mut() {
        for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health, _) in enemy_query.iter_mut() {
            let distance = player_transform.translation.distance(enemy_transform.translation);
            if distance < player_collider.radius + enemy_collider.radius {
                // Player takes damage
                player_health.0 -= 20;
                
                // Enemy also takes damage/dies from collision
                enemy_health.0 -= 50;
                
                explosion_events.write(SpawnExplosion {
                    position: enemy_transform.translation,
                    intensity: 1.2,
                });
                
                if enemy_health.0 <= 0 {
                    commands.entity(enemy_entity).despawn();
                }
                
                if player_health.0 <= 0 {
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}

// Update Systems
pub fn update_explosions(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Explosion, &mut Transform, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut explosion, mut transform, mut sprite) in explosion_query.iter_mut() {
        explosion.timer += time.delta_secs();
        
        let progress = explosion.timer / explosion.max_time;
        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        
        let scale = 1.0 + progress * 2.0;
        transform.scale = Vec3::splat(scale);
        sprite.color.set_alpha(1.0 - progress);
    }
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

pub fn update_lights(
    mut light_query: Query<(&mut Light2D, &Transform, Option<&Explosion>)>,
    explosion_query: Query<(&Transform, &Explosion), Without<Light2D>>,
) {
    for (_explosion_transform, explosion) in explosion_query.iter() {
        let progress = explosion.timer / explosion.max_time;
        let _intensity = explosion.intensity * (1.0 - progress);
    }
}

pub fn cleanup_offscreen(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (Without<Player>, Without<ParallaxLayer>, Without<HealthBarFill>, Without<ScoreText>, Without<HighScoreText>, Without<HealthBar>)>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y < -400.0 || transform.translation.y > 500.0 ||
           transform.translation.x < -700.0 || transform.translation.x > 700.0 {
            // Only log occasionally to avoid spam
            commands.entity(entity).despawn();
        }
    }
}

// Event Systems
pub fn spawn_explosion_system(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in explosion_events.read() {
            commands.spawn((
                Sprite::from_image(assets.explosion_texture.clone()),
                Transform::from_translation(event.position),
                Explosion {
                    timer: 0.0,
                    max_time: 0.5,
                    intensity: event.intensity,
                },
                Light2D {
                    color: Color::srgb(1.0, 0.8, 0.4),
                    intensity: event.intensity,
                    radius: 100.0,
                },
            ));
        }
    }
}

pub fn spawn_enemy_system(
    mut commands: Commands,
    mut enemy_events: EventReader<SpawnEnemy>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in enemy_events.read() {
            let (health, size) = match &event.ai_type {
                EnemyAI::MiniBoss { .. } => (50, 25.0),
                _ => (20, 15.0),
            };
            
            commands.spawn((
                Sprite::from_image(assets.enemy_texture.clone()),
                Transform::from_translation(event.position),
                Enemy {
                    ai_type: event.ai_type.clone(),
                    health,
                    speed: 150.0,
                },
                Collider { radius: size },
                Health(health),
            ));
        }
    }
}

// UI Systems
pub fn update_health_bar(
    player_query: Query<&Health, With<Player>>,
    mut health_fill_query: Query<&mut Node, With<HealthBarFill>>,
) {
    if let Ok(player_health) = player_query.single() {
        if let Ok(mut fill_node) = health_fill_query.single_mut() {
            let health_percent = (player_health.0 as f32 / 100.0).clamp(0.0, 1.0);
            fill_node.width = Val::Px(200.0 * health_percent);
        }
    } else {
        // No player exists (probably dead), set health bar to 0
        if let Ok(mut fill_node) = health_fill_query.single_mut() {
            fill_node.width = Val::Px(0.0);
        }
    }
}

pub fn update_score_display(
    game_score: Res<GameScore>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<HighScoreText>)>,
    mut high_score_query: Query<&mut Text, (With<HighScoreText>, Without<ScoreText>)>,
) {
    // Update current score
    if let Ok(mut score_text) = score_query.single_mut() {
        **score_text = format!("Score: {}", game_score.current);
    }
    
    // Update high score display
    if let Ok(mut high_score_text) = high_score_query.single_mut() {
        let high_score = game_score.high_scores.first().unwrap_or(&0);
        **high_score_text = format!("High: {}", high_score);
    }
}

// Score Systems
pub fn load_high_scores(mut game_score: ResMut<GameScore>) {
    game_score.high_scores = vec![10000, 7500, 5000, 2500, 1000];
}

pub fn save_high_score(game_score: &mut GameScore) {
    if game_score.current > 0 {
        game_score.high_scores.push(game_score.current);
        game_score.high_scores.sort_by(|a, b| b.cmp(a));
        game_score.high_scores.truncate(10); // Keep top 10
        
        // Reset for new game
        game_score.current = 0;
    }
}

pub fn spawn_powerup_system(
    mut commands: Commands,
    mut powerup_events: EventReader<SpawnPowerUp>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in powerup_events.read() {
            let texture = match &event.power_type {
                PowerUpType::Health { .. } => assets.health_powerup_texture.clone(),
            };
            
            commands.spawn((
                Sprite::from_image(texture),
                Transform::from_translation(event.position),
                PowerUp {
                    power_type: event.power_type.clone(),
                    bob_timer: 0.0,
                },
                Collider { radius: 12.0 },
            ));
        }
    }
}

pub fn move_powerups(
    mut powerup_query: Query<(&mut Transform, &mut PowerUp)>,
    time: Res<Time>,
) {
    for (mut transform, mut powerup) in powerup_query.iter_mut() {
        // Slow downward movement
        transform.translation.y -= 100.0 * time.delta_secs();
        
        // Bobbing animation
        powerup.bob_timer += time.delta_secs() * 3.0;
        transform.translation.y += powerup.bob_timer.sin() * 2.0 * time.delta_secs();
        
        // Gentle rotation
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 2.0);
    }
}

pub fn spawn_powerups(
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnPowerUp>,
    time: Res<Time>,
) {
    enemy_spawner.powerup_timer -= time.delta_secs();
    
    // Spawn power-ups every 15 seconds
    if enemy_spawner.powerup_timer <= 0.0 {
        let x_position = (time.elapsed_secs() * 50.0).sin() * 300.0;
        
        spawn_events.write(SpawnPowerUp {
            position: Vec3::new(x_position, 400.0, 0.0),
            power_type: PowerUpType::Health { amount: 25 },
        });
        
        enemy_spawner.powerup_timer = 15.0;
    }
}


pub fn handle_powerup_collection(
    mut commands: Commands,
    powerup_query: Query<(Entity, &Transform, &Collider, &PowerUp)>,
    mut player_query: Query<(&Transform, &Collider, &mut Health), With<Player>>,
) {
    if let Ok((player_transform, player_collider, mut player_health)) = player_query.single_mut() {
        for (powerup_entity, powerup_transform, powerup_collider, powerup) in powerup_query.iter() {
            let distance = player_transform.translation.distance(powerup_transform.translation);
            if distance < player_collider.radius + powerup_collider.radius {
                // Apply power-up effect
                match &powerup.power_type {
                    PowerUpType::Health { amount } => {
                        player_health.0 = (player_health.0 + amount).min(100); // Cap at 100
                    }
                }
                
                // Remove power-up
                commands.entity(powerup_entity).despawn();
                
                // TODO: Play power-up sound effect
            }
        }
    }
}

// Temporary debug system - remove once UI is working
pub fn debug_ui_entities(
    health_bar_query: Query<Entity, With<HealthBar>>,
    health_fill_query: Query<Entity, With<HealthBarFill>>,
    score_query: Query<Entity, With<ScoreText>>,
    high_score_query: Query<Entity, With<HighScoreText>>,
    mut debug_timer: Local<f32>,
    time: Res<Time>,
) {
    *debug_timer += time.delta_secs();
    
    // Print debug info every 5 seconds
    if *debug_timer > 5.0 {
        info!("UI Entity Debug:");
        info!("  Health bars found: {}", health_bar_query.iter().count());
        info!("  Health fills found: {}", health_fill_query.iter().count());
        info!("  Score texts found: {}", score_query.iter().count());
        info!("  High score texts found: {}", high_score_query.iter().count());
        
        *debug_timer = 0.0;
    }
}

// Game Over Systems
pub fn check_game_over(
    mut commands: Commands,
    player_query: Query<(Entity, &Health, &Transform), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
) {
    if let Ok((player_entity, player_health, player_transform)) = player_query.single() {
        if player_health.0 <= 0 {
            // Create a big explosion where the player died
            explosion_events.write(SpawnExplosion {
                position: player_transform.translation,
                intensity: 2.0, // Bigger explosion for player death
            });
            
            // Despawn the dead player immediately
            commands.entity(player_entity).despawn();
            next_state.set(GameState::GameOver);
        }
    }
}

pub fn setup_game_over_ui(
    mut commands: Commands,
    mut game_score: ResMut<GameScore>,
) {
    info!("Setting up game over UI");
    
    // Save high score when entering game over
    save_high_score(&mut game_score);
    
    // Semi-transparent overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GameOverUI,
    )).with_children(|parent| {
        // "GAME OVER" text
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.3, 0.3)),
            GameOverText,
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));
        
        // Final score
        parent.spawn((
            Text::new(format!("Final Score: {}", game_score.current)),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
            FinalScoreText,
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));
        
        // High score
        let high_score = game_score.high_scores.first().unwrap_or(&0);
        parent.spawn((
            Text::new(format!("High Score: {}", high_score)),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));
        
        // Restart button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            RestartButton,
        )).with_children(|button| {
            button.spawn((
                Text::new("RESTART"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
        
        // Instructions
        parent.spawn((
            Text::new("Press R to restart or click button above"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

pub fn cleanup_game_over_ui(
    mut commands: Commands,
    game_over_query: Query<Entity, With<GameOverUI>>,
) {
    info!("Cleaning up game over UI");
    for entity in game_over_query.iter() {
        commands.entity(entity).despawn_recursive();
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

// Reset game state when restarting
pub fn reset_game_state_on_restart(
    mut commands: Commands,
    mut game_score: ResMut<GameScore>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut input_state: ResMut<InputState>,
    mut game_started: ResMut<GameStarted>,
    // Despawn all game entities
    enemy_query: Query<Entity, With<Enemy>>,
    projectile_query: Query<Entity, With<Projectile>>,
    explosion_query: Query<Entity, With<Explosion>>,
    powerup_query: Query<Entity, With<PowerUp>>,
    player_query: Query<Entity, With<Player>>,
    assets: Option<Res<GameAssets>>,
) {
    // Only reset if this is not the first time (i.e., we're restarting)
    if !game_started.0 {
        game_started.0 = true;
        return;
    }
    
    info!("Resetting game state for restart");
    
    // Despawn all game entities
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in projectile_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in explosion_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in powerup_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Reset resources
    game_score.current = 0;
    enemy_spawner.spawn_timer = 2.0;
    enemy_spawner.wave_timer = 0.0;
    enemy_spawner.enemies_spawned = 0;
    enemy_spawner.powerup_timer = 15.0;
    input_state.shoot_timer = 0.0;
    
    // Respawn player if assets are available
    if let Some(assets) = assets {
        commands.spawn((
            Sprite {
                image: assets.player_texture.clone(),
                anchor: Anchor::Center,
                ..default()
            },
            Transform::from_xyz(0.0, -250.0, 0.0),
            Player {
                speed: 400.0,
                roll_factor: 0.3,
            },
            Collider { radius: 16.0 },
            Health(100),
        ));
    }
}