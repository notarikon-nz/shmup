use bevy::prelude::*;
use bevy::window::WindowResolution;

mod components;
mod resources;
mod systems;
mod events;

use components::*;
use resources::*;
use systems::*;
use events::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simple Shmup".into(),
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
        .init_state::<GameState>()
        .add_event::<SpawnExplosion>()
        .add_event::<SpawnEnemy>()
        .add_event::<SpawnPowerUp>()
        .add_systems(Startup, (
            startup_debug,
            setup_camera, 
            setup_background, 
            spawn_player, 
            load_assets, 
            setup_ui.after(load_assets), 
            load_high_scores
        ))
        .add_systems(Update, (
            handle_input,
            move_player,
            spawn_projectiles,
            spawn_enemies,
            spawn_powerups,
            enemy_shooting,
            move_enemies,
            move_projectiles,
            move_powerups,
            handle_collisions,
            handle_powerup_collection,
            update_explosions,
            update_parallax,
            update_lights,
            cleanup_offscreen,
        ))
        .add_systems(Update, (
            spawn_explosion_system,
            spawn_enemy_system,
            spawn_powerup_system,
            update_health_bar,
            update_score_display,
            check_game_over,
            handle_restart_input,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui)
        .add_systems(OnEnter(GameState::Playing), reset_game_state_on_restart)
        .add_systems(Update, (
            handle_restart_button,
        ).run_if(in_state(GameState::GameOver)))
        .run();

}