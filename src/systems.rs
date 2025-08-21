use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::render::camera::ScalingMode;
use std::f32::consts::TAU;
use bevy::time::*;

use crate::components::*;
use crate::resources::*;
use crate::events::*;

// Setup Systems
pub fn startup_debug() {
    info!("=== STARTUP SYSTEMS RUNNING ===");
}

// Setup Systems
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
            lives: 3,
            invincible_timer: 0.0,
        },
        Collider { radius: 16.0 },
        Health(100),
        EngineTrail,
    ));
}

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        player_texture: asset_server.load("textures/player.png"),
        enemy_texture: asset_server.load("textures/enemy.png"),
        projectile_texture: asset_server.load("textures/bullet.png"),
        explosion_texture: asset_server.load("textures/explosion.png"),
        particle_texture: asset_server.load("textures/particle.png"),
        health_powerup_texture: asset_server.load("textures/health_powerup.png"),
        shield_powerup_texture: asset_server.load("textures/shield_powerup.png"),
        speed_powerup_texture: asset_server.load("textures/speed_powerup.png"),
        multiplier_powerup_texture: asset_server.load("textures/multiplier_powerup.png"),
        rapidfire_powerup_texture: asset_server.load("textures/rapidfire_powerup.png"),
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

pub fn setup_ui(mut commands: Commands) {
    // Health bar background
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(60.0),
            width: Val::Px(204.0),
            height: Val::Px(24.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        BorderColor(Color::srgb(0.8, 0.8, 0.8)),
        HealthBar,
    ));
    
    // Health bar fill
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(22.0),
            bottom: Val::Px(62.0),
            width: Val::Px(200.0),
            height: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
        HealthBarFill,
    ));

    // Lives text
    commands.spawn((
        Text::new("Lives: 3"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::WHITE),
        LivesText,
    ));

    // Score text
    commands.spawn((
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        ScoreText,
    ));
    
    // High score text
    commands.spawn((
        Text::new("High: 0"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(50.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        HighScoreText,
    ));

    // Multiplier text
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(80.0),
            ..default()
        },
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        MultiplierText,
    ));
}


pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
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
        
        let (ai_type, enemy_type) = if enemy_spawner.wave_timer < 20.0 {
            (EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }, EnemyType::Basic)
        } else if enemy_spawner.wave_timer < 40.0 {
            (EnemyAI::Sine { amplitude: 100.0, frequency: 2.0, phase: 0.0 }, EnemyType::Fast)
        } else if enemy_spawner.wave_timer < 60.0 {
            if enemy_spawner.enemies_spawned % 2 == 0 {
                (EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }, EnemyType::Heavy)
            } else {
                (EnemyAI::Sine { amplitude: 150.0, frequency: 3.0, phase: 0.0 }, EnemyType::Fast)
            }
        } else {
            if enemy_spawner.enemies_spawned % 10 == 0 {
                (EnemyAI::MiniBoss { pattern: 0, timer: 0.0 }, EnemyType::Boss)
            } else {
                (EnemyAI::Sine { amplitude: 200.0, frequency: 4.0, phase: 0.0 }, EnemyType::Fast)
            }
        };
        
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(spawn_x, 400.0, 0.0),
            ai_type,
            enemy_type,
        });
        
        enemy_spawner.enemies_spawned += 1;
        
        let spawn_rate = (2.0 - (enemy_spawner.wave_timer * 0.02)).max(0.3);
        enemy_spawner.spawn_timer = spawn_rate;
    }
}

pub fn spawn_projectiles(
    mut commands: Commands,
    input_state: Res<InputState>,
    mut player_query: Query<&Transform, With<Player>>,
    assets: Option<Res<GameAssets>>,
    shooting_state: Res<ShootingState>,
    rapid_fire_query: Query<&RapidFire>,
    time: Res<Time>,
    mut shoot_timer: Local<f32>,
) {
    if let Some(assets) = assets {
        *shoot_timer -= time.delta_secs();
        
        let rate_multiplier = rapid_fire_query.iter().next().map_or(1.0, |rf| rf.rate_multiplier);
        let shoot_rate = shooting_state.base_rate / rate_multiplier;
        
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

// Particle Systems
pub fn spawn_engine_particles(
    mut commands: Commands,
    mut particle_events: EventWriter<SpawnParticles>,
    player_query: Query<&Transform, With<EngineTrail>>,
    input_state: Res<InputState>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    if assets.is_some() {
        *spawn_timer -= time.delta_secs();
        
        if *spawn_timer <= 0.0 {
            for transform in player_query.iter() {
                let intensity = input_state.movement.length().max(0.3);
                
                particle_events.write(SpawnParticles {
                    position: transform.translation + Vec3::new(0.0, -20.0, -0.1),
                    count: (intensity * 8.0) as u32,
                    config: ParticleConfig {
                        color_start: Color::srgb(0.2, 0.6, 1.0),
                        color_end: Color::srgba(0.8, 0.9, 1.0, 0.0),
                        velocity_range: (
                            Vec2::new(-30.0, -150.0),
                            Vec2::new(30.0, -50.0)
                        ),
                        lifetime_range: (0.2, 0.6),
                        size_range: (1.0, 3.0),
                        gravity: Vec2::ZERO,
                    },
                });
            }
            
            *spawn_timer = 0.05;
        }
    }
}

pub fn update_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite) in particle_query.iter_mut() {
        particle.lifetime += time.delta_secs();
        
        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Update position
        transform.translation += particle.velocity.extend(0.0) * time.delta_secs();
        
        // Apply gravity
        particle.velocity += Vec2::new(0.0, -100.0) * time.delta_secs();
        
        // Fade out
        let progress = particle.lifetime / particle.max_lifetime;
        let alpha = 1.0 - progress;
        sprite.color.set_alpha(alpha * particle.fade_rate);
        
        // Shrink
        let scale = particle.size * (1.0 - progress * 0.5);
        transform.scale = Vec3::splat(scale);
    }
}

pub fn update_particle_emitters(
    mut commands: Commands,
    mut emitter_query: Query<(&Transform, &mut ParticleEmitter)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        for (transform, mut emitter) in emitter_query.iter_mut() {
            if !emitter.active { continue; }
            
            emitter.spawn_timer -= time.delta_secs();
            
            if emitter.spawn_timer <= 0.0 {
                let config = &emitter.particle_config;
                let rand_x = (time.elapsed_secs() * 1234.56).fract();
                let rand_y = (time.elapsed_secs() * 5678.90).fract();
                let rand_lifetime = (time.elapsed_secs() * 9012.34).fract();
                let rand_size = (time.elapsed_secs() * 3456.78).fract();
                
                let velocity = Vec2::new(
                    config.velocity_range.0.x + (config.velocity_range.1.x - config.velocity_range.0.x) * rand_x,
                    config.velocity_range.0.y + (config.velocity_range.1.y - config.velocity_range.0.y) * rand_y,
                );
                let lifetime = config.lifetime_range.0 + (config.lifetime_range.1 - config.lifetime_range.0) * rand_lifetime;
                let size = config.size_range.0 + (config.size_range.1 - config.size_range.0) * rand_size;
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: config.color_start,
                        ..default()
                    },
                    Transform::from_translation(transform.translation).with_scale(Vec3::splat(size)),
                    Particle {
                        velocity,
                        lifetime: 0.0,
                        max_lifetime: lifetime,
                        size,
                        fade_rate: 1.0,
                    },
                ));
                
                emitter.spawn_timer = 1.0 / emitter.spawn_rate;
            }
        }
    }
}

// COLLISION SYSTEM
pub fn handle_collisions(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Collider, &Projectile)>,
    mut enemy_query: Query<(Entity, &Transform, &Collider, &mut Health, &Enemy), (With<Enemy>, Without<Projectile>, Without<Player>)>,
    player_query: Query<(Entity, &Transform, &Collider, &Player), (With<Player>, Without<Enemy>, Without<Projectile>)>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    mut particle_events: EventWriter<SpawnParticles>,
    mut player_hit_events: EventWriter<PlayerHit>,
    mut game_score: ResMut<GameScore>,
) {
    // Projectile vs Enemy collisions
    for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
        if !projectile.friendly { continue; }
        
        for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health, enemy) in enemy_query.iter_mut() {
            let distance = proj_transform.translation.distance(enemy_transform.translation);
            if distance < proj_collider.radius + enemy_collider.radius {
                enemy_health.0 -= projectile.damage;
                commands.entity(proj_entity).despawn();
                
                // Hit sparks
                particle_events.write(SpawnParticles {
                    position: proj_transform.translation,
                    count: 6,
                    config: ParticleConfig {
                        color_start: Color::srgb(1.0, 0.8, 0.2),
                        color_end: Color::srgba(1.0, 0.4, 0.0, 0.0),
                        velocity_range: (Vec2::new(-80.0, -80.0), Vec2::new(80.0, 80.0)),
                        lifetime_range: (0.1, 0.3),
                        size_range: (1.0, 2.0),
                        gravity: Vec2::ZERO,
                    },
                });
                
                if enemy_health.0 <= 0 {
                    commands.entity(enemy_entity).despawn();
                    
                    let points = match enemy.enemy_type {
                        EnemyType::Boss => 1000,
                        EnemyType::Heavy => 300,
                        EnemyType::Fast => 200,
                        EnemyType::Basic => 100,
                    };
                    
                    let multiplier = game_score.score_multiplier.max(1.0);
                    game_score.current += (points as f32 * multiplier) as u32;
                    
                    explosion_events.write(SpawnExplosion {
                        position: enemy_transform.translation,
                        intensity: match enemy.enemy_type {
                            EnemyType::Boss => 2.0,
                            EnemyType::Heavy => 1.5,
                            _ => 1.0,
                        },
                        enemy_type: Some(enemy.enemy_type.clone()),
                    });
                }
                break;
            }
        }
    }
    
    // Enemy projectiles vs Player
    for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
        if projectile.friendly { continue; }
        
        if let Ok((_, player_transform, player_collider, _)) = player_query.single() {
            let distance = proj_transform.translation.distance(player_transform.translation);
            if distance < proj_collider.radius + player_collider.radius {
                commands.entity(proj_entity).despawn();
                
                player_hit_events.write(PlayerHit {
                    position: player_transform.translation,
                    damage: projectile.damage,
                });
            }
        }
    }
    
    // Enemy vs Player direct collision
    if let Ok((_, player_transform, player_collider, _)) = player_query.single() {
        for (enemy_entity, enemy_transform, enemy_collider, mut enemy_health, _) in enemy_query.iter_mut() {
            let distance = player_transform.translation.distance(enemy_transform.translation);
            if distance < player_collider.radius + enemy_collider.radius {
                // Player takes damage
                player_hit_events.write(PlayerHit {
                    position: player_transform.translation,
                    damage: 25,
                });
                
                // Enemy also takes damage/dies from collision
                enemy_health.0 -= 50;
                
                explosion_events.write(SpawnExplosion {
                    position: enemy_transform.translation,
                    intensity: 1.2,
                    enemy_type: None,
                });
                
                if enemy_health.0 <= 0 {
                    commands.entity(enemy_entity).despawn();
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
        
        let scale = 1.0 + progress * explosion.intensity * 2.0;
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
    query: Query<(Entity, &Transform), (
        Without<Player>, 
        Without<ParallaxLayer>, 
        Without<HealthBarFill>, 
        Without<ScoreText>, 
        Without<HighScoreText>, 
        Without<HealthBar>,
        Without<LivesText>,
        Without<MultiplierText>
    )>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y < -450.0 || transform.translation.y > 550.0 ||
           transform.translation.x < -750.0 || transform.translation.x > 750.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Event Systems
pub fn spawn_explosion_system(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut particle_events: EventWriter<SpawnParticles>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in explosion_events.read() {
            let (color, particle_count, size_mult) = match &event.enemy_type {
                Some(EnemyType::Boss) => (Color::srgb(1.0, 0.2, 0.2), 30, 1.5),
                Some(EnemyType::Heavy) => (Color::srgb(1.0, 0.6, 0.2), 20, 1.2),
                Some(EnemyType::Fast) => (Color::srgb(0.2, 0.8, 1.0), 15, 1.0),
                _ => (Color::srgb(1.0, 0.8, 0.4), 12, 1.0),
            };

            commands.spawn((
                Sprite {
                    image: assets.explosion_texture.clone(),
                    color,
                    ..default()
                },
                Transform::from_translation(event.position),
                Explosion {
                    timer: 0.0,
                    max_time: 0.6 * size_mult,
                    intensity: event.intensity * size_mult,
                },
            ));

            // Explosion particles
            particle_events.write(SpawnParticles {
                position: event.position,
                count: particle_count,
                config: ParticleConfig {
                    color_start: color,
                    color_end: Color::srgba(color.to_srgba().red, color.to_srgba().green * 0.5, 0.0, 0.0),
                    velocity_range: (Vec2::new(-200.0, -200.0), Vec2::new(200.0, 200.0)),
                    lifetime_range: (0.3, 1.0),
                    size_range: (2.0 * size_mult, 8.0 * size_mult),
                    gravity: Vec2::new(0.0, -50.0),
                },
            });
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
            let (health, size, color) = match &event.enemy_type {
                EnemyType::Boss => (100, 30.0, Color::srgb(1.0, 0.3, 0.3)),
                EnemyType::Heavy => (50, 20.0, Color::srgb(0.8, 0.8, 0.3)),
                EnemyType::Fast => (15, 12.0, Color::srgb(0.3, 0.8, 1.0)),
                EnemyType::Basic => (20, 15.0, Color::WHITE),
            };
            
            commands.spawn((
                Sprite {
                    image: assets.enemy_texture.clone(),
                    color,
                    ..default()
                },
                Transform::from_translation(event.position),
                Enemy {
                    ai_type: event.ai_type.clone(),
                    health,
                    speed: 150.0,
                    enemy_type: event.enemy_type.clone(),
                },
                Collider { radius: size },
                Health(health),
            ));
        }
    }
}

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
        if let Ok(mut fill_node) = health_fill_query.single_mut() {
            fill_node.width = Val::Px(0.0);
        }
    }
}

pub fn update_score_display(
    game_score: Res<GameScore>,
    player_query: Query<&Player>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut high_score_query: Query<&mut Text, (With<HighScoreText>, Without<ScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut multiplier_query: Query<&mut Text, (With<MultiplierText>, Without<ScoreText>, Without<HighScoreText>, Without<LivesText>)>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>)>,
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

    // Update multiplier display
    if let Ok(mut multiplier_text) = multiplier_query.single_mut() {
        if game_score.score_multiplier > 1.0 {
            **multiplier_text = format!("{}x ({:.1}s)", game_score.score_multiplier, game_score.multiplier_timer);
        } else {
            **multiplier_text = String::new();
        }
    }

    // Update lives display
    if let Ok(mut lives_text) = lives_query.single_mut() {
        if let Ok(player) = player_query.single() {
            **lives_text = format!("Lives: {}", player.lives);
        } else {
            **lives_text = "Lives: 0".to_string();
        }
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
                PowerUpType::Shield { .. } => assets.shield_powerup_texture.clone(),
                PowerUpType::Speed { .. } => assets.speed_powerup_texture.clone(),
                PowerUpType::Multiplier { .. } => assets.multiplier_powerup_texture.clone(),
                PowerUpType::RapidFire { .. } => assets.rapidfire_powerup_texture.clone(),
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

pub fn spawn_particles_system(
    mut commands: Commands,
    mut particle_events: EventReader<SpawnParticles>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        for event in particle_events.read() {
            for i in 0..event.count {
                let config = &event.config;
                let rand_seed = time.elapsed_secs() * 1000.0 + i as f32;
                let rand_x = (rand_seed * 12.9898).sin() * 43758.5453;
                let rand_y = (rand_seed * 78.233).sin() * 43758.5453;
                let rand_lifetime = (rand_seed * 35.456).sin() * 43758.5453;
                let rand_size = (rand_seed * 91.123).sin() * 43758.5453;
                
                let velocity = Vec2::new(
                    config.velocity_range.0.x + (config.velocity_range.1.x - config.velocity_range.0.x) * rand_x.fract().abs(),
                    config.velocity_range.0.y + (config.velocity_range.1.y - config.velocity_range.0.y) * rand_y.fract().abs(),
                );
                let lifetime = config.lifetime_range.0 + (config.lifetime_range.1 - config.lifetime_range.0) * rand_lifetime.fract().abs();
                let size = config.size_range.0 + (config.size_range.1 - config.size_range.0) * rand_size.fract().abs();
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: config.color_start,
                        ..default()
                    },
                    Transform::from_translation(event.position).with_scale(Vec3::splat(size)),
                    Particle {
                        velocity,
                        lifetime: 0.0,
                        max_lifetime: lifetime,
                        size,
                        fade_rate: 1.0,
                    },
                ));
            }
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
    
    if enemy_spawner.powerup_timer <= 0.0 {
        let x_position = (time.elapsed_secs() * 50.0).sin() * 300.0;
        
        let power_type = match (time.elapsed_secs() as u32 / 15) % 5 {
            0 => PowerUpType::Health { amount: 25 },
            1 => PowerUpType::Shield { duration: 10.0 },
            2 => PowerUpType::Speed { multiplier: 1.5, duration: 8.0 },
            3 => PowerUpType::Multiplier { multiplier: 2.0, duration: 15.0 },
            _ => PowerUpType::RapidFire { rate_multiplier: 2.0, duration: 10.0 },
        };
        
        spawn_events.write(SpawnPowerUp {
            position: Vec3::new(x_position, 400.0, 0.0),
            power_type,
        });
        
        enemy_spawner.powerup_timer = 12.0;
    }
}

pub fn handle_player_hit(
    mut commands: Commands,
    mut player_hit_events: EventReader<PlayerHit>,
    mut player_query: Query<(Entity, &mut Health, &mut Player, Option<&Shield>), With<Player>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in player_hit_events.read() {
        if let Ok((player_entity, mut health, mut player, shield)) = player_query.single_mut() {
            // Skip damage if shielded or invincible
            if shield.is_some() || player.invincible_timer > 0.0 {
                continue;
            }

            health.0 -= event.damage;
            player.invincible_timer = 1.0; // 1 second invincibility

            explosion_events.write(SpawnExplosion {
                position: event.position,
                intensity: 0.8,
                enemy_type: None,
            });

            if health.0 <= 0 {
                player.lives -= 1;
                
                if player.lives > 0 {
                    // Respawn with full health
                    health.0 = 100;
                    player.invincible_timer = 3.0; // Longer invincibility after respawn
                } else {
                    // Game over
                    commands.entity(player_entity).despawn();
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}


pub fn handle_powerup_collection(
    mut commands: Commands,
    powerup_query: Query<(Entity, &Transform, &Collider, &PowerUp)>,
    mut player_query: Query<(Entity, &Transform, &Collider, &mut Health), With<Player>>,
) {
    if let Ok((player_entity, player_transform, player_collider, mut player_health)) = player_query.single_mut() {
        for (powerup_entity, powerup_transform, powerup_collider, powerup) in powerup_query.iter() {
            let distance = player_transform.translation.distance(powerup_transform.translation);
            if distance < player_collider.radius + powerup_collider.radius {
                match &powerup.power_type {
                    PowerUpType::Health { amount } => {
                        player_health.0 = (player_health.0 + amount).min(100);
                    }
                    PowerUpType::Shield { duration } => {
                        commands.entity(player_entity).insert(Shield {
                            timer: *duration,
                            alpha_timer: 0.0,
                        });
                    }
                    PowerUpType::Speed { multiplier, duration } => {
                        commands.entity(player_entity).insert(SpeedBoost {
                            timer: *duration,
                            multiplier: *multiplier,
                        });
                    }
                    PowerUpType::Multiplier { multiplier, duration } => {
                        commands.entity(player_entity).insert(ScoreMultiplier {
                            timer: *duration,
                            multiplier: *multiplier,
                        });
                    }
                    PowerUpType::RapidFire { rate_multiplier, duration } => {
                        commands.entity(player_entity).insert(RapidFire {
                            timer: *duration,
                            rate_multiplier: *rate_multiplier,
                        });
                    }
                }
                
                commands.entity(powerup_entity).despawn();
            }
        }
    }
}

pub fn update_player_effects(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player), With<Player>>,
    mut shield_query: Query<(Entity, &mut Shield)>,
    mut speed_query: Query<(Entity, &mut SpeedBoost)>,
    mut multiplier_query: Query<(Entity, &mut ScoreMultiplier)>,
    mut rapid_fire_query: Query<(Entity, &mut RapidFire)>,
    mut game_score: ResMut<GameScore>,
    time: Res<Time>,
) {
    if let Ok((_, mut player)) = player_query.single_mut() {
        player.invincible_timer = (player.invincible_timer - time.delta_secs()).max(0.0);
    }

    // Update shield
    for (entity, mut shield) in shield_query.iter_mut() {
        shield.timer -= time.delta_secs();
        shield.alpha_timer += time.delta_secs();
        
        if shield.timer <= 0.0 {
            commands.entity(entity).remove::<Shield>();
        }
    }

    // Update speed boost
    for (entity, mut speed) in speed_query.iter_mut() {
        speed.timer -= time.delta_secs();
        
        if speed.timer <= 0.0 {
            commands.entity(entity).remove::<SpeedBoost>();
        }
    }

    // Update score multiplier
    game_score.score_multiplier = 1.0;
    for (entity, mut multiplier) in multiplier_query.iter_mut() {
        multiplier.timer -= time.delta_secs();
        game_score.score_multiplier = multiplier.multiplier;
        game_score.multiplier_timer = multiplier.timer;
        
        if multiplier.timer <= 0.0 {
            commands.entity(entity).remove::<ScoreMultiplier>();
        }
    }

    // Update rapid fire
    for (entity, mut rapid_fire) in rapid_fire_query.iter_mut() {
        rapid_fire.timer -= time.delta_secs();
        
        if rapid_fire.timer <= 0.0 {
            commands.entity(entity).remove::<RapidFire>();
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
    player_query: Query<(Entity, &Health, &Transform, &Player), With<Player>>,
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
            
            commands.entity(player_entity).despawn();
            next_state.set(GameState::GameOver);
        }
    }
}

pub fn setup_game_over_ui(
    mut commands: Commands,
    mut game_score: ResMut<GameScore>,
) {
    save_high_score(&mut game_score);
    
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
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::srgb(1.0, 0.3, 0.3)),
            GameOverText,
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new(format!("Final Score: {}", game_score.current)),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
            FinalScoreText,
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        
        let high_score = game_score.high_scores.first().unwrap_or(&0);
        parent.spawn((
            Text::new(format!("High Score: {}", high_score)),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
        ));
        
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
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
        
        parent.spawn((
            Text::new("Press R to restart or click button above"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

pub fn cleanup_game_over_ui(
    mut commands: Commands,
    game_over_query: Query<Entity, With<GameOverUI>>,
) {
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
    mut shooting_state: ResMut<ShootingState>,
    // Despawn all game entities
    enemy_query: Query<Entity, With<Enemy>>,
    projectile_query: Query<Entity, With<Projectile>>,
    explosion_query: Query<Entity, With<Explosion>>,
    powerup_query: Query<Entity, With<PowerUp>>,
    particle_query: Query<Entity, With<Particle>>,
    emitter_query: Query<Entity, With<ParticleEmitter>>,
    player_query: Query<Entity, With<Player>>,
    assets: Option<Res<GameAssets>>,
) {
    if !game_started.0 {
        game_started.0 = true;
        return;
    }
    
    // Despawn all game entities
    for entity in enemy_query.iter().chain(projectile_query.iter())
        .chain(explosion_query.iter()).chain(powerup_query.iter())
        .chain(particle_query.iter()).chain(emitter_query.iter())
        .chain(player_query.iter()) {
        commands.entity(entity).despawn();
    }
    
    // Reset resources
    game_score.current = 0;
    game_score.score_multiplier = 1.0;
    game_score.multiplier_timer = 0.0;
    enemy_spawner.spawn_timer = 2.0;
    enemy_spawner.wave_timer = 0.0;
    enemy_spawner.enemies_spawned = 0;
    enemy_spawner.powerup_timer = 12.0;
    input_state.shoot_timer = 0.0;
    shooting_state.rate_multiplier = 1.0;
    
    // Respawn player
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
                lives: 3,
                invincible_timer: 3.0,
            },
            Collider { radius: 16.0 },
            Health(100),
            EngineTrail,
        ));
    }
}

// Pause System
pub fn setup_pause_ui(mut commands: Commands) {
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        PauseOverlay,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("PAUSED"),
            TextFont { font_size: 64.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new("Press ESC to resume"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

pub fn cleanup_pause_ui(
    mut commands: Commands,
    pause_query: Query<Entity, With<PauseOverlay>>,
) {
    for entity in pause_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

