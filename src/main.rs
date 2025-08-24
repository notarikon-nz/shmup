use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::sprite::Anchor;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin};

mod components;
mod resources;
mod systems;
mod events;
mod enemy_types;
mod enemy_systems;
mod weapon_systems;
mod currency_systems;
mod biological_systems; 
mod audio; 
mod lighting;
mod explosions;
mod particles;
mod tidal_mechanics;
mod high_scores;
mod powerup_systems;

use components::*;
use resources::*;
use systems::*;
use events::*;
use enemy_types::*;
use enemy_systems::*;
use weapon_systems::*;
use currency_systems::*;
use biological_systems::*; 
use audio::*;
use lighting::*;
use explosions::*;
use particles::*;
use tidal_mechanics::*;
use high_scores::*;
use powerup_systems::{spawn_powerup_system};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cosmic Tidal Pool".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.05, 0.15, 0.25))) // Deep water blue-black
        .init_resource::<InputState>()
        .init_resource::<EnemySpawner>()
        .init_resource::<GameScore>()
        .init_resource::<GameStarted>()
        .init_resource::<ShootingState>()
        .init_resource::<FluidEnvironment>()
        .init_resource::<ChemicalEnvironment>()
        .init_resource::<TidalPoolPhysics>()
        .init_resource::<BioluminescenceManager>()
        .init_resource::<EcosystemState>()
        .init_resource::<ScreenShakeResource>()
        .init_resource::<TidalState>()

        .init_state::<GameState>()
        .add_event::<SpawnExplosion>()
        .add_event::<SpawnEnemy>()
        .add_event::<SpawnPowerUp>()
        .add_event::<SpawnParticles>()
        .add_event::<PlayerHit>()
        .add_event::<AddScreenShake>()
        .add_event::<EnemyHit>()
        .add_event::<SpawnEnhancedExplosion>()
        .add_event::<TidalEvent>()

        .add_systems(Startup, (
            setup_camera, 
            setup_biological_background, 
            spawn_biological_player, 
            load_biological_assets, 
            setup_biological_ui.after(load_biological_assets), 
            load_high_scores_from_file,
            init_particle_pool,
            init_fluid_environment, 
            init_chemical_zones,
            init_current_generator,
            // init_tidal_state, // do we need this?
            start_ambient_music.after(load_biological_assets),
        ))
        .add_systems(Update, fps_system)
        // FIXED: Separate systems into different Update groups to avoid query conflicts
        .add_systems(Update, (
            audio_system,
            handle_pause_input,
        ))
        .add_systems(Update, (
            handle_input,
            biological_movement_system, 
            enhanced_shooting_system,
            spawn_enemies,
            spawn_biological_powerups, 
            spawn_evolution_powerups, 

            link_symbiotic_pairs,
            spawn_evolution_chambers, 

            // environmental
            corrupted_coral_system,
        ).run_if(in_state(GameState::Playing)))

        // Improved Enemy AI 
        .add_systems(Update, (

            // predator_prey_system, // accesses component(s) Transform in a way that conflicts with a previous system parameter
            adaptive_difficulty_system,
            ecosystem_balance_system,
            chemical_trail_system,
        ).run_if(in_state(GameState::Playing)))

        .add_systems(Update, (
            update_cell_wall_timer_ui,
            update_evolution_ui,
            update_tidal_ui,

            spawn_extra_life_powerups,
            extra_life_collection_system,
            update_dynamic_lights,
            render_light_effects,
        ).run_if(in_state(GameState::Playing)))

        // Fifth Update group - effect and cleanup systems
        .add_systems(Update, (                
            update_biological_effects,
            update_temporary_evolution_effects,
            consolidated_explosion_system, 

            unified_particle_system, 

            update_parallax,
            cleanup_offscreen,
            spawn_bioluminescent_trail,
        ).run_if(in_state(GameState::Playing)))

        // Second Update group - projectile and movement systems
        .add_systems(Update, (
            move_projectiles,

            unified_weapon_update_system,
           
            move_biological_powerups,
            move_atp,
            collect_atp_with_energy_transfer,

        ).run_if(in_state(GameState::Playing)))

        .add_systems(Update, (
            update_cell_wall_timer,
            enemy_flash_system,
            screen_shake_system,
        ).run_if(in_state(GameState::Playing)))

        // Third Update group - enemy systems (separate from projectile systems)
        .add_systems(Update, (
            enemy_shooting,
            turret_shooting,
            move_enemies,
            update_spawner_enemies,
            update_formations,
            formation_coordination_system,
            procedural_colony_spawning,
        ).run_if(in_state(GameState::Playing)))
        // Fourth Update group - collision and interaction systems
        .add_systems(Update, (            
            collision_system, 
            atp_pickup_system, 
            evolution_powerup_collection,
            evolution_chamber_interaction,
            handle_biological_powerup_collection,
        ).run_if(in_state(GameState::Playing)))

        // Sixth Update group - biological environment systems
        .add_systems(Update, (
            damage_text_system,
            fluid_dynamics_system,
            chemical_environment_system,
            update_current_field,
            organic_ai_system,
            generate_procedural_currents,
            cell_division_system,
            symbiotic_pair_system,
            thermal_vent_effects_system,
            dynamic_chemical_zone_system,
            scroll_thermal_vents,
        ).run_if(in_state(GameState::Playing)))

        // TIDAL MECHANICS GROUP

        .add_systems(Update, (
            advanced_tidal_system, // PROBLEM with apply_tidal_effects
            process_tidal_events,
            update_king_tide,
            update_tidal_debris,
        ).run_if(in_state(GameState::Playing)))

        // Chemical Environment Effects
        .add_systems(Update, (
            apply_chemical_damage_system,
            pheromone_communication_system,
            ecosystem_monitoring_system,
        ).run_if(in_state(GameState::Playing)))

        // animation systems
        .add_systems(Update, (
            signal_particle_spawning,
            virus_pulsing_animation,
            bacteria_flagella_animation,
            corruption_color_shift,
            warning_flash_animation,
            offspring_wiggle_animation,
            pseudopod_animation,
            gestation_animation,
            toxic_aura_animation,
        ).run_if(in_state(GameState::Playing)))

        // Event processing systems
        .add_systems(Update, (
            spawn_explosion_system,
            spawn_enemy_system,
            spawn_powerup_system,
            spawn_particles_system,
            spawn_atp_on_death,
            handle_player_hit,
            update_health_bar,
            update_biological_ui,
            update_evolution_ui,
            check_game_over,
            handle_restart_input,
        ).run_if(in_state(GameState::Playing)))

        // Debug Systems
        .add_systems(Update, (
            debug_atp_spawner,
            debug_spawn_evolution_chamber,
            debug_trigger_king_tide,
        ).run_if(in_state(GameState::Playing)))

        // Game Over
        .add_systems(OnEnter(GameState::GameOver), (save_high_score_to_file, enhanced_game_over_ui).chain())
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
        .add_systems(OnEnter(GameState::Playing), reset_biological_game_state)
        
        // Game Paused
        .add_systems(OnEnter(GameState::Paused), setup_pause_ui)
        .add_systems(OnExit(GameState::Paused), cleanup_pause_ui)
        .add_systems(Update, (
            handle_restart_button,
        ).run_if(in_state(GameState::GameOver)))

        .run();
}

pub fn init_tidal_state(mut commands: Commands) {
    commands.init_resource::<TidalState>();
}

// Enhanced player spawning with biological properties
pub fn spawn_biological_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("textures/player.png");
    
    commands.spawn((
        Sprite {
            image: texture,
            anchor: Anchor::Center,
            color: Color::srgb(0.8, 1.0, 0.9), // Slightly bioluminescent tint
            ..default()
        },
        Transform::from_xyz(0.0, -250.0, 0.0),
        Player {
            speed: 400.0,
            roll_factor: 0.3,
            lives: 3,
            invincible_timer: 0.0,
            cell_membrane_thickness: 1.0,
        },
        EvolutionSystem::default(),
        ATP { amount: 0 },
        CellularUpgrades::default(),
        Collider { radius: 16.0 },
        Health(100),
        EngineTrail,
        FluidDynamics {
            velocity: Vec2::ZERO,
            viscosity_resistance: 0.8,
            buoyancy: 20.0,
            current_influence: 1.0,
        },
        ChemicalSensitivity {
            ph_tolerance_min: 6.5,
            ph_tolerance_max: 7.5,
            oxygen_requirement: 0.3,
            damage_per_second_outside_range: 5,
        },
        CriticalHitStats::default(),
    ));
}

// Biological UI setup with updated terminology
pub fn setup_biological_ui(mut commands: Commands) {
    // Cellular integrity bar (health bar)
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
        BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
        BorderColor(Color::srgb(0.4, 0.8, 0.6)),
        HealthBar,
    ));
    
    // Cellular integrity fill
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(22.0),
            bottom: Val::Px(62.0),
            width: Val::Px(200.0),
            height: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.8, 0.4)), // Healthy green
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
        TextColor(Color::srgb(0.8, 1.0, 0.9)),
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
        TextColor(Color::srgb(0.8, 1.0, 0.9)),
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
        TextColor(Color::srgb(0.6, 0.8, 0.7)),
        HighScoreText,
    ));

    // ATP text (currency)
    commands.spawn((
        Text::new("ATP: 0⚡"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.3)),
        ATPText,
    ));

    // Evolution info text
    commands.spawn((
        Text::new("Evolution: Cytoplasmic Spray"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(50.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.7, 1.0, 0.7)),
        EvolutionText,
    ));

    // Tidal State
    commands.spawn((
        Text::new("Tide: Normal"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(110.0),
            ..default()
        },
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.6, 0.9, 1.0)),
        TidalStatusText,
    ));

    // Emergency spore counter
    commands.spawn((
        Text::new("Emergency Spores: 3"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(250.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 1.0)),
        SporeText,
    ));

    // Controls help
    commands.spawn((
        Text::new("SPACE: Emergency Spore | Near Evolution Chamber: 1-9 to evolve"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(100.0),
            ..default()
        },
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.5, 0.7, 0.6)),
        ControlsText,
    ));

    // Symbiotic multiplier text
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

    // Environmental status (new)
    commands.spawn((
        Text::new("pH: 7.0 | O₂: Normal"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(80.0),
            ..default()
        },
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.6, 0.9, 0.8)),
        EnvironmentText,
    ));

    // set up cellwall timer
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(130.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.4, 1.0, 0.8)),
        CellWallTimerText,
    ));

    commands.spawn((
        Text::new("FPS: --"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0), // Changed from bottom: Val::Px(40.0)
            ..default()
        },
        TextFont { 
            font_size: 18.0, // Increased size
            ..default() 
        },
        TextColor(Color::srgb(0.9, 1.0, 0.9)), // Brighter color
        FpsText,
    ));  
}

// New biological background with organic elements
pub fn setup_biological_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Organic background layers representing different depth levels
    let layers = vec![
        ("textures/bg_layer1.png", 0.05, -100.0, Color::srgb(0.2, 0.4, 0.6)), // Deep water
        ("textures/bg_layer2.png", 0.15, -50.0, Color::srgb(0.3, 0.6, 0.8)),  // Mid water with plankton
        ("textures/bg_layer3.png", 0.3, -25.0, Color::srgb(0.4, 0.8, 1.0)),   // Surface with debris
    ];
    
    for (path, speed, depth, tint) in layers {
        let texture = asset_server.load(path);
        commands.spawn((
            Sprite {
                image: texture,
                color: tint,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, depth),
            ParallaxLayer { speed, depth },
        ));
    }

    // Add some ambient current indicators
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 200.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.3, 0.8, 1.0, 0.1),
                custom_size: Some(Vec2::new(50.0, 200.0)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, -80.0),
            CurrentField {
                direction: Vec2::new(0.0, -1.0),
                strength: 50.0,
                turbulence: 0.2,
            },
            ParallaxLayer { speed: 0.1, depth: -80.0 },
        ));
    }
}

// Load biological-themed assets
pub fn load_biological_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        // Player and general
        player_texture: asset_server.load("textures/player.png"),
        projectile_texture: asset_server.load("textures/bullet.png"),
        explosion_texture: asset_server.load("textures/explosion.png"),
        particle_texture: asset_server.load("textures/particle.png"),
        barrier_texture: asset_server.load("textures/shield_barrier.png"),
        
        // UNIQUE ENEMY SPRITES
        enemy_texture: asset_server.load("textures/enemy.png"),
        viral_particle_texture: asset_server.load("textures/enemies/viral_particle.png"),
        aggressive_bacteria_texture: asset_server.load("textures/enemies/aggressive_bacteria.png"),
        parasitic_protozoa_texture: asset_server.load("textures/enemies/parasitic_protozoa.png"),
        infected_macrophage_texture: asset_server.load("textures/enemies/infected_macrophage.png"),
        suicidal_spore_texture: asset_server.load("textures/enemies/suicidal_spore.png"),
        biofilm_colony_texture: asset_server.load("textures/enemies/biofilm_colony.png"),
        swarm_cell_texture: asset_server.load("textures/enemies/swarm_cell.png"),
        reproductive_vesicle_texture: asset_server.load("textures/enemies/reproductive_vesicle.png"),
        offspring_texture: asset_server.load("textures/enemies/offspring.png"),
        
        // Power-ups (existing)
        health_powerup_texture: asset_server.load("textures/health_powerup.png"),
        shield_powerup_texture: asset_server.load("textures/shield_powerup.png"),
        speed_powerup_texture: asset_server.load("textures/speed_powerup.png"),
        multiplier_powerup_texture: asset_server.load("textures/symbiotic.png"),
        rapidfire_powerup_texture: asset_server.load("textures/weapon_powerup.png"),
        
        // Background and audio (existing)
        background_layers: vec![
            asset_server.load("textures/bg_layer1.png"),
            asset_server.load("textures/bg_layer2.png"),
            asset_server.load("textures/bg_layer3.png"),
        ],
        sfx_shoot: asset_server.load("audio/organic_pulse.ogg"),
        sfx_explosion: asset_server.load("audio/cell_burst.ogg"),
        sfx_powerup: asset_server.load("audio/evolution.ogg"),
        music: asset_server.load("audio/tidal_pool_ambience.ogg"),
    };
    commands.insert_resource(assets);
}

// Initialize fluid environment
pub fn init_fluid_environment(mut commands: Commands) {
    commands.insert_resource(FluidEnvironment {
        current_field: vec![Vec2::ZERO; 64 * 64],
        grid_size: 64,
        cell_size: 20.0,
        tidal_phase: 0.0,
        turbulence_intensity: 0.3,
    });
}

// Initialize chemical zones
pub fn init_chemical_zones(mut commands: Commands) {
    commands.insert_resource(ChemicalEnvironment {
        ph_zones: vec![
            resources::ChemicalZone {
                position: Vec2::new(-200.0, 100.0),
                radius: 150.0,
                ph_level: 5.5, // Acidic zone
                intensity: 0.8,
            },
            resources::ChemicalZone {
                position: Vec2::new(200.0, -100.0),
                radius: 120.0,
                ph_level: 8.5, // Alkaline zone
                intensity: 0.6,
            },
        ],
        oxygen_zones: vec![
            OxygenZone {
                position: Vec2::new(0.0, 200.0),
                radius: 180.0,
                oxygen_level: 0.9,
                depletion_rate: 0.1,
            },
        ],
        base_ph: 7.0,
        base_oxygen: 0.5,
        diffusion_rate: 0.1,
    });
}

// Enhanced UI update system with biological terminology
pub fn update_biological_ui(
    game_score: Res<GameScore>,
    player_query: Query<(&Player, &ATP, &EvolutionSystem)>,
    environment: Res<ChemicalEnvironment>,
    mut atp_query: Query<&mut Text, (With<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut evolution_query: Query<&mut Text, (With<EvolutionText>, Without<ATPText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut spore_query: Query<&mut Text, (With<SporeText>, Without<ATPText>, Without<EvolutionText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut high_score_query: Query<&mut Text, (With<HighScoreText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut multiplier_query: Query<&mut Text, (With<MultiplierText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<EnvironmentText>)>,
    mut environment_query: Query<&mut Text, (With<EnvironmentText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
) {
    if let Ok((player, atp, evolution_system)) = player_query.single() {
        // Update ATP display
        if let Ok(mut atp_text) = atp_query.single_mut() {
            **atp_text = format!("ATP: {}⚡", atp.amount);
        }
        
        // Update evolution display
        if let Ok(mut evolution_text) = evolution_query.single_mut() {
            **evolution_text = format!("Evolution: {}", evolution_system.primary_evolution.get_display_name());
        }
        
        // Update emergency spore counter
        if let Ok(mut spore_text) = spore_query.single_mut() {
            **spore_text = format!("Emergency Spores: {}", evolution_system.emergency_spores);
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

    // Update symbiotic multiplier
    if let Ok(mut multiplier_text) = multiplier_query.single_mut() {
        if game_score.score_multiplier > 1.0 {
            **multiplier_text = format!("{}x Symbiosis ({:.1}s)", game_score.score_multiplier, game_score.multiplier_timer);
        } else {
            **multiplier_text = String::new(); // This should clear it
        }
    }

    // Update environment status
    if let Ok(mut env_text) = environment_query.single_mut() {
        **env_text = format!("pH: {:.1} | O₂: {:.0}%", 
            environment.base_ph, 
            environment.base_oxygen * 100.0
        );
    }
}

// Enhanced game reset with biological state
pub fn reset_biological_game_state(
    mut commands: Commands,
    mut game_score: ResMut<GameScore>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut input_state: ResMut<InputState>,
    mut game_started: ResMut<GameStarted>,
    mut shooting_state: ResMut<ShootingState>,
    mut fluid_environment: ResMut<FluidEnvironment>,
    mut chemical_environment: ResMut<ChemicalEnvironment>,
    // Despawn all game entities
    (enemy_query, projectile_query): (Query<Entity, With<Enemy>>,Query<Entity, (With<Projectile>, Without<AlreadyDespawned>)>),
    explosion_query: Query<Entity, With<Explosion>>,
    (powerup_query,weapon_powerup_query): (Query<Entity, With<PowerUp>>, Query<Entity, With<EvolutionPowerUp>>),
    (currency_entity_query, upgrade_station_query): (Query<Entity, (With<ATP>, Without<Player>)>, Query<Entity, With<EvolutionChamber>>),
    (particle_query, emitter_query): (Query<Entity, With<Particle>>,Query<Entity, With<ParticleEmitter>>),
    (laser_query, smart_bomb_query): (Query<Entity, With<LaserBeam>>, Query<Entity, With<SporeWave>>),
    (player_query, upgrade_ui_query) : (Query<Entity, With<Player>>, Query<Entity, With<EvolutionUI>>),
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
        commands.entity(entity)
            .insert(AlreadyDespawned)
            .despawn();
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
    
    // Reset fluid environment
    fluid_environment.tidal_phase = 0.0;
    fluid_environment.turbulence_intensity = 0.3;
    
    // Reset chemical environment
    chemical_environment.base_ph = 7.0;
    chemical_environment.base_oxygen = 0.5;
    
    // Respawn biological player
    if let Some(assets) = assets {
        commands.spawn((
            Sprite {
                image: assets.player_texture.clone(),
                anchor: Anchor::Center,
                color: Color::srgb(0.8, 1.0, 0.9),
                ..default()
            },
            Transform::from_xyz(0.0, -250.0, 0.0),
            Player {
                speed: 400.0,
                roll_factor: 0.3,
                lives: 3,
                invincible_timer: 3.0,
                cell_membrane_thickness: 1.0,
            },
            EvolutionSystem::default(),
            ATP { amount: 0 },
            CellularUpgrades::default(),
            Collider { radius: 16.0 },
            Health(100),
            EngineTrail,
            FluidDynamics {
                velocity: Vec2::ZERO,
                viscosity_resistance: 0.8,
                buoyancy: 20.0,
                current_influence: 1.0,
            },
            ChemicalSensitivity {
                ph_tolerance_min: 6.5,
                ph_tolerance_max: 7.5,
                oxygen_requirement: 0.3,
                damage_per_second_outside_range: 5,
            },
        ));
    }
}

// New biological movement system
pub fn biological_movement_system(
    mut player_query: Query<(&mut Transform, &mut FluidDynamics, &Player)>,
    input_state: Res<InputState>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut fluid, player)) = player_query.single_mut() {
        // Player input creates thrust against fluid resistance
        let thrust = input_state.movement * player.speed * 2.0;
        
        // Sample current from fluid field
        let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
        let current = sample_current(&fluid_environment, grid_pos);
        
        // Physics integration with biological properties
        let drag = fluid.velocity * -fluid.viscosity_resistance;
        let buoyancy = Vec2::new(0.0, fluid.buoyancy);
        let current_force = current * fluid.current_influence;
        
        let acceleration = thrust + current_force + drag + buoyancy;
        fluid.velocity += acceleration * time.delta_secs();
        
        // Apply velocity to position with organic damping
        transform.translation += fluid.velocity.extend(0.0) * time.delta_secs();
        
        // Boundary conditions with surface tension effect
        transform.translation.x = transform.translation.x.clamp(-600.0, 600.0);
        transform.translation.y = transform.translation.y.clamp(-350.0, 350.0);
        
        // Organic roll motion based on fluid flow
        let flow_influence = (fluid.velocity.x + current.x) * 0.001;
        let target_roll = -input_state.movement.x * player.roll_factor + flow_influence;
        transform.rotation = transform.rotation.lerp(
            Quat::from_rotation_z(target_roll),
            time.delta_secs() * 6.0
        );
    }
}

// Enhanced particle system for organic effects
pub fn update_organic_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite, Option<&BioluminescentParticle>), Without<AlreadyDespawned>>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite, bioluminescent) in particle_query.iter_mut() {
        particle.lifetime += time.delta_secs();
        
        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity)
                .insert(AlreadyDespawned)
                .despawn();
            continue;
        }

        // Organic motion based on drift pattern
        match particle.drift_pattern {
            DriftPattern::Floating => {
                let bob = (time.elapsed_secs() * 2.0 + transform.translation.x * 0.01).sin();
                transform.translation.y += bob * 15.0 * time.delta_secs();
                
                // Gentle rotation
                transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.5);
            }
            
            DriftPattern::Pulsing => {
                let pulse = (time.elapsed_secs() * 4.0).sin();
                let scale = particle.size * (0.8 + pulse * 0.2);
                transform.scale = Vec3::splat(scale);
            }
            
            DriftPattern::Spiraling => {
                let angle = time.elapsed_secs() * 2.0;
                let spiral_radius = 10.0;
                particle.velocity.x += angle.cos() * spiral_radius * time.delta_secs();
                particle.velocity.y += angle.sin() * spiral_radius * time.delta_secs();
            }
            
            DriftPattern::Brownian => {
                // Random micro-movements for molecular motion
                let random_force = Vec2::new(
                    (time.elapsed_secs() * 123.45).sin() * 50.0,
                    (time.elapsed_secs() * 678.90).cos() * 50.0,
                );
                particle.velocity += random_force * time.delta_secs();
            }
        }

        // Apply current influence to organic particles
        let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
        let current = sample_current(&fluid_environment, grid_pos);
        particle.velocity += current * 0.3 * time.delta_secs();

        // Update position
        transform.translation += particle.velocity.extend(0.0) * time.delta_secs();
        
        // Apply fluid drag
        particle.velocity *= 0.98;
        
        // Bioluminescent effects
        if let Some(bio_particle) = bioluminescent {
            let pulse = (time.elapsed_secs() * bio_particle.pulse_frequency).sin();
            let brightness = 0.7 + pulse * bio_particle.pulse_intensity;
            
            let mut color = bio_particle.base_color;
            let alpha = (1.0 - particle.lifetime / particle.max_lifetime) * particle.fade_rate;
            color.set_alpha(alpha * brightness);
            sprite.color = color;
        } else {
            // Standard particle fade
            let progress = particle.lifetime / particle.max_lifetime;
            let alpha = 1.0 - progress;
            sprite.color.set_alpha(alpha * particle.fade_rate);
        }
        
        // Organic size variation
        if particle.bioluminescent {
            let size_pulse = (time.elapsed_secs() * 3.0 + particle.lifetime * 2.0).sin();
            let scale = particle.size * (0.9 + size_pulse * 0.1);
            transform.scale = Vec3::splat(scale);
        }
    }
}

// Bioluminescent trail system (replaces engine particles)
pub fn spawn_bioluminescent_trail(
    mut commands: Commands,
    player_query: Query<&Transform, With<EngineTrail>>,
    input_state: Res<InputState>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut trail_segments: Local<Vec<Vec3>>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer -= time.delta_secs();
    
    if *spawn_timer <= 0.0 {
        for transform in player_query.iter() {
            let intensity = input_state.movement.length().max(0.2);
            
            // Add new trail segment
            trail_segments.push(transform.translation + Vec3::new(0.0, -18.0, -0.1));
            
            // Keep only last 15 segments
            if trail_segments.len() > 15 {
                trail_segments.remove(0);
            }
            
            // Spawn connected membrane segments
            if let Some(assets) = &assets {
                for (i, &segment_pos) in trail_segments.iter().enumerate() {
                    let age = i as f32 / trail_segments.len() as f32;
                    let alpha = age * 0.6 * intensity;
                    let width = (age * 8.0 + 2.0) * intensity;
                    
                    commands.spawn((
                        Sprite {
                            image: assets.particle_texture.clone(),
                            color: Color::srgba(0.3, 0.9, 1.0, alpha),
                            custom_size: Some(Vec2::splat(width)),
                            ..default()
                        },
                        Transform::from_translation(segment_pos),
                        Particle {
                            velocity: Vec2::ZERO,
                            lifetime: 0.0,
                            max_lifetime: 0.8,
                            size: width,
                            fade_rate: 2.0,
                            bioluminescent: true,
                            drift_pattern: DriftPattern::Floating,
                        },
                    ));
                }
            }
        }
        
        *spawn_timer = 0.05;
    }
}

// Update biological effects (replaces update_player_effects)
pub fn update_biological_effects(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &Transform), With<Player>>,
    mut cell_wall_query: Query<(Entity, &mut CellWallReinforcement)>,
    mut cell_wall_visual_query: Query<(Entity, &mut Transform, &mut Sprite), (With<CellWallVisual>, Without<Player>, Without<AlreadyDespawned>)>,
    mut flagella_query: Query<(Entity, &mut FlagellaBoost)>,
    mut symbiotic_query: Query<(Entity, &mut SymbioticMultiplier)>,
    mut mitochondria_query: Query<(Entity, &mut MitochondriaOvercharge)>,
    mut photosynthesis_query: Query<(Entity, &mut PhotosynthesisActive, &mut Health)>,
    mut chemotaxis_query: Query<(Entity, &mut ChemotaxisActive)>,
    mut osmoregulation_query: Query<(Entity, &mut OsmoregulationActive)>,
    mut binary_fission_query: Query<(Entity, &mut BinaryFissionActive)>,
    mut game_score: ResMut<GameScore>,
    time: Res<Time>,
) {
    if let Ok((_, mut player, _player_transform)) = player_query.single_mut() {
        player.invincible_timer = (player.invincible_timer - time.delta_secs()).max(0.0);
    }

    // Update cell wall reinforcement
    let mut cell_wall_active = false;
    for (entity, mut cell_wall) in cell_wall_query.iter_mut() {
        cell_wall.timer -= time.delta_secs();
        cell_wall.alpha_timer += time.delta_secs();
        cell_wall_active = true;
        
        if cell_wall.timer <= 0.0 {
            commands.entity(entity).remove::<CellWallReinforcement>();
            cell_wall_active = false;
        }
    }

    // Update cell wall visual with organic pulsing
    if let Ok((_player_entity, _, player_transform)) = player_query.single() {
        if cell_wall_active {
            if let Ok((_, mut cell_wall_transform, mut cell_wall_sprite)) = cell_wall_visual_query.single_mut() {
                // Follow player position
                cell_wall_transform.translation = player_transform.translation;
                
                // Organic pulsing effect
                let pulse = (time.elapsed_secs() * 3.0).sin() * 0.15 + 0.85;
                cell_wall_transform.scale = Vec3::splat(pulse);
                
                // Bioluminescent breathing alpha
                let alpha = 0.3 + (time.elapsed_secs() * 2.0).sin().abs() * 0.2;
                cell_wall_sprite.color = Color::srgba(0.4, 1.0, 0.8, alpha);
                
                // Organic rotation
                cell_wall_transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.3);
            } else {
                // Create new cell wall visual
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.4, 1.0, 0.8, 0.4),
                        custom_size: Some(Vec2::splat(70.0)),
                        ..default()
                    },
                    Transform::from_translation(player_transform.translation),
                    CellWallVisual,
                ));
            }
        } else {
            // Remove cell wall visual when expired
            for (cell_wall_visual_entity, _, _) in cell_wall_visual_query.iter() {
                commands.entity(cell_wall_visual_entity)
                    .insert(AlreadyDespawned)
                    .despawn();
            }
        }
    }

    // Update other biological effects
    for (entity, mut flagella) in flagella_query.iter_mut() {
        flagella.timer -= time.delta_secs();
        if flagella.timer <= 0.0 {
            commands.entity(entity).remove::<FlagellaBoost>();
        }
    }

    for (entity, mut symbiotic) in symbiotic_query.iter_mut() {
        symbiotic.timer -= time.delta_secs();
        game_score.score_multiplier = symbiotic.multiplier;
        game_score.multiplier_timer = symbiotic.timer;
        
        if symbiotic.timer <= 0.0 {
            game_score.score_multiplier = 1.0; // Add this line
            game_score.multiplier_timer = 0.0;  // Add this line
            commands.entity(entity).remove::<SymbioticMultiplier>();
        }
    }

    for (entity, mut mitochondria) in mitochondria_query.iter_mut() {
        mitochondria.timer -= time.delta_secs();
        if mitochondria.timer <= 0.0 {
            commands.entity(entity).remove::<MitochondriaOvercharge>();
        }
    }

    // New biological effects
    for (entity, mut photosynthesis, mut health) in photosynthesis_query.iter_mut() {
        photosynthesis.timer -= time.delta_secs();
        
        // Heal over time from photosynthesis
        health.0 = (health.0 + (photosynthesis.energy_per_second * time.delta_secs()) as i32).min(100);
        
        if photosynthesis.timer <= 0.0 {
            commands.entity(entity).remove::<PhotosynthesisActive>();
        }
    }

    for (entity, mut chemotaxis) in chemotaxis_query.iter_mut() {
        chemotaxis.timer -= time.delta_secs();
        if chemotaxis.timer <= 0.0 {
            commands.entity(entity).remove::<ChemotaxisActive>();
        }
    }

    for (entity, mut osmoregulation) in osmoregulation_query.iter_mut() {
        osmoregulation.timer -= time.delta_secs();
        if osmoregulation.timer <= 0.0 {
            commands.entity(entity).remove::<OsmoregulationActive>();
        }
    }

    for (entity, mut binary_fission) in binary_fission_query.iter_mut() {
        binary_fission.timer -= time.delta_secs();
        binary_fission.clone_timer -= time.delta_secs();
        
        if binary_fission.clone_timer <= 0.0 {
            // Spawn clone projectile
            // This would be implemented in the weapon system
            binary_fission.clone_timer = 0.5; // Reset clone timer
        }
        
        if binary_fission.timer <= 0.0 {
            commands.entity(entity).remove::<BinaryFissionActive>();
        }
    }
}

// Helper functions for biological systems
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

// New UI component markers
#[derive(Component)]
pub struct ATPText;

#[derive(Component)]
pub struct EvolutionText;

#[derive(Component)]
pub struct SporeText;

#[derive(Component)]
pub struct EnvironmentText;

// fix for the fluid_dynamics_system panic
pub fn init_current_generator(mut commands: Commands) {
    commands.insert_resource(CurrentGenerator::default());
}


pub fn debug_atp_spawner(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    assets: Option<Res<GameAssets>>,
) {
    if keyboard.just_pressed(KeyCode::F2) {
        if let Ok(player_transform) = player_query.single() {
            if let Some(assets) = assets {
                commands.spawn((
                    Sprite {
                        image: assets.multiplier_powerup_texture.clone(),
                        color: Color::srgb(1.0, 1.0, 0.3),
                        custom_size: Some(Vec2::splat(18.0)),
                        ..default()
                    },
                    Transform::from_translation(player_transform.translation + Vec3::new(32.0, 0.0, 0.0)),
                    ATP { amount: 1000 },
                    Collider { radius: 9.0 },
                ));
            }
        }
    }
}


pub fn debug_spawn_evolution_chamber(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    assets: Option<Res<GameAssets>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        if let Some(assets) = assets {
            commands.spawn((
                Sprite {
                    image: assets.enemy_texture.clone(),
                    color: Color::srgb(0.3, 0.9, 0.6),
                    custom_size: Some(Vec2::splat(60.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, -100.0, 0.0), // Near player for testing
                EvolutionChamber,
                BioluminescentParticle {
                    base_color: Color::srgb(0.3, 0.9, 0.6),
                    pulse_frequency: 1.0,
                    pulse_intensity: 0.6,
                    organic_motion: OrganicMotion {
                        undulation_speed: 0.8,
                        response_to_current: 0.2,
                    },
                },
            ));
        }
    }
}


pub fn debug_trigger_king_tide(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tidal_physics: ResMut<TidalPoolPhysics>,
    mut tidal_events: EventWriter<TidalEvent>,
) {
    if keyboard.just_pressed(KeyCode::F4) {
        println!("🌊 DEBUG: Triggering King Tide!");
        tidal_physics.king_tide_active = true;
        tidal_physics.king_tide_timer = 0.0;
        tidal_physics.king_tide_intensity = 3.0;
        tidal_events.write(TidalEvent::KingTideBegin {
            intensity: 3.0,
            duration: 15.0,
        });
    }
}