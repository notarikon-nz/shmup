use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy::input::gamepad::*;
use bevy::window::WindowResolution;
use bevy::sprite::Anchor;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

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
mod user_interface;
mod debug;
mod achievements;
mod background;
mod tidal_feedback;
mod input;

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
use user_interface::*;
use achievements::*;
use debug::*;
use background::*;
use tidal_feedback::*;
use input::*;


#[derive(Component)]
struct PerfHudText;

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
        .add_plugins(LogDiagnosticsPlugin::default()) // For GPU display
        .add_plugins(input::InputPlugin)        // Remappable input (keyboard, gamepad)

        .insert_resource(ClearColor(Color::srgb(0.05, 0.15, 0.25))) // Deep ocean background

        // ===== CORE GAME RESOURCES =====
        .init_resource::<OldInputState>()        // Legacy Input (Temporary)
        .init_resource::<EnemySpawner>()         // Enemy wave spawning system
        .init_resource::<GameScore>()            // Score tracking and high scores
        .init_resource::<GameStarted>()          // Game initialization flag
        .init_resource::<ShootingState>()        // Weapon firing rate modifiers
        .init_resource::<ScreenShakeResource>()  // Screen shake for impact feedback
        .init_resource::<AudioChannels>()

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

        // ===== STARTUP SYSTEMS (Run Once) =====
        .add_systems(Startup, (
            setup_camera,                    // Initialize 2D camera with orthographic projection
            setup_biological_background,    // Spawn current indicators and environmental elements
            spawn_biological_player,        // Create player cell with fluid dynamics
            load_biological_assets,         // Load unique enemy sprites and audio
            load_game_fonts,                // Load custom game font
            load_high_scores_from_file,     // Load persistent high score data
            init_particle_pool,             // Pre-allocate particle system
            init_fluid_environment,         // Initialize water current simulation
            init_chemical_zones,            // Place initial pH and oxygen zones
            init_current_generator,         // Set up thermal vents and major currents
            setup_achievement_system,       // Initialize Steam-ready achievements
            init_procedural_background,     // Set up dynamic background generation
            start_ambient_music.after(load_biological_assets), // Begin ocean ambience
        ))
        .add_systems(Startup, (
            setup_biological_ui,            // Create UI with biological terminology
            setup_fps_ui,
        ).after(load_game_fonts))

        // ===== CORE GAME LOOP SYSTEMS =====
        .add_systems(Update, (
            audio_system,           // Play sound effects for shooting, explosions
            // handle_pause_input,     // ESC/P key pause toggle
            fps_text_update_system,
        ))

        // ===== PRIMARY GAMEPLAY SYSTEMS (Playing State Only) =====
        .add_systems(Update, (

            // Core player and enemy interaction
            handle_input_legacy,             // Process keyboard/gamepad input
            biological_movement_system,      // Player movement with fluid dynamics
            enhanced_shooting_system,        // Evolution-based weapon systems
            spawn_enemies,                   // Wave-based enemy spawning
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
            chemical_trail_system,          // Pheromone tracking for AI
            chemical_trail_following,       // Enemies follow chemical trails
            ecosystem_balance_system,       // Population dynamics simulation

            // Environmental storytelling
            enhanced_coral_system,          // Dynamic coral health and corruption
            contamination_visualization_system, // Show pollution effects
            microscopic_debris_system,      // Story fragments in debris
            bioluminescent_warning_system,  // Emergency lighting for dangers
            environmental_narrative_system, // Dynamic environmental storytelling
        ).run_if(in_state(GameState::Playing)))

        // ===== SPECIAL EFFECT SYSTEMS =====
        .add_systems(Update, (
            spawn_extra_life_powerups,      // Rare life-extending power-ups
            extra_life_collection_system,   // Handle life gain with celebration
            update_dynamic_lights,          // Bioluminescent lighting effects
            render_light_effects,           // Convert lighting to visual sprites
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
            update_current_field,           // Current indicator visualization
            organic_ai_system,              // Biological AI behaviors (chemotaxis, etc.)
            generate_procedural_currents,   // Dynamic current pattern generation
            cell_division_system,           // Enemy reproduction mechanics
            symbiotic_pair_system,          // Paired organism death mechanics
            thermal_vent_effects_system,    // Heat effects and thermal particles
            dynamic_chemical_zone_system,   // Adaptive chemical zone spawning
            scroll_thermal_vents,           // Move thermal vents with current
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
            apply_chemical_damage_system,   // pH and oxygen damage to entities
            pheromone_communication_system, // Colony chemical coordination
            ecosystem_monitoring_system,    // Track ecosystem health metrics
        ).run_if(in_state(GameState::None)))

        // ===== ENEMY VISUAL ANIMATION SYSTEMS =====
        .add_systems(Update, (
            signal_particle_spawning,       // Coordination indicators for swarm cells
            virus_pulsing_animation,        // Viral particle size pulsing
            bacteria_flagella_animation,    // Bacterial undulation movement
            corruption_color_shift,         // Color changes for corrupted enemies
            warning_flash_animation,        // Warning flashes for dangerous enemies
            offspring_wiggle_animation,     // Juvenile organism movement
            pseudopod_animation,            // Protozoa shape-shifting animation
            gestation_animation,            // Reproductive vesicle growth animation
            toxic_aura_animation,           // Biofilm colony toxin particle emission
        ).run_if(in_state(GameState::Playing)))

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
        ).run_if(in_state(GameState::Playing)))

        // ===== DEBUG SYSTEMS (Development Only) =====
        .add_systems(Update, (
            debug_atp_spawner,              // F2: Spawn 1000 ATP for testing
            debug_spawn_evolution_chamber,  // F3: Spawn evolution chamber
            debug_trigger_king_tide,        // F4: Force trigger king tide event
        ).run_if(in_state(GameState::Playing)))

        // ===== GAME STATE TRANSITION SYSTEMS =====
        
        // When transitioning TO game over state
        .add_systems(OnEnter(GameState::GameOver), (
            save_high_score_to_file,        // Persist high score data
            enhanced_game_over_ui            // Show detailed stats and high score table
        ).chain()) // Ensure save happens before UI
        
        // When leaving game over state
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
        
        // When starting/restarting a game
        .add_systems(OnEnter(GameState::Playing), reset_biological_game_state)
        
        // Pause state management
        .add_systems(OnEnter(GameState::Paused), setup_pause_ui)
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
pub fn setup_biological_background(mut commands: Commands, asset_server: Res<AssetServer>) {

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

/// Load game fonts for UI display
pub fn load_game_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fonts = GameFonts {
        default_font: asset_server.load("fonts/planetary_contact.ttf"),
    };
    commands.insert_resource(fonts);
}


/// Enhanced player movement with fluid dynamics and organic motion
pub fn biological_movement_system(
    mut player_query: Query<(&mut Transform, &mut FluidDynamics, &Player)>,
    input_manager: Res<InputManager>, // Changed from InputState
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut fluid, player)) = player_query.single_mut() {
        // Get movement vector from input manager
        let movement = input_manager.movement_vector(); // Smooth analog movement
        
        // Player input creates thrust against fluid resistance
        let thrust = movement * player.speed * 2.0;
        
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
        let target_roll = -movement.x * player.roll_factor + flow_influence;
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
    input_manager: Res<InputManager>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut trail_segments: Local<Vec<Vec3>>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer -= time.delta_secs();
    
    if *spawn_timer <= 0.0 {
        for transform in player_query.iter() {
            let intensity = input_manager.movement_vector().length().max(0.2);
            
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



// fix for the fluid_dynamics_system panic
pub fn init_current_generator(mut commands: Commands) {
    commands.insert_resource(CurrentGenerator::default());
}


fn setup_fps_ui(mut commands: Commands, asset_server: Res<AssetServer>) {

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("FPS: ... | ms: ... | Entities: ..."),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PerfHudText,
            ));
        });
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    all_entities: Query<Entity>, // ✅ safe way to count entities
    mut query: Query<&mut Text, With<PerfHudText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        // FPS
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .unwrap_or(0.0);

        // Frame time (ms)
        let frametime = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|ft| ft.smoothed())
            .map(|s| s * 1000.0)
            .unwrap_or(0.0);

        // Entity count
        let entity_count = all_entities.iter().len();

        *text = Text::new(format!(
            "FPS: {:>5.0} | {:>5.0} ms | Entities: {}",
            fps, frametime, entity_count
        ));
    }
}