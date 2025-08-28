use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::sprite::Anchor;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use Cosmic_Tidal_Pool::*;
use crate::lighting::PerformantLightingPlugin;
use cosmic_ui::prelude::*;
use crate::balance_systems::*;


fn main() {
    App::new()
        // ===== CORE BEVY SETUP =====
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cosmic Tidal Pool".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default()) // For FPS display
        // .add_plugins(LogDiagnosticsPlugin::default()) // For GPU display
        .add_plugins(input::InputPlugin)        // Remappable input (keyboard, gamepad)
        .add_plugins(PerformantLightingPlugin)
        .add_plugins(MenuSystemsPlugin)

        // ===== NEW: COSMIC UI SETUP =====
        .add_plugins(CosmicUIPlugin)              // Add the Cosmic UI plugin
        // .register_hud::<BiologicalGameHUD>()      // Register your HUD (generates all update systems!)

        .insert_resource(ClearColor(Color::srgb(0.05, 0.15, 0.25))) // Deep ocean background

        // ===== CORE GAME RESOURCES =====
        .init_resource::<OldInputState>()        // Legacy Input (Temporary)
        .init_resource::<EnemySpawner>()         // Enemy wave spawning system
        .init_resource::<GameScore>()            // Score tracking and high scores
        .init_resource::<GameStarted>()          // Game initialization flag
        .init_resource::<ShootingState>()        // Weapon firing rate modifiers
        .init_resource::<ScreenShakeResource>()  // Screen shake for impact feedback

        // ===== BIOLOGICAL SYSTEMS RESOURCES =====
        .init_resource::<FluidEnvironment>()     // Water current simulation grid
        .init_resource::<ChemicalEnvironment>()  // pH zones and oxygen simulation
        .init_resource::<TidalPoolPhysics>()     // Tide mechanics and king tide events
        .init_resource::<BioluminescenceManager>() // Organic lighting system
        .init_resource::<EcosystemState>()       // Environmental health tracking
        .init_resource::<TidalState>()           // Tidal event state tracking
        .init_resource::<AchievementManager>()   // Steam-ready achievement system
        .init_resource::<TidalFeedbackSystem>()  // Visual feedback for tidal effects
        .init_resource::<DiagnosticsStore>()
        .init_resource::<MenuSettings>() 
        .init_resource::<WaveManager>()
        .init_resource::<BalanceAnalyzer>()

        // ===== GAME STATE MANAGEMENT =====
        .init_state::<GameState>() // Playing, Paused, GameOver states

        // ===== CORE GAME EVENTS =====
        .add_event::<SpawnExplosion>()          // Biological cell bursts and explosions
        .add_event::<SpawnEnemy>()              // Dynamic enemy spawning with AI types
        .add_event::<SpawnPowerUp>()            // Biological evolution power-ups
        .add_event::<SpawnParticles>()          // Organic particle effects system
        .add_event::<PlayerHit>()               // Player damage and invincibility frames
        .add_event::<AddScreenShake>()          // Dynamic screen shake for impacts
        .add_event::<EnemyHit>()                // Enemy flash effects when damaged
        .add_event::<SpawnEnhancedExplosion>()  // Advanced explosion system
        .add_event::<TidalEvent>()              // King tides, current reversals
        .add_event::<AchievementEvent>()        // Achievement progression tracking
        .add_event::<BalanceAdjustmentEvent>()

        // ===== STARTUP SYSTEMS (Run Once) =====
        .add_systems(Startup, (
            setup_camera,                    // Initialize 2D camera with orthographic projection
            setup_biological_background,    // Spawn current indicators and environmental elements
            spawn_biological_player,        // Create player cell with fluid dynamics
            load_biological_assets,         // Load unique enemy sprites and audio
            // load_game_fonts,                // Load custom game font
            load_high_scores_from_file,     // Load persistent high score data
            init_particle_pool,             // Pre-allocate particle system
            init_fluid_environment,         // Initialize water current simulation
            init_chemical_zones,            // Place initial pH and oxygen zones
            init_current_generator,         // Set up thermal vents and major currents
            setup_achievement_system,       // Initialize Steam-ready achievements
            init_procedural_background,     // Set up dynamic background generation
            
            setup_audio_system, // replaces vv
            // start_ambient_music.after(load_biological_assets), // Begin ocean ambience

            initialize_balance_analyzer,
            // NEW: Spawn the Cosmic UI HUD
            // spawn_game_hud.after(load_game_fonts),
        ))

        .add_systems(OnEnter(GameState::Playing), (
            setup_biological_ui,            // Create UI with biological terminology
            setup_fps_ui,
            setup_wave_ui,
        ))

        // ===== CORE GAME LOOP SYSTEMS =====
        .add_systems(Update, (
            audio_system,           // Play sound effects for shooting, explosions
            music_system, 
            audio_cleanup_system,
            // handle_pause_input,     // ESC/P key pause toggle
            fps_text_update_system,
        ))

        // ===== PRIMARY GAMEPLAY SYSTEMS (Playing State Only) =====
        .add_systems(Update, (

            // Core player and enemy interaction
            // handle_input_legacy,             // Process keyboard/gamepad input
            biological_movement_system,      // Player movement with fluid dynamics
            enhanced_shooting_system,        // Evolution-based weapon systems

            atp_magnet_system,

            // spawn_enemies,               // Wave-based enemy spawning, replaced by following 3 functions
            wave_progression_system,
            wave_spawning_system,
            environmental_hazard_system,
            wave_completion_system,

            spawn_biological_powerups,      // ATP and evolution power-ups
            spawn_evolution_powerups,       // Advanced evolutionary upgrades

            // Achievement and progression tracking
            track_achievements_system,       // Monitor progress for Steam achievements
            update_achievement_notifications, // Display achievement unlock notifications

            // Special enemy behaviors
            link_symbiotic_pairs,           // Connect paired organisms
            spawn_evolution_chambers,       // Player upgrade stations
        ).run_if(in_state(GameState::Playing)))
        
        // ===== PROCEDURAL BACKGROUND SYSTEMS =====
        .add_systems(Update, (
            procedural_background_generation, // Dynamic coral reefs, debris spawning
            update_background_particles,     // Animate plankton, chemical particles
            enhanced_parallax_system,        // Multi-layer scrolling backgrounds
            biological_feedback_system,      // Environmental health indicators
            update_depth_of_field_focus,    // Camera depth effects for immersion
        ).run_if(in_state(GameState::None)))

        // ===== ADVANCED AI SYSTEMS =====
        .add_systems(Update, (
            // Ecosystem simulation
            adaptive_difficulty_system,      // Scale challenge to player evolution
            // chemical_trail_system,          // Pheromone tracking for AI - can ause a panic with despawn
            // chemical_trail_following,       // Enemies follow chemical trails
            ecosystem_balance_system,       // Population dynamics simulation

            // Environmental storytelling
            enhanced_coral_system,          // Dynamic coral health and corruption
            //contamination_visualization_system, // Show pollution effects
            //microscopic_debris_system,      // Story fragments in debris
            //bioluminescent_warning_system,  // Emergency lighting for dangers
            //environmental_narrative_system, // Dynamic environmental storytelling
        ).run_if(in_state(GameState::Playing)))

        // ===== SPECIAL EFFECT SYSTEMS =====
        .add_systems(Update, (
            spawn_extra_life_powerups,      // Rare life-extending power-ups
            extra_life_collection_system,   // Handle life gain with celebration
            // update_dynamic_lights,          // Bioluminescent lighting effects
            // render_light_effects,           // Convert lighting to visual sprites
        ).run_if(in_state(GameState::Playing)))

        // ===== PARTICLE AND EFFECT SYSTEMS =====
        .add_systems(Update, (        
            performance_optimization_system, // Limit entity processing per frame
            
            // Visual effects management
            update_biological_effects,      // Player power-up timers and visuals
            update_temporary_evolution_effects, // Temporary stat modifications
            consolidated_explosion_system,  // Multi-layered explosion rendering

            unified_particle_system,        // All particle types in one system

            // Environmental effects
            update_parallax,                // Background layer scrolling
            cleanup_offscreen,              // Remove entities outside play area
            spawn_bioluminescent_trail,     // Player movement trail effects

            // NEW: Add Cosmic UI animation systems
            update_progress_bars,
            animate_status_indicators,            
        ).run_if(in_state(GameState::Playing)))

        // ===== PROJECTILE AND MOVEMENT SYSTEMS =====
        .add_systems(Update, (
            move_projectiles,               // Update all projectile positions

            unified_weapon_update_system,   // Homing missiles, laser beams, toxin clouds
           
            // Currency and upgrade systems
            move_biological_powerups,       // Organic floating animation for power-ups
            move_atp,                       // ATP energy particles with current response
            collect_atp_with_energy_transfer, // Enhanced ATP collection with particles
        ).run_if(in_state(GameState::Playing)))

        // ===== FEEDBACK AND UI SYSTEMS =====
        .add_systems(Update, (
            
            update_cell_wall_timer,         // Shield timer display
            
            enemy_flash_system,             // Flash enemies white when hit
            screen_shake_system,            // Camera shake for impacts

            // Advanced tidal feedback systems
            enhanced_tidal_feedback_system, // Visual indicators for currents/tides
            update_fluid_motion_visualizers, // Current flow visualization
            update_tidal_wave_effects,      // King tide wave propagation
            tidal_audio_feedback_system,    // Sound cues for tidal events
            tidal_movement_response_system, // Enhanced player response to currents
        ).run_if(in_state(GameState::Playing)))

        // ===== BALANCE ANALYSIS SYSTEMS (ADD TO UPDATE) =====
        .add_systems(Update, (
            // Core balance analysis systems
            real_time_balance_analysis,
            weapon_performance_tracking,
            atp_economy_analysis,
            progression_balance_system,
            
            // Balance testing and tuning
            auto_balance_tuning_system,
            handle_balance_adjustments,
            
            // Debug and UI systems
            balance_debug_ui,
            balance_debug_commands,
        ).run_if(in_state(GameState::Playing)))

        // ===== ENEMY AI AND COMBAT SYSTEMS =====
        .add_systems(Update, (
            enemy_shooting,                 // Enemy projectile attacks
            turret_shooting,                // Biofilm colony ranged attacks
            move_enemies,                   // All enemy movement AI patterns
            update_spawner_enemies,         // Reproductive vesicle offspring spawning
            update_formations,              // Colony coordination and movement
            formation_coordination_system,  // Chemical signaling between colony members
            procedural_colony_spawning,     // Dynamic enemy group generation
        ).run_if(in_state(GameState::Playing)))

        // ===== COLLISION AND INTERACTION SYSTEMS =====
        .add_systems(Update, (            
            collision_system,               // All projectile-enemy-player collisions
            atp_pickup_system,              // Energy collection from defeated enemies
            evolution_powerup_collection,   // Evolutionary upgrade collection
            evolution_chamber_interaction,  // Player upgrades at evolution chambers
            handle_biological_powerup_collection, // Power-up effects application
            damage_text_system,             // Floating combat damage numbers
        ).run_if(in_state(GameState::Playing)))

        // ===== BIOLOGICAL ENVIRONMENT SIMULATION =====
        .add_systems(Update, (
            
            fluid_dynamics_system,          // Water current field generation
            chemical_environment_system,    // pH and oxygen zone simulation
            //update_current_field,           // Current indicator visualization
            organic_ai_system,              // Biological AI behaviors (chemotaxis, etc.)
            //generate_procedural_currents,   // Dynamic current pattern generation
            cell_division_system,           // Enemy reproduction mechanics
            symbiotic_pair_system,          // Paired organism death mechanics
            thermal_vent_effects_system,    // Heat effects and thermal particles
            //dynamic_chemical_zone_system,   // Adaptive chemical zone spawning
            //scroll_thermal_vents,           // Move thermal vents with current
        ).run_if(in_state(GameState::None)))

        // ===== TIDAL MECHANICS SYSTEMS =====
        .add_systems(Update, (
            advanced_tidal_system,          // King tide events, tidal cycles
            process_tidal_events,           // Handle tidal event responses
            update_king_tide,               // King tide duration and effects
            update_tidal_debris,            // Debris movement during king tides
        ).run_if(in_state(GameState::None)))

        // ===== CHEMICAL AND ENVIRONMENTAL EFFECTS =====
        .add_systems(Update, (
            //apply_chemical_damage_system,   // pH and oxygen damage to entities
            pheromone_communication_system, // Colony chemical coordination
            ecosystem_monitoring_system,    // Track ecosystem health metrics
        ).run_if(in_state(GameState::None)))

        // ===== EVENT PROCESSING SYSTEMS =====
        .add_systems(Update, (
            spawn_explosion_system,         // Create explosion entities from events
            spawn_enemy_system,             // Create enemy entities from events
            spawn_powerup_system,           // Create power-up entities from events
            spawn_particles_system,         // Create particle effects from events
            spawn_atp_on_death,             // Drop ATP currency when enemies die
            handle_player_hit,              // Process player damage and lives
            update_health_bar,              // Update UI health display
            check_game_over,                // Transition to game over state
            handle_restart_input,           // R key restart functionality
        ).run_if(in_state(GameState::Playing)))

        // ===== USER INTERFACE SYSTEMS =====
        .add_systems(Update, (
            update_cell_wall_timer_ui,      // Shield duration display
            update_evolution_ui,            // Evolution chamber upgrade menu
            update_tidal_ui,                // Tide status indicator
            update_biological_ui,           // ATP, score, lives, ecosystem status
            wave_ui_system,
        ).run_if(in_state(GameState::Playing)))

        // ===== DEBUG SYSTEMS (Development Only) =====
        .add_systems(Update, (
            debug_atp_spawner,              // F2: Spawn 1000 ATP for testing
            debug_spawn_evolution_chamber,  // F3: Spawn evolution chamber
            debug_trigger_king_tide,        // F4: Force trigger king tide event
        ).run_if(in_state(GameState::Playing)))

        // ===== GAME STATE TRANSITION SYSTEMS =====
        
        .add_systems(OnExit(GameState::Playing), (
            finalize_balance_session,
            save_balance_data_system,
        ))

        // When transitioning TO game over state
        .add_systems(OnEnter(GameState::GameOver), (
            save_high_score_to_file,        // Persist high score data
            // enhanced_game_over_ui_cosmic,
            enhanced_game_over_ui            // Show detailed stats and high score table
        ).chain()) // Ensure save happens before UI
        
        // When leaving game over state
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
        
        // When starting/restarting a game
        .add_systems(OnEnter(GameState::Playing), reset_biological_game_state)
        
        // Pause state management
        .add_systems(OnEnter(GameState::Paused), setup_pause_ui)
        // .add_systems(OnEnter(GameState::Paused), setup_pause_ui_cosmic)
        .add_systems(OnExit(GameState::Paused), cleanup_pause_ui)
        
        // Game over input handling
        .add_systems(Update, (
            handle_restart_button,          // UI button for restarting
        ).run_if(in_state(GameState::GameOver)))

        .run();
}

// ===== INITIALIZATION HELPER FUNCTIONS =====

/// Initialize tidal state tracking for king tides and debris
pub fn init_tidal_state(mut commands: Commands) {
    commands.init_resource::<TidalState>();
}

/// Set up Steam-ready achievement system with progress tracking
pub fn setup_achievement_system(mut commands: Commands) {
    let mut achievement_manager = achievements::initialize_achievements();
    
    // Load saved progress if available
    if let Ok(save_data) = std::fs::read_to_string("achievements.json") {
        if let Ok(loaded_data) = serde_json::from_str::<achievements::AchievementSaveData>(&save_data) {
            achievement_manager.unlocked_achievements = loaded_data.unlocked_achievements;
            achievement_manager.lifetime_stats = loaded_data.lifetime_stats;
            println!("Loaded {} unlocked achievements", achievement_manager.unlocked_achievements.len());
        }
    }
    
    commands.insert_resource(achievement_manager);
}

/// Spawn the player cell with advanced biological properties
/// Includes fluid dynamics, chemical sensitivity, and evolution system
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



/// Set up environmental background elements with current indicators
pub fn setup_biological_background(mut commands: Commands) {

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

/// Load all game assets including unique enemy sprites and biological sound effects
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
        sfx_shoot: asset_server.load("audio/organic_pulse.ogg"),
        sfx_explosion: asset_server.load("audio/cell_burst.ogg"),
        sfx_powerup: asset_server.load("audio/evolution.ogg"),
        music: asset_server.load("audio/tidal_pool_ambience.ogg"),
    };
    commands.insert_resource(assets);
}

/// Initialize the fluid dynamics simulation grid for water currents
pub fn init_fluid_environment(mut commands: Commands) {
    commands.insert_resource(FluidEnvironment {
        current_field: vec![Vec2::ZERO; 64 * 64],
        grid_size: 64,
        cell_size: 20.0,
        tidal_phase: 0.0,
        turbulence_intensity: 0.3,
    });
}

/// Place initial chemical zones for pH and oxygen simulation
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



/// Reset all game state when starting a new game
/// Despawns all entities and respawns the player
pub fn reset_biological_game_state(
    mut commands: Commands,
    mut game_score: ResMut<GameScore>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut input_state: ResMut<OldInputState>,
    mut game_started: ResMut<GameStarted>,
    mut shooting_state: ResMut<ShootingState>,
    (mut fluid_environment, mut chemical_environment) : (ResMut<FluidEnvironment>,ResMut<ChemicalEnvironment>),
    mut wave_manager: ResMut<WaveManager>,
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

    wave_manager.current_wave = 1;
    wave_manager.wave_active = false;
    wave_manager.enemies_remaining = 0;
    wave_manager.wave_complete_time = 0.0;
    wave_manager.difficulty_multiplier = 1.0;

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

// fix for the fluid_dynamics_system panic
pub fn init_current_generator(mut commands: Commands) {
    commands.insert_resource(CurrentGenerator::default());
}



// System to finalize balance session data when game ends
pub fn finalize_balance_session(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    player_query: Query<(&Player, &EvolutionSystem, &ATP)>,
    game_score: Res<GameScore>,
    wave_manager: Res<WaveManager>,
    time: Res<Time>,
) {
    if let Ok((player, evolution_system, atp)) = player_query.single() {

        let mut balance_analyzer_clone = balance_analyzer.clone();
        let session = &mut balance_analyzer.real_time_balance.current_session;
        
        // Finalize session data
        session.end_time = time.elapsed_secs();
        session.final_score = game_score.current;
        session.atp_collected = game_score.total_atp_collected as u32;
        session.waves_reached = wave_manager.current_wave;
        
        // Track final evolution
        let final_evolution = evolution_system.primary_evolution.get_display_name().to_string();
        if !session.evolutions_used.contains(&final_evolution) {
            session.evolutions_used.push(final_evolution);
        }
        
        // Move session to historical data
        let completed_session = session.clone();
        

        balance_analyzer_clone.real_time_balance.historical_data.push(completed_session);
        
        // Reset for next session
        *session = BalanceSession {
            start_time: time.elapsed_secs(),
            end_time: 0.0,
            waves_reached: 0,
            evolutions_used: Vec::new(),
            atp_collected: 0,
            atp_spent: 0,
            upgrades_purchased: Vec::new(),
            deaths: 0,
            final_score: 0,
            balance_issues: Vec::new(),
        };
    }
}

// System to save balance data to file
pub fn save_balance_data_system(balance_analyzer: Res<BalanceAnalyzer>) {
    save_balance_data(&balance_analyzer);
}

// Enhanced achievement tracking for balance analysis
pub fn enhanced_balance_achievement_tracking(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    mut achievement_events: EventReader<AchievementEvent>,
    time: Res<Time>,
) {
    for event in achievement_events.read() {

        let mut balance_analyzer_clone = balance_analyzer.clone();
        let session = &mut balance_analyzer.real_time_balance.current_session;
        
        match event {
            AchievementEvent::EnemyKilled(enemy_type) => {
                // Track kills per weapon type (would need weapon identification)
                for weapon_stats in balance_analyzer.weapon_stats.values_mut() {
                    weapon_stats.kill_count += 1; // Simplified
                }
            }
            AchievementEvent::EvolutionReached(evolution_name) => {
                let unlock_time = time.elapsed_secs() - session.start_time;
                balance_analyzer.atp_economy.evolution_unlock_times.insert(
                    evolution_name.clone(), 
                    unlock_time
                );
            }
            AchievementEvent::ShotFired => {
                // Update accuracy tracking for current weapon
                if let Some(current_weapon) = session.evolutions_used.last() {
                    if let Some(weapon_stats) = balance_analyzer_clone.weapon_stats.get_mut(current_weapon) {
                        // Would need to track shots per weapon
                    }
                }
            }
            AchievementEvent::ShotHit => {
                // Similar to ShotFired but for hits
            }
            _ => {}
        }
    }
}

// Modified ATP collection system to track economy data
pub fn enhanced_atp_collection_tracking(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    mut explosion_events: EventReader<SpawnExplosion>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let mut atp_this_frame = 0u32;
    
    for event in explosion_events.read() {
        if let Some(enemy_type) = &event.enemy_type {
            // Calculate ATP that would be generated
            for &(ref stored_enemy_type, atp_amount, drop_chance) in &ATP_GENERATION_RATES {
                if std::mem::discriminant(enemy_type) == std::mem::discriminant(stored_enemy_type) {
                    if (event.position.x * 123.456).sin().abs() < drop_chance {
                        atp_this_frame += atp_amount;
                    }
                    break;
                }
            }
        }
    }
    
    if atp_this_frame > 0 {
        // Update generation rate (exponential moving average)
        let generation_rate = atp_this_frame as f32 / dt;
        balance_analyzer.atp_economy.generation_rate_per_second = 
            balance_analyzer.atp_economy.generation_rate_per_second * 0.9 + generation_rate * 0.1;
    }
}

// System to apply balance adjustments to actual game values
pub fn apply_balance_adjustments_to_gameplay(
    balance_analyzer: Res<BalanceAnalyzer>,
    mut projectile_query: Query<&mut Projectile>,
    mut evolution_query: Query<&mut EvolutionSystem>,
    mut player_query: Query<&mut Player>,
) {
    // Apply active balance adjustments to live gameplay
    for adjustment in &balance_analyzer.real_time_balance.active_adjustments {
        if !adjustment.active { continue; }
        
        match &adjustment.adjustment_type {
            AdjustmentType::DamageMultiplier => {
                // Apply damage multiplier to projectiles
                for mut projectile in projectile_query.iter_mut() {
                    if projectile.friendly {
                        projectile.damage = ((projectile.damage as f32) * adjustment.multiplier) as i32;
                    }
                }
            }
            AdjustmentType::MovementSpeed => {
                // Apply movement speed adjustment to player
                for mut player in player_query.iter_mut() {
                    player.speed *= adjustment.multiplier;
                }
            }
            AdjustmentType::InvincibilityDuration => {
                // Modify invincibility timer
                for mut player in player_query.iter_mut() {
                    if player.invincible_timer > 0.0 {
                        player.invincible_timer *= adjustment.multiplier;
                    }
                }
            }
            _ => {
                // Other adjustment types would be handled here
            }
        }
    }
}

// Balance-aware enemy spawning that considers current balance state
pub fn balance_aware_enemy_spawning(
    balance_analyzer: Res<BalanceAnalyzer>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    player_query: Query<(&EvolutionSystem, &CellularUpgrades)>,
) {
    if let Ok((evolution_system, upgrades)) = player_query.single() {
        let player_power_level = calculate_current_player_power(&evolution_system, &upgrades);
        
        // Adjust spawn rates based on balance analysis
        let economy_health = balance_analyzer.atp_economy.economy_health;
        
        // If economy is struggling, spawn more ATP-generous enemies
        if economy_health < 0.4 {
            enemy_spawner.spawn_timer *= 0.9; // Spawn enemies 10% faster for more ATP
        }
        
        // If player is overpowered, increase difficulty
        if player_power_level > 2.0 {
            enemy_spawner.spawn_timer *= 0.8; // Spawn 20% faster
        }
        
        // If weapons are underperforming, reduce spawn rate
        let average_weapon_efficiency = balance_analyzer.weapon_stats.values()
            .map(|w| w.cost_efficiency)
            .sum::<f32>() / balance_analyzer.weapon_stats.len().max(1) as f32;
            
        if average_weapon_efficiency < 1.0 {
            enemy_spawner.spawn_timer *= 1.1; // Spawn 10% slower
        }
    }
}

// Calculate current player power level for balance adjustments
fn calculate_current_player_power(
    evolution_system: &EvolutionSystem,
    upgrades: &CellularUpgrades,
) -> f32 {
    let weapon_power = match &evolution_system.primary_evolution {
        EvolutionType::CytoplasmicSpray { .. } => 1.0,
        EvolutionType::PseudopodNetwork { .. } => 1.5,
        EvolutionType::BioluminescentBeam { .. } => 2.0,
        EvolutionType::SymbioticHunters { .. } => 2.2,
        EvolutionType::EnzymeBurst { .. } => 1.8,
        EvolutionType::ToxinCloud { .. } => 2.1,
        EvolutionType::ElectricDischarge { .. } => 2.5,
    };
    
    let upgrade_power = (upgrades.damage_amplification + upgrades.movement_efficiency + upgrades.metabolic_rate) / 3.0;
    let adaptation_power = (evolution_system.cellular_adaptations.metabolic_efficiency + 
                           evolution_system.cellular_adaptations.membrane_permeability) / 2.0;
    
    weapon_power * upgrade_power * adaptation_power
}

// Enhanced weapon performance tracking that identifies specific weapons
pub fn detailed_weapon_performance_tracking(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    projectile_query: Query<&Projectile, Added<Projectile>>,
    player_query: Query<&EvolutionSystem, With<Player>>,
    mut collision_events: EventReader<EnemyHit>,
    time: Res<Time>,
) {
    if let Ok(evolution_system) = player_query.single() {
        let current_weapon = evolution_system.primary_evolution.get_display_name().to_string();
        
        // Track new projectiles fired
        let projectiles_fired = projectile_query.iter().filter(|p| p.friendly).count() as u32;
        if projectiles_fired > 0 {
            if let Some(weapon_stats) = balance_analyzer.weapon_stats.get_mut(&current_weapon) {
                // Update fire rate and usage
                weapon_stats.usage_frequency += projectiles_fired;
            }
        }
        
        // Track hits for accuracy
        let hits_this_frame = collision_events.read().count() as u32;
        if hits_this_frame > 0 {
            if let Some(weapon_stats) = balance_analyzer.weapon_stats.get_mut(&current_weapon) {
                // Update accuracy (simplified calculation)
                if weapon_stats.usage_frequency > 0 {
                    weapon_stats.accuracy_rate = hits_this_frame as f32 / weapon_stats.usage_frequency as f32;
                }
            }
        }
    }
}

// Balance-influenced power-up spawning
pub fn balance_influenced_powerup_spawning(
    balance_analyzer: Res<BalanceAnalyzer>,
    mut powerup_events: EventWriter<SpawnPowerUp>,
    player_query: Query<(&Transform, &Health, &EvolutionSystem)>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer += time.delta_secs();
    
    if *spawn_timer >= 20.0 { // Check every 20 seconds
        *spawn_timer = 0.0;
        
        if let Ok((transform, health, evolution_system)) = player_query.single() {
            let mut spawn_powerup = false;
            let mut powerup_type = PowerUpType::CellularRegeneration { amount: 30 };
            
            // Check balance issues and spawn appropriate power-ups
            for issue in &balance_analyzer.real_time_balance.current_session.balance_issues {
                match issue.issue_type {
                    BalanceIssueType::ATPStarvation => {
                        // Spawn ATP-generating power-up or reduce costs temporarily
                        powerup_type = PowerUpType::SymbioticBoost { multiplier: 2.0, duration: 15.0 };
                        spawn_powerup = true;
                        break;
                    }
                    BalanceIssueType::WeaponUnderPowered => {
                        // Spawn damage-boosting power-up
                        powerup_type = PowerUpType::MitochondriaOvercharge { rate_multiplier: 1.5, duration: 20.0 };
                        spawn_powerup = true;
                        break;
                    }
                    BalanceIssueType::ProgressionTooSlow => {
                        // Spawn speed-boosting power-up
                        powerup_type = PowerUpType::Flagella { multiplier: 1.4, duration: 15.0 };
                        spawn_powerup = true;
                        break;
                    }
                    _ => {}
                }
            }
            
            // Also check player health for emergency healing
            if health.0 < 30 {
                powerup_type = PowerUpType::CellularRegeneration { amount: 50 };
                spawn_powerup = true;
            }
            
            if spawn_powerup {
                powerup_events.send(SpawnPowerUp {
                    position: transform.translation + Vec3::new(0.0, 50.0, 0.0),
                    power_type: powerup_type,
                });
            }
        }
    }
}

// Balance metrics for achievement system integration
pub fn balance_achievements_system(
    balance_analyzer: Res<BalanceAnalyzer>,
    mut achievement_events: EventWriter<AchievementEvent>,
    mut tracked_achievements: Local<Vec<String>>,
) {
    // Track balance-related achievements
    
    // Perfect Balance achievement - maintain good balance for extended time
    if balance_analyzer.atp_economy.economy_health > 0.6 && 
       balance_analyzer.atp_economy.economy_health < 0.8 {
        if !tracked_achievements.contains(&"perfect_balance".to_string()) {
            tracked_achievements.push("perfect_balance".to_string());
            // Would trigger achievement after sustained balance
        }
    }
    
    // Weapon Master achievement - high efficiency with multiple weapons
    let high_efficiency_weapons = balance_analyzer.weapon_stats.values()
        .filter(|w| w.cost_efficiency > 2.0)
        .count();
        
    if high_efficiency_weapons >= 3 && !tracked_achievements.contains(&"weapon_master".to_string()) {
        tracked_achievements.push("weapon_master".to_string());
        achievement_events.send(AchievementEvent::EvolutionReached("WeaponMaster".to_string()));
    }
    
    // Economy Expert achievement - maintain healthy ATP economy
    if balance_analyzer.atp_economy.economy_health > 0.8 && 
       !tracked_achievements.contains(&"economy_expert".to_string()) {
        tracked_achievements.push("economy_expert".to_string());
        // Custom achievement event for economy management
    }
}

// Dynamic difficulty adjustment based on balance analysis
pub fn dynamic_difficulty_from_balance(
    balance_analyzer: Res<BalanceAnalyzer>,
    mut wave_manager: ResMut<WaveManager>,
    player_query: Query<(&Player, &Health, &EvolutionSystem)>,
    time: Res<Time>,
) {
    if let Ok((player, health, evolution_system)) = player_query.single() {
        let mut difficulty_adjustment = 1.0;
        
        // Analyze player performance vs expected performance
        let expected_wave = (time.elapsed_secs() / 45.0) as u32; // Expect 1 wave per 45 seconds
        let wave_performance = wave_manager.current_wave as f32 / expected_wave.max(1) as f32;
        
        // Check weapon efficiency
        let current_weapon = evolution_system.primary_evolution.get_display_name();
        if let Some(weapon_stats) = balance_analyzer.weapon_stats.get(current_weapon) {
            if weapon_stats.cost_efficiency > 3.0 {
                difficulty_adjustment += 0.2; // Increase difficulty if weapon is very efficient
            } else if weapon_stats.cost_efficiency < 1.0 {
                difficulty_adjustment -= 0.15; // Decrease difficulty if weapon is struggling
            }
        }
        
        // Check ATP economy health
        if balance_analyzer.atp_economy.economy_health < 0.3 {
            difficulty_adjustment -= 0.1; // Ease up if economy is struggling
        } else if balance_analyzer.atp_economy.economy_health > 0.8 {
            difficulty_adjustment += 0.1; // Ramp up if economy is too healthy
        }
        
        // Check player health trend
        let health_percentage = health.0 as f32 / 100.0;
        if health_percentage < 0.3 {
            difficulty_adjustment -= 0.05; // Slight relief for low health
        }
        
        // Apply adjustment to wave manager
        wave_manager.difficulty_multiplier = (wave_manager.difficulty_multiplier * 0.95) + (difficulty_adjustment * 0.05);
        wave_manager.difficulty_multiplier = wave_manager.difficulty_multiplier.clamp(0.5, 3.0);
    }
}

// Balance-aware evolution chamber spawning
pub fn balanced_evolution_chamber_spawning(
    balance_analyzer: Res<BalanceAnalyzer>,
    mut commands: Commands,
    player_query: Query<(&Transform, &ATP, &EvolutionSystem), With<Player>>,
    chamber_query: Query<Entity, With<EvolutionChamber>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer += time.delta_secs();
    
    // Spawn evolution chambers based on balance needs
    if *spawn_timer >= 90.0 && chamber_query.is_empty() {
        *spawn_timer = 0.0;
        
        if let Ok((transform, atp, evolution_system)) = player_query.single() {
            let should_spawn = if balance_analyzer.atp_economy.economy_health > 0.7 {
                // Player has good ATP economy, offer evolution opportunity
                true
            } else if atp.amount > 100 {
                // Player has saved up ATP, give them a chance to spend it
                true
            } else {
                // Check if current weapon is underperforming
                let current_weapon = evolution_system.primary_evolution.get_display_name();
                balance_analyzer.weapon_stats.get(current_weapon)
                    .map(|stats| stats.cost_efficiency < 1.5)
                    .unwrap_or(false)
            };
            
            if should_spawn {
                if let Some(assets) = assets {
                    commands.spawn((
                        Sprite {
                            image: assets.enemy_texture.clone(),
                            color: Color::srgb(0.3, 0.9, 0.6),
                            custom_size: Some(Vec2::splat(60.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 380.0, 0.0),
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
    }
}

// Performance monitoring for balance impact
pub fn balance_performance_monitor(
    balance_analyzer: Res<BalanceAnalyzer>,
    diagnostics: Res<DiagnosticsStore>,
    mut performance_timer: Local<f32>,
    time: Res<Time>,
) {
    *performance_timer += time.delta_secs();
    
    if *performance_timer >= 5.0 {
        *performance_timer = 0.0;
        
        if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_value) = fps.smoothed() {
                // If balance system is causing performance issues
                if fps_value < 30.0 && balance_analyzer.debug_mode {
                    println!("Warning: Balance system may be impacting performance. FPS: {:.1}", fps_value);
                    
                    // Could automatically disable some balance features here
                }
            }
        }
    }
}

// Balance data export for external analysis
pub fn export_balance_data_system(
    balance_analyzer: Res<BalanceAnalyzer>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F10) {
        let export_data = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "weapon_performance": balance_analyzer.weapon_stats,
            "atp_economy": balance_analyzer.atp_economy,
            "progression_metrics": balance_analyzer.progression_metrics,
            "current_session": balance_analyzer.real_time_balance.current_session,
            "balance_issues": balance_analyzer.real_time_balance.current_session.balance_issues
        });
        
        let filename = format!("balance_export_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        if let Ok(json_string) = serde_json::to_string_pretty(&export_data) {
            if let Err(e) = std::fs::write(&filename, json_string) {
                eprintln!("Failed to export balance data: {}", e);
            } else {
                println!("Balance data exported to: {}", filename);
            }
        }
    }
}


pub fn render_magnet_field(
    mut commands: Commands,
    player_query: Query<(&Transform, &CellularUpgrades), With<Player>>,
    existing_field_query: Query<Entity, With<MagnetFieldVisual>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        if let Ok((player_transform, upgrades)) = player_query.single() {
            // Remove old field visual
            for entity in existing_field_query.iter() {
                commands.entity(entity).despawn();
            }
            
            // Create new field visual if magnet is upgraded
            if upgrades.magnet_radius > 0.0 || upgrades.magnet_strength > 0.0 {
                let radius = 80.0 + upgrades.magnet_radius;
                let alpha = 0.1 + (upgrades.magnet_strength * 0.05).min(0.15);
                
                commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: Color::srgba(0.3, 0.8, 1.0, alpha),
                        custom_size: Some(Vec2::splat(radius * 2.0)),
                        ..default()
                    },
                    Transform::from_translation(player_transform.translation + Vec3::new(0.0, 0.0, -0.1)),
                    MagnetFieldVisual,
                ));
            }
        }
    }
}