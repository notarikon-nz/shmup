use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::input::*;

pub fn debug_atp_spawner(
    mut commands: Commands,
    input_manager: Res<InputManager>, // Changed from keyboard
    assets: Option<Res<GameAssets>>,
) {
    if !input_manager.debug_enabled { return; }
    
    if input_manager.just_pressed(InputAction::DebugSpawnATP) {
        if let Some(assets) = assets {
            for i in 0..20 {
                let x = (i as f32 - 10.0) * 30.0;
                commands.spawn((
                    Sprite {
                        image: assets.multiplier_powerup_texture.clone(),
                        color: Color::srgb(1.0, 1.0, 0.3),
                        custom_size: Some(Vec2::splat(18.0)),
                        ..default()
                    },
                    Transform::from_xyz(x, 200.0, 0.0),
                    ATP { amount: 50 },
                    Collider { radius: 9.0 },
                ));
            }
        }
    }
}

pub fn debug_spawn_evolution_chamber(
    mut commands: Commands,
    input_manager: Res<InputManager>,
    assets: Option<Res<GameAssets>>,
) {
    if !input_manager.debug_enabled { return; }
    
    if input_manager.just_pressed(InputAction::DebugSpawnEvolutionChamber) {
        if let Some(assets) = assets {
            commands.spawn((
                Sprite {
                    image: assets.enemy_texture.clone(),
                    color: Color::srgb(0.3, 0.9, 0.6),
                    custom_size: Some(Vec2::splat(60.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 200.0, 0.0),
                EvolutionChamber,
            ));
        }
    }
}

pub fn debug_trigger_king_tide(
    mut commands: Commands,
    input_manager: Res<InputManager>,
    mut tidal_events: EventWriter<TidalEvent>,
) {
    if !input_manager.debug_enabled { return; }
    
    if input_manager.just_pressed(InputAction::DebugTriggerKingTide) {
        tidal_events.write(TidalEvent::KingTideBegin {
            intensity: 2.0,
            duration: 15.0,
        });
    }
}
