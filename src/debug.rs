use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::systems::*;
use crate::events::*;

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