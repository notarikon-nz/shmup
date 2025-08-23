use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
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
        Without<CellWallVisual>
    )>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y < -450.0 || transform.translation.y > 550.0 ||
           transform.translation.x < -750.0 || transform.translation.x > 750.0 {
            commands.entity(entity).despawn();
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
        commands.entity(entity).despawn();
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
        commands.entity(entity).despawn();
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

// Enhanced explosion system with organic effects
pub fn update_explosions(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut EnhancedExplosion, &mut Transform, &mut Sprite)>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        for (entity, mut explosion, mut transform, mut sprite) in explosion_query.iter_mut() {
            explosion.timer += time.delta_secs();
            
            if explosion.timer >= explosion.max_time {
                commands.entity(entity).despawn();
                continue;
            }
            
            let explosion_clone = explosion.clone();
            // Process each explosion layer
            for layer in &mut explosion.layers {
                if !layer.completed && explosion_clone.timer >= layer.delay {
                    let layer_progress = (explosion_clone.timer - layer.delay) / layer.duration;
                    
                    if layer_progress >= 1.0 {
                        layer.completed = true;
                        continue;
                    }
                    
                    match layer.phase {
                        ExplosionPhase::Shockwave => {
                            update_shockwave_layer(&mut commands, &assets, &transform, layer, layer_progress, &explosion_clone.explosion_type);
                        }
                        ExplosionPhase::CoreBlast => {
                            update_core_blast_layer(&mut commands, &assets, &transform, layer, layer_progress, explosion_clone.intensity);
                        }
                        ExplosionPhase::Debris => {
                            update_debris_layer(&mut commands, &assets, &transform, layer, layer_progress, &explosion_clone.explosion_type);
                        }
                        ExplosionPhase::Afterglow => {
                            update_afterglow_layer(&mut commands, &assets, &transform, layer, layer_progress);
                        }
                        ExplosionPhase::Membrane => {
                            update_membrane_layer(&mut commands, &assets, &transform, layer, layer_progress);
                        }
                    }
                }
            }
            
            // Update main explosion sprite opacity
            let global_progress = explosion.timer / explosion.max_time;
            sprite.color.set_alpha(0.8 * (1.0 - global_progress).powi(2));
        }
    }
}

// Create layered explosion based on type
fn create_explosion_layers(explosion_type: &ExplosionType, intensity: f32) -> Vec<ExplosionLayer> {
    match explosion_type {
        ExplosionType::Biological { toxin_release, membrane_rupture } => {
            let mut layers = vec![
                // Membrane rupture (immediate)
                ExplosionLayer {
                    phase: ExplosionPhase::Membrane,
                    delay: 0.0,
                    duration: 0.2,
                    particle_count: (25.0 * intensity) as u32,
                    color_start: Color::srgb(0.9, 1.0, 0.8),
                    color_end: Color::srgba(0.4, 0.8, 0.6, 0.0),
                    size_range: (2.0, 8.0),
                    velocity_range: (Vec2::new(-150.0, -150.0), Vec2::new(150.0, 150.0)),
                    completed: false,
                },
                // Core biological explosion
                ExplosionLayer {
                    phase: ExplosionPhase::CoreBlast,
                    delay: 0.05,
                    duration: 0.4,
                    particle_count: (40.0 * intensity) as u32,
                    color_start: Color::srgb(0.8, 0.9, 0.4),
                    color_end: Color::srgba(0.2, 0.6, 0.3, 0.0),
                    size_range: (1.0, 6.0),
                    velocity_range: (Vec2::new(-200.0, -200.0), Vec2::new(200.0, 200.0)),
                    completed: false,
                },
                // Cellular debris
                ExplosionLayer {
                    phase: ExplosionPhase::Debris,
                    delay: 0.1,
                    duration: 0.8,
                    particle_count: (15.0 * intensity) as u32,
                    color_start: Color::srgb(0.6, 0.8, 0.5),
                    color_end: Color::srgba(0.3, 0.5, 0.4, 0.0),
                    size_range: (0.5, 3.0),
                    velocity_range: (Vec2::new(-100.0, -50.0), Vec2::new(100.0, 50.0)),
                    completed: false,
                },
            ];
            
            if *toxin_release {
                layers.push(ExplosionLayer {
                    phase: ExplosionPhase::Afterglow,
                    delay: 0.3,
                    duration: 1.2,
                    particle_count: (8.0 * intensity) as u32,
                    color_start: Color::srgb(0.9, 0.4, 0.6),
                    color_end: Color::srgba(0.7, 0.3, 0.4, 0.0),
                    size_range: (3.0, 12.0),
                    velocity_range: (Vec2::new(-50.0, -25.0), Vec2::new(50.0, 25.0)),
                    completed: false,
                });
            }
            
            layers
        }
        
        ExplosionType::Chemical { ph_change, oxygen_release } => {
            vec![
                ExplosionLayer {
                    phase: ExplosionPhase::Shockwave,
                    delay: 0.0,
                    duration: 0.15,
                    particle_count: (30.0 * intensity) as u32,
                    color_start: if *ph_change < 0.0 { Color::srgb(1.0, 0.3, 0.3) } else { Color::srgb(0.3, 0.3, 1.0) },
                    color_end: Color::srgba(0.8, 0.8, 0.8, 0.0),
                    size_range: (1.0, 4.0),
                    velocity_range: (Vec2::new(-300.0, -300.0), Vec2::new(300.0, 300.0)),
                    completed: false,
                },
                ExplosionLayer {
                    phase: ExplosionPhase::Afterglow,
                    delay: 0.2,
                    duration: 2.0,
                    particle_count: (12.0 * intensity) as u32,
                    color_start: Color::srgb(0.7, 0.9, 0.8),
                    color_end: Color::srgba(0.3, 0.7, 0.6, 0.0),
                    size_range: (4.0, 16.0),
                    velocity_range: (Vec2::new(-30.0, -30.0), Vec2::new(30.0, 30.0)),
                    completed: false,
                },
            ]
        }
        
        _ => {
            vec![
                ExplosionLayer {
                    phase: ExplosionPhase::CoreBlast,
                    delay: 0.0,
                    duration: 0.5,
                    particle_count: (20.0 * intensity) as u32,
                    color_start: Color::srgb(1.0, 0.8, 0.4),
                    color_end: Color::srgba(1.0, 0.4, 0.2, 0.0),
                    size_range: (1.0, 5.0),
                    velocity_range: (Vec2::new(-180.0, -180.0), Vec2::new(180.0, 180.0)),
                    completed: false,
                },
            ]
        }
    }
}

// Layer update functions
fn update_shockwave_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
    explosion_type: &ExplosionType,
) {
    if progress < 0.1 { // Only spawn particles early in shockwave
        let ring_particles = 12;
        for i in 0..ring_particles {
            let angle = (i as f32 / ring_particles as f32) * std::f32::consts::TAU;
            let radius = 20.0 + progress * 100.0;
            let position = transform.translation + Vec3::new(
                angle.cos() * radius,
                angle.sin() * radius,
                0.1,
            );
            
            let velocity = Vec2::from_angle(angle) * 250.0;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: layer.color_start,
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                Transform::from_translation(position),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 0.3,
                    size: 3.0,
                    fade_rate: 3.0,
                    bioluminescent: matches!(explosion_type, ExplosionType::Biological { .. }),
                    drift_pattern: DriftPattern::Pulsing,
                },
            ));
        }
    }
}

fn update_core_blast_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
    intensity: f32,
) {
    if progress < 0.2 { // Spawn core particles early
        let count = (layer.particle_count as f32 * (1.0 - progress * 5.0)).max(0.0) as u32;
        
        for i in 0..count.min(8) { // Limit per frame
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU + progress * 10.0;
            let speed = 80.0 + progress * 120.0;
            let velocity = Vec2::from_angle(angle) * speed;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: layer.color_start,
                    custom_size: Some(Vec2::splat(4.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 0.6,
                    size: 4.0 * intensity,
                    fade_rate: 1.5,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Spiraling,
                },
                BioluminescentParticle {
                    base_color: layer.color_start,
                    pulse_frequency: 6.0,
                    pulse_intensity: 0.8,
                    organic_motion: OrganicMotion {
                        undulation_speed: 3.0,
                        response_to_current: 0.2,
                    },
                },
            ));
        }
    }
}

fn update_debris_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
    explosion_type: &ExplosionType,
) {
    if progress < 0.3 {
        let debris_color = match explosion_type {
            ExplosionType::Biological { .. } => Color::srgb(0.7, 0.8, 0.6),
            ExplosionType::Chemical { .. } => Color::srgb(0.8, 0.9, 0.5),
            _ => Color::srgb(0.6, 0.6, 0.6),
        };
        
        for i in 0..(layer.particle_count / 8).min(4) {
            let angle = (i as f32 * 1.7) + progress * 8.0;
            let distance = 25.0 + progress * 40.0;
            let velocity = Vec2::from_angle(angle) * (60.0 + progress * 80.0);
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: debris_color,
                    custom_size: Some(Vec2::splat(2.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation + Vec3::new(
                    angle.cos() * distance,
                    angle.sin() * distance,
                    0.0,
                )),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 1.5,
                    size: 2.0,
                    fade_rate: 0.8,
                    bioluminescent: false,
                    drift_pattern: DriftPattern::Brownian,
                },
            ));
        }
    }
}

fn update_afterglow_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
) {
    if progress < 0.4 {
        for i in 0..(layer.particle_count / 10).min(3) {
            let velocity = Vec2::new(
                (progress * 50.0 + i as f32 * 20.0).sin() * 30.0,
                (progress * 40.0 + i as f32 * 15.0).cos() * 25.0,
            );
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: layer.color_start,
                    custom_size: Some(Vec2::splat(6.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 3.0,
                    size: 6.0,
                    fade_rate: 0.4,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Floating,
                },
                BioluminescentParticle {
                    base_color: layer.color_start,
                    pulse_frequency: 1.0,
                    pulse_intensity: 0.6,
                    organic_motion: OrganicMotion {
                        undulation_speed: 1.5,
                        response_to_current: 0.9,
                    },
                },
            ));
        }
    }
}

fn update_membrane_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
) {
    if progress < 0.15 {
        // Membrane fragments with organic shapes
        for i in 0..6 {
            let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
            let fragment_size = 4.0 + (i % 3) as f32 * 2.0;
            let velocity = Vec2::from_angle(angle) * (120.0 + progress * 100.0);
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgb(0.8, 1.0, 0.9),
                    custom_size: Some(Vec2::splat(fragment_size)),
                    ..default()
                },
                Transform::from_translation(transform.translation + Vec3::new(
                    angle.cos() * 15.0,
                    angle.sin() * 15.0,
                    0.1,
                )),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 1.0,
                    size: fragment_size,
                    fade_rate: 1.2,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Floating,
                },
                BioluminescentParticle {
                    base_color: Color::srgb(0.8, 1.0, 0.9),
                    pulse_frequency: 4.0,
                    pulse_intensity: 0.4,
                    organic_motion: OrganicMotion {
                        undulation_speed: 2.5,
                        response_to_current: 0.8,
                    },
                },
            ));
        }
    }
}



// Enhanced particle emitter with biological particles
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
                
                let mut particle_commands = commands.spawn((
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
                        bioluminescent: config.organic_motion,
                        drift_pattern: if config.organic_motion { DriftPattern::Floating } else { DriftPattern::Brownian },
                    },
                ));
                
                // Add bioluminescent component if organic
                if config.organic_motion {
                    particle_commands.insert(BioluminescentParticle {
                        base_color: config.color_start,
                        pulse_frequency: 2.0 + rand_x * 2.0,
                        pulse_intensity: config.bioluminescence,
                        organic_motion: OrganicMotion {
                            undulation_speed: 1.0 + rand_y,
                            response_to_current: 0.6,
                        },
                    });
                }
                
                emitter.spawn_timer = 1.0 / emitter.spawn_rate;
            }
        }
    }
}

// Enhanced player hit system with biological feedback
pub fn handle_player_hit(
    mut commands: Commands,
    mut player_hit_events: EventReader<PlayerHit>,
    mut player_query: Query<(Entity, &mut Health, &mut Player, Option<&CellWallReinforcement>), With<Player>>,
    mut explosion_events: EventWriter<SpawnExplosion>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in player_hit_events.read() {
        if let Ok((player_entity, mut health, mut player, cell_wall)) = player_query.single_mut() {
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
                    // Cellular regeneration
                    health.0 = 100;
                    player.invincible_timer = 3.0;
                } else {
                    // Final cellular breakdown
                    commands.entity(player_entity).despawn();
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}

// Event Systems with biological enhancements
pub fn spawn_explosion_system(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut shake_events: EventWriter<AddScreenShake>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in explosion_events.read() {
            let explosion_type = match &event.enemy_type {
                Some(EnemyType::InfectedMacrophage) => ExplosionType::Biological { 
                    toxin_release: true, 
                    membrane_rupture: true 
                },
                Some(EnemyType::BiofilmColony) => ExplosionType::Chemical { 
                    ph_change: -1.5, 
                    oxygen_release: 0.3 
                },
                Some(EnemyType::AggressiveBacteria) => ExplosionType::Biological { 
                    toxin_release: true, 
                    membrane_rupture: false 
                },
                _ => ExplosionType::Standard,
            };
            
            let layers = create_explosion_layers(&explosion_type, event.intensity);
            let shake_amount = calculate_shake_amount(&explosion_type, event.intensity);
            
            shake_events.write(AddScreenShake { amount: shake_amount });
            
            // Spawn main explosion entity
            let explosion_entity = commands.spawn((
                Sprite {
                    image: assets.explosion_texture.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0), // Start transparent
                    ..default()
                },
                Transform::from_translation(event.position),
                EnhancedExplosion {
                    timer: 0.0,
                    max_time: 1.5,
                    intensity: event.intensity,
                    explosion_type: explosion_type.clone(),
                    layers,
                    light_id: None, // Will be set by lighting system
                },
            )).id();
            
            // Create initial shockwave
            spawn_shockwave(&mut commands, &assets, event.position, event.intensity, &explosion_type);
        }
    }
}

// Calculate screen shake based on explosion properties
fn calculate_shake_amount(explosion_type: &ExplosionType, intensity: f32) -> f32 {
    let base_shake = intensity * 0.3;
    
    match explosion_type {
        ExplosionType::Biological { membrane_rupture: true, .. } => base_shake * 1.5,
        ExplosionType::Chemical { .. } => base_shake * 1.2,
        ExplosionType::Electrical { .. } => base_shake * 0.8,
        ExplosionType::Thermal { .. } => base_shake * 1.3,
        _ => base_shake,
    }
}

// Spawn shockwave effect
fn spawn_shockwave(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    intensity: f32,
    explosion_type: &ExplosionType,
) {
    let ring_color = match explosion_type {
        ExplosionType::Biological { .. } => Color::srgb(0.4, 1.0, 0.8),
        ExplosionType::Chemical { .. } => Color::srgb(0.9, 0.9, 0.3),
        ExplosionType::Electrical { .. } => Color::srgb(0.3, 0.8, 1.0),
        _ => Color::srgb(1.0, 0.8, 0.4),
    };
    
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgba(ring_color.to_srgba().red, ring_color.to_srgba().green, ring_color.to_srgba().blue, 0.6),
            custom_size: Some(Vec2::splat(20.0)),
            ..default()
        },
        Transform::from_translation(position),
        MiniExplosion {
            timer: 0.0,
            max_time: 0.4,
            size: intensity,
        },
    ));
}


// Spawning Systems
pub fn spawn_enemies(
    mut commands: Commands,
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
        }
    }
}

pub fn spawn_powerup_system(
    mut commands: Commands,
    mut powerup_events: EventReader<SpawnPowerUp>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in powerup_events.read() {
            let (texture, color) = match &event.power_type {
                PowerUpType::CellularRegeneration { .. } => (assets.health_powerup_texture.clone(), Color::srgb(0.4, 1.0, 0.6)),
                PowerUpType::CellWall { .. } => (assets.shield_powerup_texture.clone(), Color::srgb(0.4, 1.0, 0.8)),
                PowerUpType::Flagella { .. } => (assets.speed_powerup_texture.clone(), Color::srgb(0.6, 0.9, 1.0)),
                PowerUpType::SymbioticBoost { .. } => (assets.multiplier_powerup_texture.clone(), Color::srgb(1.0, 0.8, 0.4)),
                PowerUpType::MitochondriaOvercharge { .. } => (assets.rapidfire_powerup_texture.clone(), Color::srgb(1.0, 0.6, 0.8)),
                PowerUpType::Photosynthesis { .. } => (assets.health_powerup_texture.clone(), Color::srgb(0.6, 1.0, 0.3)),
                PowerUpType::Chemotaxis { .. } => (assets.speed_powerup_texture.clone(), Color::srgb(0.8, 0.6, 1.0)),
                PowerUpType::Osmoregulation { .. } => (assets.shield_powerup_texture.clone(), Color::srgb(0.3, 0.8, 0.9)),
                PowerUpType::BinaryFission { .. } => (assets.rapidfire_powerup_texture.clone(), Color::srgb(1.0, 0.9, 0.3)),
            };
            
            commands.spawn((
                Sprite {
                    image: texture,
                    color,
                    ..default()
                },
                Transform::from_translation(event.position),
                PowerUp {
                    power_type: event.power_type.clone(),
                    bob_timer: 0.0,
                    bioluminescent_pulse: 0.0,
                },
                Collider { radius: 12.0 },
                BioluminescentParticle {
                    base_color: color,
                    pulse_frequency: 2.5,
                    pulse_intensity: 0.5,
                    organic_motion: OrganicMotion {
                        undulation_speed: 1.8,
                        response_to_current: 0.7,
                    },
                },
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
                let rand_x = (rand_seed * 12.9898).sin().abs().fract();
                let rand_y = (rand_seed * 78.233).sin().abs().fract();
                let rand_lifetime = (rand_seed * 35.456).sin().abs().fract();
                let rand_size = (rand_seed * 91.123).sin().abs().fract();
                
                let velocity = Vec2::new(
                    config.velocity_range.0.x + (config.velocity_range.1.x - config.velocity_range.0.x) * rand_x,
                    config.velocity_range.0.y + (config.velocity_range.1.y - config.velocity_range.0.y) * rand_y,
                );
                let lifetime = config.lifetime_range.0 + (config.lifetime_range.1 - config.lifetime_range.0) * rand_lifetime;
                let size = config.size_range.0 + (config.size_range.1 - config.size_range.0) * rand_size;
                
                let drift_pattern = if config.organic_motion {
                    match i % 4 {
                        0 => DriftPattern::Floating,
                        1 => DriftPattern::Pulsing,
                        2 => DriftPattern::Spiraling,
                        _ => DriftPattern::Brownian,
                    }
                } else {
                    DriftPattern::Brownian
                };
                
                let mut particle_commands = commands.spawn((
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
                        bioluminescent: config.organic_motion,
                        drift_pattern,
                    },
                ));
                
                // Add bioluminescent properties for organic particles
                if config.organic_motion {
                    particle_commands.insert(BioluminescentParticle {
                        base_color: config.color_start,
                        pulse_frequency: 1.5 + rand_x * 3.0,
                        pulse_intensity: config.bioluminescence,
                        organic_motion: OrganicMotion {
                            undulation_speed: 1.0 + rand_y * 2.0,
                            response_to_current: 0.6 + rand_size * 0.4,
                        },
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

    projectile_query: Query<(Entity, &Transform, &Collider, &Projectile)>,
    mut enemy_query: Query<(Entity, &Transform, &Collider, &mut Health, Option<&Enemy>), Without<Projectile>>,
    player_query: Query<(Entity, &Transform, &Collider, &Player, &CriticalHitStats), (With<Player>, Without<Enemy>)>,
) {
    if let Ok((player_entity, player_transform, player_collider, player, crit_stats)) = player_query.single() {
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
                
                commands.entity(proj_entity).despawn();
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
                        TextFont { font_size, ..default() },
                        TextColor(text_color),
                        Transform::from_translation(enemy_transform.translation + Vec3::new(0.0, 25.0, 1.0)),
                        DamageText {
                            timer: 1.5,
                            velocity: Vec2::new(0.0, 80.0),
                        },
                    ));
                    
                    commands.entity(proj_entity).despawn();
                    
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
                        commands.entity(enemy_entity).despawn();
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
}


// BEGIN FPS
pub fn fps_system(diagnostics: Res<DiagnosticsStore>, mut fps_text: Query<&mut Text, With<FpsText>>) {
    if let Ok(mut text) = fps_text.single_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **text = format!("FPS: {:.0}", value);
            }
        }
    }
}

#[derive(Component)]
pub struct FpsText;

// END FPS

// floating combat text
pub fn damage_text_system(
    mut commands: Commands,
    mut damage_query: Query<(Entity, &mut Transform, &mut DamageText, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut damage_text, mut text_color) in damage_query.iter_mut() {
        damage_text.timer -= time.delta_secs();
        transform.translation += damage_text.velocity.extend(0.0) * time.delta_secs();
        
        let alpha = damage_text.timer / 1.5;
        text_color.0 = Color::srgba(1.0, 0.3, 0.3, alpha);
        
        if damage_text.timer <= 0.0 {
            commands.entity(entity).despawn();
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
    
    // Update flashes with organic pulsing
    for (entity, mut flash, mut sprite) in flash_query.iter_mut() {
        flash.timer += time.delta_secs();
        
        if flash.timer >= flash.duration {
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<FlashEffect>();
        } else {
            let progress = flash.timer / flash.duration;
            // Organic flash curve - sharp peak, gentle falloff
            let flash_intensity = if progress < 0.3 {
                1.0 - (progress / 0.3)
            } else {
                ((1.0 - progress) / 0.7).powi(2)
            };
            
            sprite.color = flash.flash_color.mix(&flash.original_color, flash_intensity);
        }
    }
}

pub fn mini_explosion_system(
    mut commands: Commands,
    mut mini_explosion_query: Query<(Entity, &mut MiniExplosion, &mut Transform, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut explosion, mut transform, mut sprite) in mini_explosion_query.iter_mut() {
        explosion.timer += time.delta_secs();
        
        if explosion.timer >= explosion.max_time {
            commands.entity(entity).despawn();
            continue;
        }
        
        let progress = explosion.timer / explosion.max_time;
        let scale = explosion.size * (1.0 + progress * 2.0);
        transform.scale = Vec3::splat(scale);
        sprite.color.set_alpha(1.0 - progress);
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

// fn spawn_mini_explosion(
pub fn spawn_mini_explosions_on_collision(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    intensity: f32,
) {
    // Main mini explosion
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgb(1.0, 0.8, 0.4),
            custom_size: Some(Vec2::splat(8.0 * intensity)),
            ..default()
        },
        Transform::from_translation(position),
        MiniExplosion {
            timer: 0.0,
            max_time: 0.25,
            size: intensity,
        },
    ));
    
    // Spawn micro particles for detail
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
        let velocity = Vec2::from_angle(angle) * (80.0 + intensity * 40.0);
        
        commands.spawn((
            Sprite {
                image: assets.particle_texture.clone(),
                color: Color::srgb(1.0, 0.9, 0.6),
                custom_size: Some(Vec2::splat(2.0)),
                ..default()
            },
            Transform::from_translation(position),
            Particle {
                velocity,
                lifetime: 0.0,
                max_lifetime: 0.4,
                size: 2.0,
                fade_rate: 2.5,
                bioluminescent: true,
                drift_pattern: DriftPattern::Pulsing,
            },
        ));
    }
}

pub fn update_enhanced_mini_explosions(
    mut commands: Commands,
    mut mini_explosion_query: Query<(Entity, &mut MiniExplosion, &mut Transform, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut explosion, mut transform, mut sprite) in mini_explosion_query.iter_mut() {
        explosion.timer += time.delta_secs();
        
        if explosion.timer >= explosion.max_time {
            commands.entity(entity).despawn();
            continue;
        }
        
        let progress = explosion.timer / explosion.max_time;
        
        // Organic expansion with slight irregularity
        let base_scale = explosion.size * (1.0 + progress * 2.5);
        let organic_variation = (progress * 20.0).sin() * 0.1;
        let scale = base_scale * (1.0 + organic_variation);
        
        transform.scale = Vec3::splat(scale);
        
        // Color transition with organic fade
        let alpha = (1.0 - progress).powi(2);
        let color_shift = progress * 0.3;
        sprite.color = Color::srgba(
            1.0 - color_shift,
            0.8 + color_shift * 0.2,
            0.4 + color_shift * 0.4,
            alpha
        );
        
        // Slight organic rotation
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 2.0);
    }
}

pub fn explosion_lighting_system(
    mut commands: Commands,
    newly_spawned: Query<(Entity, &Transform, &EnhancedExplosion), Added<EnhancedExplosion>>,
    mut existing_explosions: Query<&mut EnhancedExplosion, Without<Transform>>,
) {
    // Spawn lights for new explosions
    for (explosion_entity, transform, explosion) in newly_spawned.iter() {
        let (light_color, intensity, radius) = match &explosion.explosion_type {
            ExplosionType::Biological { .. } => (Color::srgb(0.4, 1.0, 0.8), 1500.0, 120.0),
            ExplosionType::Chemical { .. } => (Color::srgb(0.9, 0.9, 0.3), 2000.0, 150.0),
            ExplosionType::Electrical { .. } => (Color::srgb(0.3, 0.8, 1.0), 1800.0, 100.0),
            _ => (Color::srgb(1.0, 0.6, 0.3), 1200.0, 100.0),
        };
        
        let light_entity = commands.spawn((
            ExplosionLight {
                color: light_color,
                intensity,
                radius,
                timer: 0.0,
                max_time: explosion.max_time,
                falloff: 2.0,
            },
            Transform::from_translation(transform.translation),
        )).id();
        
        // Link light to explosion - update in separate system or use commands
        commands.entity(explosion_entity).insert(LinkedExplosionLight(light_entity));
    }
}

// Add this component to link lights to explosions
#[derive(Component)]
pub struct LinkedExplosionLight(pub Entity);


pub fn update_explosion_lights(
    mut commands: Commands,
    mut light_query: Query<(Entity, &mut ExplosionLight)>,
    time: Res<Time>,
) {
    for (entity, mut light) in light_query.iter_mut() {
        light.timer += time.delta_secs();
        
        if light.timer >= light.max_time {
            commands.entity(entity).despawn();
        } else {
            // Organic light fade with multiple falloff curves
            let progress = light.timer / light.max_time;
            let fade_curve = if progress < 0.1 {
                // Quick bright flash
                1.0 - (progress / 0.1) * 0.3
            } else if progress < 0.4 {
                // Sustained glow
                0.7 - ((progress - 0.1) / 0.3) * 0.3
            } else {
                // Slow organic fade
                0.4 * (1.0 - ((progress - 0.4) / 0.6)).powi(3)
            };
            
            light.intensity = light.intensity * fade_curve;
            light.radius = light.radius * (0.8 + fade_curve * 0.2);
        }
    }
}



pub fn update_cell_wall_timer_ui(
    cell_wall_query: Query<&CellWallReinforcement>,
    mut timer_text_query: Query<&mut Text, With<CellWallTimerText>>,
) {
    if let Ok(mut text) = timer_text_query.single_mut() {
        if let Ok(cell_wall) = cell_wall_query.single() {
            let remaining = cell_wall.timer.max(0.0);
            let color_intensity = if remaining < 3.0 { "⚠️" } else { "🛡️" };
            **text = format!("{} Cell Wall: {:.1}s", color_intensity, remaining);
        } else {
            **text = String::new();
        }
    }
}

// TO DO


// Update your main collision system to use enhanced explosions
pub fn replace_explosion_events_with_enhanced(
    mut explosion_events: EventReader<SpawnExplosion>,
    mut enhanced_explosion_events: EventWriter<SpawnEnhancedExplosion>,
) {
    for event in explosion_events.read() {
        enhanced_explosion_events.write(SpawnEnhancedExplosion {
            position: event.position,
            intensity: event.intensity,
            explosion_type: event.enemy_type.as_ref().map(|e| {
                // Convert enemy type to explosion type
                match e {
                    EnemyType::InfectedMacrophage => ExplosionType::Biological { 
                        toxin_release: true, 
                        membrane_rupture: true 
                    },
                    EnemyType::BiofilmColony => ExplosionType::Chemical { 
                        ph_change: -1.5, 
                        oxygen_release: 0.3 
                    },
                    _ => ExplosionType::Standard,
                }
            }).unwrap_or(ExplosionType::Standard),
        });
    }
}