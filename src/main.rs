use bevy::prelude::*;
use bevy::window::WindowResolution;

mod components;
mod resources;
mod systems;
mod events;
mod enemy_types;
mod enemy_systems;

use components::*;
use resources::*;
use systems::*;
use events::*;
use enemy_types::*;
use enemy_systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Enhanced Shmup".into(),
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
            spawn_player, 
            load_assets, 
            setup_ui.after(load_assets), 
            load_high_scores,
            init_particle_pool,
        ))
        .add_systems(Update, (
            handle_pause_input,
            handle_input.run_if(in_state(GameState::Playing)),
            move_player.run_if(in_state(GameState::Playing)),
            spawn_projectiles.run_if(in_state(GameState::Playing)),
            spawn_enemies_enhanced.run_if(in_state(GameState::Playing)),
            spawn_powerups.run_if(in_state(GameState::Playing)),
            enemy_shooting.run_if(in_state(GameState::Playing)),
            turret_shooting.run_if(in_state(GameState::Playing)),
            move_enemies.run_if(in_state(GameState::Playing)),
            update_spawner_enemies.run_if(in_state(GameState::Playing)),
        ))
        .add_systems(Update, (            
            update_formations.run_if(in_state(GameState::Playing)),
            move_projectiles.run_if(in_state(GameState::Playing)),
            move_powerups.run_if(in_state(GameState::Playing)),
            handle_collisions.run_if(in_state(GameState::Playing)),
            handle_powerup_collection.run_if(in_state(GameState::Playing)),
            update_player_effects.run_if(in_state(GameState::Playing)),
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
            handle_player_hit,
            update_health_bar,
            update_score_display,
            check_game_over,
            handle_restart_input,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::GameOver), (save_high_score_system, setup_game_over_ui).chain())
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
        .add_systems(OnEnter(GameState::Playing), reset_game_state_on_restart)
        .add_systems(OnEnter(GameState::Paused), setup_pause_ui)
        .add_systems(OnExit(GameState::Paused), cleanup_pause_ui)
        .add_systems(Update, (
            handle_restart_button,
        ).run_if(in_state(GameState::GameOver)))
        .run();
}