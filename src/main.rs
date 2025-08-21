use bevy::prelude::*;
use bevy::window::WindowResolution;

mod components;
mod resources;
mod systems;
mod events;
mod enemy_types;
mod enemy_systems;
mod weapon_systems;
mod currency_systems;

use components::*;
use resources::*;
use systems::*;
use events::*;
use enemy_types::*;
use enemy_systems::*;
use weapon_systems::*;
use currency_systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Enhanced Shmup - Multi-Weapon Edition".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<InputState>()
        .init_resource::<EnemySpawner>()
        .init_resource::<GameScore>()
        .init_resource::<GameStarted>()
        .init_resource::<ShootingState>()
        .init_state::<GameState>()
        .add_event::<SpawnExplosion>()
        .add_event::<SpawnEnemy>()
        .add_event::<SpawnPowerUp>()
        .add_event::<SpawnParticles>()
        .add_event::<PlayerHit>()
        .add_systems(Startup, (
            setup_camera, 
            setup_background, 
            spawn_enhanced_player, 
            load_assets, 
            setup_enhanced_ui.after(load_assets), 
            load_high_scores,
            init_particle_pool,
        ))
        .add_systems(Update, (
            handle_pause_input,
            handle_input.run_if(in_state(GameState::Playing)),
            move_player.run_if(in_state(GameState::Playing)),
            enhanced_shooting_system.run_if(in_state(GameState::Playing)),
            spawn_enemies_enhanced.run_if(in_state(GameState::Playing)),
            spawn_powerups.run_if(in_state(GameState::Playing)),
            spawn_weapon_powerups.run_if(in_state(GameState::Playing)),
            spawn_upgrade_stations.run_if(in_state(GameState::Playing)),
            enemy_shooting.run_if(in_state(GameState::Playing)),
            turret_shooting.run_if(in_state(GameState::Playing)),
            move_enemies.run_if(in_state(GameState::Playing)),
            update_spawner_enemies.run_if(in_state(GameState::Playing)),
        ))
        .add_systems(Update, (            
            update_formations.run_if(in_state(GameState::Playing)),
            formation_coordination_system.run_if(in_state(GameState::Playing)),
            move_projectiles.run_if(in_state(GameState::Playing)),
            update_missiles.run_if(in_state(GameState::Playing)),
            update_laser_beams.run_if(in_state(GameState::Playing)),
            update_smart_bombs.run_if(in_state(GameState::Playing)),
            move_powerups.run_if(in_state(GameState::Playing)),
            move_currency.run_if(in_state(GameState::Playing)),
            enhanced_projectile_collisions.run_if(in_state(GameState::Playing)),
            currency_pickup_system.run_if(in_state(GameState::Playing)),
            weapon_powerup_collection.run_if(in_state(GameState::Playing)),
        ))
        .add_systems(Update, (                
            upgrade_station_interaction.run_if(in_state(GameState::Playing)),
            handle_powerup_collection.run_if(in_state(GameState::Playing)),
            update_player_effects.run_if(in_state(GameState::Playing)),
            update_temporary_weapon_effects.run_if(in_state(GameState::Playing)),
            update_explosions.run_if(in_state(GameState::Playing)),
            update_particles.run_if(in_state(GameState::Playing)),
            update_particle_emitters.run_if(in_state(GameState::Playing)),
            update_parallax.run_if(in_state(GameState::Playing)),
            cleanup_offscreen.run_if(in_state(GameState::Playing)),
            spawn_engine_particles.run_if(in_state(GameState::Playing)),
        ))
        .add_systems(Update, (
            spawn_explosion_system,
            spawn_enemy_system,
            spawn_powerup_system,
            spawn_particles_system,
            spawn_currency_on_death,
            handle_player_hit,
            update_health_bar,
            update_enhanced_ui,
            update_upgrade_ui,
            check_game_over,
            handle_restart_input,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::GameOver), (save_high_score_system, setup_game_over_ui).chain())
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
        .add_systems(OnEnter(GameState::Playing), reset_enhanced_game_state)
        .add_systems(OnEnter(GameState::Paused), setup_pause_ui)
        .add_systems(OnExit(GameState::Paused), cleanup_pause_ui)
        .add_systems(Update, (
            handle_restart_button,
        ).run_if(in_state(GameState::GameOver)))
        .run();
}

// Enhanced player spawning with weapon system
pub fn spawn_enhanced_player(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        WeaponSystem::default(),
        Currency { amount: 0 },
        PermanentUpgrades::default(),
        Collider { radius: 16.0 },
        Health(100),
        EngineTrail,
    ));
}

// Enhanced UI setup
pub fn setup_enhanced_ui(mut commands: Commands) {
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

    // Currency text
    commands.spawn((
        Text::new("Currency: 0¤"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.3)),
        CurrencyText,
    ));

    // Weapon info text
    commands.spawn((
        Text::new("Weapon: Basic"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(50.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.7, 1.0, 0.7)),
        WeaponText,
    ));

    // Smart bomb counter
    commands.spawn((
        Text::new("Smart Bombs: 3"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(250.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 1.0)),
        SmartBombText,
    ));

    // Controls help
    commands.spawn((
        Text::new("SPACE: Smart Bomb | Near Station: 1-9 to upgrade"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(100.0),
            ..default()
        },
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ControlsText,
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

// Enhanced UI update system
pub fn update_enhanced_ui(
    game_score: Res<GameScore>,
    player_query: Query<(&Player, &Currency, &WeaponSystem)>,
    mut currency_query: Query<&mut Text, (With<CurrencyText>, Without<WeaponText>, Without<SmartBombText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut weapon_query: Query<&mut Text, (With<WeaponText>, Without<CurrencyText>, Without<SmartBombText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut smart_bomb_query: Query<&mut Text, (With<SmartBombText>, Without<CurrencyText>, Without<WeaponText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<CurrencyText>, Without<WeaponText>, Without<SmartBombText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut high_score_query: Query<&mut Text, (With<HighScoreText>, Without<CurrencyText>, Without<WeaponText>, Without<SmartBombText>, Without<ScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut multiplier_query: Query<&mut Text, (With<MultiplierText>, Without<CurrencyText>, Without<WeaponText>, Without<SmartBombText>, Without<ScoreText>, Without<HighScoreText>, Without<LivesText>)>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<CurrencyText>, Without<WeaponText>, Without<SmartBombText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>)>,
) {
    if let Ok((player, currency, weapon_system)) = player_query.single() {
        // Update currency display
        if let Ok(mut currency_text) = currency_query.single_mut() {
            **currency_text = format!("Currency: {}¤", currency.amount);
        }
        
        // Update weapon display
        if let Ok(mut weapon_text) = weapon_query.single_mut() {
            let weapon_name = match &weapon_system.primary_weapon {
                WeaponType::Basic { .. } => "Basic",
                WeaponType::SpreadShot { .. } => "Spread Shot",
                WeaponType::Laser { .. } => "Laser",
                WeaponType::Missile { .. } => "Missiles",
                WeaponType::RapidFire { .. } => "Rapid Fire",
            };
            **weapon_text = format!("Weapon: {}", weapon_name);
        }
        
        // Update smart bomb counter
        if let Ok(mut bomb_text) = smart_bomb_query.single_mut() {
            **bomb_text = format!("Smart Bombs: {}", weapon_system.smart_bombs);
        }
        
        // Update lives
        if let Ok(mut lives_text) = lives_query.single_mut() {
            **lives_text = format!("Lives: {}", player.lives);
        }
    }
    
    // Update score
    if let Ok(mut score_text) = score_query.single_mut() {
        **score_text = format!("Score: {}", game_score.current);
    }
    
    // Update high score
    if let Ok(mut high_score_text) = high_score_query.single_mut() {
        let high_score = game_score.high_scores.first().unwrap_or(&0);
        **high_score_text = format!("High: {}", high_score);
    }

    // Update multiplier
    if let Ok(mut multiplier_text) = multiplier_query.single_mut() {
        if game_score.score_multiplier > 1.0 {
            **multiplier_text = format!("{}x ({:.1}s)", game_score.score_multiplier, game_score.multiplier_timer);
        } else {
            **multiplier_text = String::new();
        }
    }
}

// Enhanced game reset system
pub fn reset_enhanced_game_state(
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
    (powerup_query,weapon_powerup_query): (Query<Entity, With<PowerUp>>, Query<Entity, With<WeaponPowerUp>>),
    currency_entity_query: Query<Entity, (With<Currency>, Without<Player>)>,
    upgrade_station_query: Query<Entity, With<UpgradeStation>>,
    (particle_query, emitter_query): (Query<Entity, With<Particle>>,Query<Entity, With<ParticleEmitter>>),
    (laser_query, smart_bomb_query): (Query<Entity, With<LaserBeam>>, Query<Entity, With<SmartBombWave>>),
    (player_query, upgrade_ui_query) : (Query<Entity, With<Player>>, Query<Entity, With<UpgradeUI>>),
    assets: Option<Res<GameAssets>>,
) {
    if !game_started.0 {
        game_started.0 = true;
        return;
    }
    
    // Despawn all game entities
    for entity in enemy_query.iter()
        .chain(projectile_query.iter())
        .chain(explosion_query.iter())
        .chain(powerup_query.iter())
        .chain(weapon_powerup_query.iter())
        .chain(currency_entity_query.iter())
        .chain(upgrade_station_query.iter())
        .chain(particle_query.iter())
        .chain(emitter_query.iter())
        .chain(laser_query.iter())
        .chain(smart_bomb_query.iter())
        .chain(player_query.iter())
        .chain(upgrade_ui_query.iter()) {
        commands.entity(entity).despawn_recursive();
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
    
    // Respawn enhanced player
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
            WeaponSystem::default(),
            Currency { amount: 0 },
            PermanentUpgrades::default(),
            Collider { radius: 16.0 },
            Health(100),
            EngineTrail,
        ));
    }
}

// Additional UI component markers
#[derive(Component)]
pub struct CurrencyText;

#[derive(Component)]
pub struct WeaponText;

#[derive(Component)]
pub struct SmartBombText;

#[derive(Component)]
pub struct ControlsText;