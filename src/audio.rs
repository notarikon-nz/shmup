use bevy::prelude::*;
use bevy::audio::*;
use crate::resources::*;
use crate::events::*;

#[derive(Resource)]
pub struct AudioChannels {
    explosion_count: u8,
    shoot_time: f32,
}

impl Default for AudioChannels {
    fn default() -> Self {
        Self {
            explosion_count: 0,
            shoot_time: 0.0,
        }
    }
}

pub fn audio_system(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut powerup_events: EventReader<SpawnPowerUp>,
    input_state: Res<InputState>,
    assets: Option<Res<GameAssets>>,
    mut channels: ResMut<AudioChannels>,
    time: Res<Time>,
    audio_query: Query<Entity, With<AudioPlayer>>,
) {
    let Some(assets) = assets else { return };
    
    // Clean up finished audio entities if too many
    let audio_count = audio_query.iter().count();
    if audio_count > 20 {
        // Only spawn new sounds if under limit
        return;
    }
    
    channels.shoot_time -= time.delta_secs();
    
    // Shooting - throttled
    if input_state.shooting && channels.shoot_time <= 0.0 {
        commands.spawn((
            AudioPlayer::new(assets.sfx_shoot.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.3)),
        ));
        channels.shoot_time = 0.1;
    }
    
    // Explosions - max 2 per frame
    channels.explosion_count = 0;
    for _ in explosion_events.read() {
        if channels.explosion_count < 2 {
            commands.spawn((
                AudioPlayer::new(assets.sfx_explosion.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.5)),
            ));
            channels.explosion_count += 1;
        }
    }
    
    // PowerUps - max 1 per frame
    if powerup_events.read().next().is_some() {
        commands.spawn((
            AudioPlayer::new(assets.sfx_powerup.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.4)),
        ));
        powerup_events.clear();
    }
}

pub fn start_ambient_music(mut commands: Commands, assets: Option<Res<GameAssets>>) {
    if let Some(assets) = assets {
        commands.spawn((
            AudioPlayer::new(assets.music.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.2)),
        ));
    }
}

// Add this system to clean up old audio entities
pub fn cleanup_audio(
    mut commands: Commands,
    audio_query: Query<(Entity, &AudioPlayer)>,
) {
    let count = audio_query.iter().count();
    if count > 30 {
        // Force cleanup of oldest audio entities
        for (entity, _) in audio_query.iter().take(count - 20) {
            commands.entity(entity).despawn();
        }
    }
}