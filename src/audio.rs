use bevy::prelude::*;
use bevy::audio::*;
use crate::resources::*;
use crate::events::*;

pub fn audio_system(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut powerup_events: EventReader<SpawnPowerUp>,
    input_state: Res<InputState>,
    assets: Option<Res<GameAssets>>,
    mut last_shoot_audio: Local<f32>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        *last_shoot_audio -= time.delta_secs();
        
        // Shooting sounds
        if input_state.shooting && *last_shoot_audio <= 0.0 {
            commands.spawn((
                AudioPlayer::new(assets.sfx_shoot.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.3)),
            ));
            *last_shoot_audio = 0.1;
        }

        // Explosion sounds - limit to one per frame
        if let Some(_event) = explosion_events.read().next() {
            commands.spawn((
                AudioPlayer::new(assets.sfx_explosion.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.5)),
            ));
            explosion_events.clear();
        }

        // Power-up sounds - limit to one per frame
        if let Some(_event) = powerup_events.read().next() {
            commands.spawn((
                AudioPlayer::new(assets.sfx_powerup.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.4)),
            ));
        }
    }
}




// NEXT WE NEED A PLAYLIST SYSTEM
pub fn start_ambient_music(mut commands: Commands, assets: Option<Res<GameAssets>>) {
    if let Some(assets) = assets {
        commands.spawn((
            AudioPlayer::new(assets.music.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.2)),
        ));
    }
}
