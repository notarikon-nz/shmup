use bevy::prelude::*;
use bevy::audio::*;
use std::collections::HashMap;
use crate::resources::*;
use crate::events::*;
use crate::input::*;
use crate::enemy_types::*;
use crate::achievements::{AchievementEvent};
use crate::despawn::*;
// ===== CONSTANTS =====
const MAX_CONCURRENT_SFX: usize = 20;
const MAX_EXPLOSION_SFX_PER_FRAME: u8 = 2;
const SHOOT_SFX_THROTTLE: f32 = 0.1;
const MUSIC_FADE_DURATION: f32 = 2.0;
const AUDIO_CLEANUP_THRESHOLD: usize = 30;

// ===== AUDIO CONFIGURATION =====
#[derive(Resource)]
pub struct AudioConfig {
    pub sfx_library: HashMap<SfxType, SfxData>,
    pub music_tracks: HashMap<MusicTrack, MusicData>,
    pub playlists: HashMap<PlaylistType, Vec<MusicTrack>>,
    pub base_volumes: VolumeSettings,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SfxType {
    // Player sounds
    PlayerShoot,
    PlayerDamage,
    PlayerEvolution,
    PlayerDeath,
    
    // Enemy sounds
    EnemyDeath(EnemyType),
    EnemyMovement(EnemyType),
    EnemyAttack(EnemyType),
    
    // UI sounds
    ButtonClick,
    MenuTransition,
    AchievementUnlock,
    PowerupCollect,
    
    // Environmental sounds
    TidalWave,
    CurrentFlow,
    ExplosionStandard,
    ExplosionBiological,
    ExplosionChemical,
    AtpCollect,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MusicTrack {
    MenuAmbient,
    GameplayTidalPool1,
    GameplayTidalPool2,
    GameplayTidalPool3,
    BossWave,
    GameOver,
    Victory,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaylistType {
    MainMenu,
    Gameplay,
    Boss,
}

#[derive(Clone)]
pub struct SfxData {
    pub handle: Handle<AudioSource>,
    pub base_volume: f32,
    pub priority: u8, // 0 = low, 255 = max priority
    pub max_concurrent: u8,
    pub throttle_time: f32,
}

#[derive(Clone)]
pub struct MusicData {
    pub handle: Handle<AudioSource>,
    pub base_volume: f32,
    pub loop_track: bool,
    pub duration: f32,
}

#[derive(Clone)]
pub struct VolumeSettings {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
    pub ui: f32,
}

// ===== AUDIO MANAGER =====
#[derive(Resource)]
pub struct AudioManager {
    pub current_playlist: Option<PlaylistType>,
    pub current_track: Option<MusicTrack>,
    pub track_timer: f32,
    pub track_index: usize,
    pub sfx_throttles: HashMap<SfxType, f32>,
    pub sfx_counts: HashMap<SfxType, u8>,
    pub fade_state: FadeState,
    pub pending_track: Option<MusicTrack>,
}

#[derive(Clone)]
pub enum FadeState {
    None,
    FadingOut { timer: f32, target_track: MusicTrack },
    FadingIn { timer: f32 },
}

impl Default for AudioManager {
    fn default() -> Self {
        Self {
            current_playlist: None,
            current_track: None,
            track_timer: 0.0,
            track_index: 0,
            sfx_throttles: HashMap::new(),
            sfx_counts: HashMap::new(),
            fade_state: FadeState::None,
            pending_track: None,
        }
    }
}

// ===== AUDIO COMPONENTS =====
#[derive(Component)]
pub struct ManagedAudioSource {
    pub sfx_type: Option<SfxType>,
    pub priority: u8,
    pub spawn_time: f32,
}

#[derive(Component)]
pub struct MusicPlayer {
    pub track: MusicTrack,
    pub fade_volume: f32,
}

// ===== INITIALIZATION =====
impl AudioConfig {
    pub fn new(asset_server: &AssetServer) -> Self {
        let mut sfx_library = HashMap::new();
        let mut music_tracks = HashMap::new();
        let mut playlists = HashMap::new();

        // ===== SFX LIBRARY =====
        // Player sounds
        sfx_library.insert(SfxType::PlayerShoot, SfxData {
            handle: asset_server.load("audio/organic_pulse.ogg"),
            base_volume: 0.3,
            priority: 100,
            max_concurrent: 3,
            throttle_time: SHOOT_SFX_THROTTLE,
        });

        sfx_library.insert(SfxType::PlayerDamage, SfxData {
            handle: asset_server.load("audio/cell_damage.ogg"),
            base_volume: 0.5,
            priority: 200,
            max_concurrent: 1,
            throttle_time: 0.2,
        });

        sfx_library.insert(SfxType::PlayerEvolution, SfxData {
            handle: asset_server.load("audio/evolution.ogg"),
            base_volume: 0.6,
            priority: 255,
            max_concurrent: 1,
            throttle_time: 1.0,
        });

        sfx_library.insert(SfxType::PlayerDeath, SfxData {
            handle: asset_server.load("audio/cellular_breakdown.ogg"),
            base_volume: 0.7,
            priority: 255,
            max_concurrent: 1,
            throttle_time: 0.0,
        });

        // Enemy death sounds by type
        for enemy_type in [EnemyType::ViralParticle, EnemyType::AggressiveBacteria, 
                          EnemyType::ParasiticProtozoa, EnemyType::InfectedMacrophage] {
            let audio_file = match enemy_type {
                EnemyType::ViralParticle => "audio/viral_pop.ogg",
                EnemyType::AggressiveBacteria => "audio/bacterial_burst.ogg",
                EnemyType::ParasiticProtozoa => "audio/protozoa_splash.ogg",
                EnemyType::InfectedMacrophage => "audio/macrophage_rupture.ogg",
                _ => "audio/cell_burst.ogg",
            };

            sfx_library.insert(SfxType::EnemyDeath(enemy_type), SfxData {
                handle: asset_server.load(audio_file),
                base_volume: 0.4,
                priority: 80,
                max_concurrent: 4,
                throttle_time: 0.05,
            });
        }

        // UI sounds
        sfx_library.insert(SfxType::ButtonClick, SfxData {
            handle: asset_server.load("audio/ui_click.ogg"),
            base_volume: 0.4,
            priority: 150,
            max_concurrent: 2,
            throttle_time: 0.1,
        });

        sfx_library.insert(SfxType::PowerupCollect, SfxData {
            handle: asset_server.load("audio/evolution.ogg"), // Reuse evolution sound
            base_volume: 0.5,
            priority: 120,
            max_concurrent: 2,
            throttle_time: 0.1,
        });

        // Environmental sounds
        sfx_library.insert(SfxType::TidalWave, SfxData {
            handle: asset_server.load("audio/tidal_wave.ogg"),
            base_volume: 0.6,
            priority: 180,
            max_concurrent: 1,
            throttle_time: 0.0,
        });

        sfx_library.insert(SfxType::AtpCollect, SfxData {
            handle: asset_server.load("audio/energy_absorb.ogg"),
            base_volume: 0.2,
            priority: 60,
            max_concurrent: 5,
            throttle_time: 0.02,
        });

        // Add missing SFX entries with fallbacks to existing sounds
        sfx_library.insert(SfxType::MenuTransition, SfxData {
            handle: asset_server.load("audio/ui_whoosh.ogg"),
            base_volume: 0.3,
            priority: 120,
            max_concurrent: 1,
            throttle_time: 0.2,
        });

        sfx_library.insert(SfxType::AchievementUnlock, SfxData {
            handle: asset_server.load("audio/achievement_chime.ogg"),
            base_volume: 0.5,
            priority: 200,
            max_concurrent: 1,
            throttle_time: 0.5,
        });

        sfx_library.insert(SfxType::ExplosionStandard, SfxData {
            handle: asset_server.load("audio/cell_burst.ogg"),
            base_volume: 0.5,
            priority: 100,
            max_concurrent: 3,
            throttle_time: 0.05,
        });

        sfx_library.insert(SfxType::ExplosionBiological, SfxData {
            handle: asset_server.load("audio/bio_explosion.ogg"),
            base_volume: 0.6,
            priority: 110,
            max_concurrent: 2,
            throttle_time: 0.05,
        });

        sfx_library.insert(SfxType::ExplosionChemical, SfxData {
            handle: asset_server.load("audio/chemical_burst.ogg"),
            base_volume: 0.5,
            priority: 105,
            max_concurrent: 2,
            throttle_time: 0.05,
        });

        // ===== MUSIC TRACKS =====
        music_tracks.insert(MusicTrack::MenuAmbient, MusicData {
            handle: asset_server.load("audio/menu_ambient.ogg"),
            base_volume: 0.3,
            loop_track: true,
            duration: 180.0, // 3 minutes
        });

        music_tracks.insert(MusicTrack::GameplayTidalPool1, MusicData {
            handle: asset_server.load("audio/tidal_pool_ambience.ogg"),
            base_volume: 0.25,
            loop_track: false,
            duration: 240.0, // 4 minutes
        });

        music_tracks.insert(MusicTrack::GameplayTidalPool2, MusicData {
            handle: asset_server.load("audio/deep_currents.ogg"),
            base_volume: 0.25,
            loop_track: false,
            duration: 300.0, // 5 minutes
        });

        music_tracks.insert(MusicTrack::GameplayTidalPool3, MusicData {
            handle: asset_server.load("audio/microscopic_realm.ogg"),
            base_volume: 0.25,
            loop_track: false,
            duration: 270.0, // 4.5 minutes
        });

        music_tracks.insert(MusicTrack::BossWave, MusicData {
            handle: asset_server.load("audio/cellular_warfare.ogg"),
            base_volume: 0.4,
            loop_track: true,
            duration: 120.0, // 2 minutes
        });

        music_tracks.insert(MusicTrack::GameOver, MusicData {
            handle: asset_server.load("audio/cellular_breakdown_ambient.ogg"),
            base_volume: 0.3,
            loop_track: false,
            duration: 15.0, // Short stinger
        });

        music_tracks.insert(MusicTrack::Victory, MusicData {
            handle: asset_server.load("audio/evolution_triumph.ogg"),
            base_volume: 0.4,
            loop_track: false,
            duration: 20.0, // Victory stinger
        });

        // ===== PLAYLISTS =====
        playlists.insert(PlaylistType::MainMenu, vec![MusicTrack::MenuAmbient]);
        
        playlists.insert(PlaylistType::Gameplay, vec![
            MusicTrack::GameplayTidalPool1,
            MusicTrack::GameplayTidalPool2,
            MusicTrack::GameplayTidalPool3,
        ]);
        
        playlists.insert(PlaylistType::Boss, vec![MusicTrack::BossWave]);

        Self {
            sfx_library,
            music_tracks,
            playlists,
            base_volumes: VolumeSettings {
                master: 0.7,
                music: 0.6,
                sfx: 0.8,
                ui: 0.7,
            },
        }
    }
}

// ===== CORE AUDIO SYSTEMS =====
pub fn audio_system(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut powerup_events: EventReader<SpawnPowerUp>,
    mut player_hit_events: EventReader<PlayerHit>,
    mut achievement_events: EventReader<AchievementEvent>,
    input_manager: Res<InputManager>,
    audio_config: Res<AudioConfig>,
    audio_settings: Res<AudioMenuSettings>,
    mut audio_manager: ResMut<AudioManager>,
    time: Res<Time>,
    managed_audio_query: Query<Entity, With<ManagedAudioSource>>,
) {
    let delta = time.delta_secs();
    
    // Update throttles
    for (_, throttle) in audio_manager.sfx_throttles.iter_mut() {
        *throttle -= delta;
    }
    
    // Reset frame counters
    audio_manager.sfx_counts.clear();
    
    // Clean up if too many concurrent sounds
    if managed_audio_query.iter().count() > MAX_CONCURRENT_SFX {
        return; // Skip this frame to prevent audio spam
    }

    // ===== PLAYER SOUNDS =====
    if input_manager.just_pressed(InputAction::Shoot) {
        play_sfx(&mut commands, &audio_config, &audio_settings, &mut audio_manager, 
                SfxType::PlayerShoot, time.elapsed_secs());
    }

    // Player damage
    for event in player_hit_events.read() {
        play_sfx(&mut commands, &audio_config, &audio_settings, &mut audio_manager,
                SfxType::PlayerDamage, time.elapsed_secs());
    }

    // ===== EXPLOSIONS =====
    let mut explosion_count = 0;
    for event in explosion_events.read() {
        if explosion_count >= MAX_EXPLOSION_SFX_PER_FRAME { break; }
        
        let sfx_type = if let Some(enemy_type) = &event.enemy_type {
            SfxType::EnemyDeath(enemy_type.clone())
        } else {
            SfxType::ExplosionStandard
        };
        
        play_sfx(&mut commands, &audio_config, &audio_settings, &mut audio_manager,
                sfx_type, time.elapsed_secs());
        explosion_count += 1;
    }

    // ===== POWERUPS =====
    for _ in powerup_events.read() {
        play_sfx(&mut commands, &audio_config, &audio_settings, &mut audio_manager,
                SfxType::PowerupCollect, time.elapsed_secs());
        break; // Only one per frame
    }

    // ===== ACHIEVEMENTS =====
    for _ in achievement_events.read() {
        play_sfx(&mut commands, &audio_config, &audio_settings, &mut audio_manager,
                SfxType::AchievementUnlock, time.elapsed_secs());
        break; // Only one per frame
    }
}

pub fn music_system(
    mut commands: Commands,
    audio_config: Res<AudioConfig>,
    audio_settings: Res<AudioMenuSettings>,
    mut audio_manager: ResMut<AudioManager>,
    game_state: Res<State<GameState>>,
    time: Res<Time>,
    mut music_query: Query<(Entity, &mut MusicPlayer, &mut PlaybackSettings)>,
) {
    let delta = time.delta_secs();
    audio_manager.track_timer += delta;

    // Determine target playlist based on game state
    let target_playlist = match game_state.get() {
        GameState::TitleScreen | GameState::Settings | GameState::HighScores => Some(PlaylistType::MainMenu),
        GameState::Playing => Some(PlaylistType::Gameplay),
        GameState::GameOver => None, // Let current track finish
        _ => audio_manager.current_playlist,
    };

    // Switch playlist if needed
    if target_playlist != audio_manager.current_playlist {
        if let Some(playlist) = target_playlist {
            start_playlist(&mut commands, &audio_config, &audio_settings, &mut audio_manager, playlist);
        }
    }

    // Handle track progression and fading
    handle_music_transitions(&mut commands, &audio_config, &audio_settings, &mut audio_manager, delta);
    
    // Update music volume based on settings
    update_music_volume(&mut music_query, &audio_settings);
}

pub fn audio_cleanup_system(
    mut commands: Commands,
    audio_query: Query<(Entity, &ManagedAudioSource)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    let mut entities_to_remove = Vec::new();

    // Clean up old audio entities
    for (entity, managed_audio) in audio_query.iter() {
        if current_time - managed_audio.spawn_time > 10.0 { // 10 second max lifetime
            entities_to_remove.push(entity);
        }
    }

    // Force cleanup if too many entities
    if audio_query.iter().count() > AUDIO_CLEANUP_THRESHOLD {
        let mut sorted_entities: Vec<_> = audio_query.iter().collect();
        sorted_entities.sort_by_key(|(_, managed)| (managed.priority, managed.spawn_time as u32));
        
        let excess = sorted_entities.len() - MAX_CONCURRENT_SFX;
        entities_to_remove.extend(sorted_entities.iter().take(excess).map(|(entity, _)| *entity));
    }

    for entity in entities_to_remove {
        commands.entity(entity).safe_despawn();
    }
}

// ===== HELPER FUNCTIONS =====
fn play_sfx(
    commands: &mut Commands,
    audio_config: &AudioConfig,
    audio_settings: &AudioMenuSettings,
    audio_manager: &mut AudioManager,
    sfx_type: SfxType,
    current_time: f32,
) {
    let Some(sfx_data) = audio_config.sfx_library.get(&sfx_type) else { return };
    
    // Check throttling
    if let Some(throttle) = audio_manager.sfx_throttles.get(&sfx_type) {
        if *throttle > 0.0 { return; }
    }

    // Check concurrent limit
    let current_count = audio_manager.sfx_counts.get(&sfx_type).unwrap_or(&0);
    if *current_count >= sfx_data.max_concurrent { return; }

    // Calculate final volume
    let volume_category = match sfx_type {
        SfxType::ButtonClick | SfxType::MenuTransition | SfxType::AchievementUnlock => audio_settings.sfx_volume,
        _ => audio_settings.sfx_volume,
    };
    
    let final_volume = sfx_data.base_volume * volume_category * audio_settings.master_volume;

    let sfx_type_clone = sfx_type.clone();

    // Spawn audio entity
    commands.spawn((
        AudioPlayer::new(sfx_data.handle.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(final_volume)),
        ManagedAudioSource {
            sfx_type: Some(sfx_type),
            priority: sfx_data.priority,
            spawn_time: current_time,
        },
    ));

    // Update throttle and count
    audio_manager.sfx_throttles.insert(sfx_type_clone.clone(), sfx_data.throttle_time);
    audio_manager.sfx_counts.insert(sfx_type_clone, current_count + 1);
}

fn start_playlist(
    commands: &mut Commands,
    audio_config: &AudioConfig,
    audio_settings: &AudioMenuSettings,
    audio_manager: &mut AudioManager,
    playlist: PlaylistType,
) {
    audio_manager.current_playlist = Some(playlist);
    audio_manager.track_index = 0;
    audio_manager.track_timer = 0.0;

    if let Some(tracks) = audio_config.playlists.get(&playlist) {
        if let Some(&first_track) = tracks.first() {
            start_music_track(commands, audio_config, audio_settings, audio_manager, first_track);
        }
    }
}

fn start_music_track(
    commands: &mut Commands,
    audio_config: &AudioConfig,
    audio_settings: &AudioMenuSettings,
    audio_manager: &mut AudioManager,
    track: MusicTrack,
) {
    if let Some(music_data) = audio_config.music_tracks.get(&track) {
        let final_volume = music_data.base_volume * audio_settings.music_volume * audio_settings.master_volume;
        
        let playback_settings = if music_data.loop_track {
            PlaybackSettings::LOOP.with_volume(Volume::Linear(final_volume))
        } else {
            PlaybackSettings::ONCE.with_volume(Volume::Linear(final_volume))
        };

        commands.spawn((
            AudioPlayer::new(music_data.handle.clone()),
            playback_settings,
            MusicPlayer { track, fade_volume: 1.0 },
        ));

        audio_manager.current_track = Some(track);
        audio_manager.track_timer = 0.0;
    }
}

fn handle_music_transitions(
    commands: &mut Commands,
    audio_config: &AudioConfig,
    audio_settings: &AudioMenuSettings,
    audio_manager: &mut AudioManager,
    delta: f32,
) {
    // Handle fade states
    match &mut audio_manager.fade_state {
        FadeState::FadingOut { timer, target_track } => {
            *timer += delta;
            if *timer >= MUSIC_FADE_DURATION {
                let target = *target_track;
                audio_manager.fade_state = FadeState::None;
                start_music_track(commands, audio_config, audio_settings, audio_manager, target);
            }
        }
        FadeState::FadingIn { timer } => {
            *timer += delta;
            if *timer >= MUSIC_FADE_DURATION {
                audio_manager.fade_state = FadeState::None;
            }
        }
        FadeState::None => {
            // Check if current track should advance (for playlists)
            if let (Some(current_track), Some(playlist)) = (audio_manager.current_track, audio_manager.current_playlist) {
                if let Some(music_data) = audio_config.music_tracks.get(&current_track) {
                    if !music_data.loop_track && audio_manager.track_timer >= music_data.duration {
                        advance_playlist(commands, audio_config, audio_settings, audio_manager);
                    }
                }
            }
        }
    }
}

fn advance_playlist(
    commands: &mut Commands,
    audio_config: &AudioConfig,
    audio_settings: &AudioMenuSettings,
    audio_manager: &mut AudioManager,
) {
    if let Some(playlist) = audio_manager.current_playlist {
        if let Some(tracks) = audio_config.playlists.get(&playlist) {
            audio_manager.track_index = (audio_manager.track_index + 1) % tracks.len();
            if let Some(&next_track) = tracks.get(audio_manager.track_index) {
                start_music_track(commands, audio_config, audio_settings, audio_manager, next_track);
            }
        }
    }
}

fn update_music_volume(
    music_query: &mut Query<(Entity, &mut MusicPlayer, &mut PlaybackSettings)>,
    audio_settings: &AudioMenuSettings,
) {
    for (_, music_player, mut playback) in music_query.iter_mut() {
        let target_volume = 0.3 * audio_settings.music_volume * audio_settings.master_volume * music_player.fade_volume;
        playback.volume = Volume::Linear(target_volume);
    }
}

// ===== CONVENIENCE FUNCTIONS FOR EXTERNAL USE =====
pub fn play_ui_sound(commands: &mut Commands, audio_config: &AudioConfig, audio_settings: &AudioMenuSettings, sfx_type: SfxType, current_time: f32) {
    let mut dummy_manager = AudioManager::default();
    play_sfx(commands, audio_config, audio_settings, &mut dummy_manager, sfx_type, current_time);
}

pub fn trigger_tidal_wave_audio(commands: &mut Commands, audio_config: &AudioConfig, audio_settings: &AudioMenuSettings, current_time: f32) {
    let mut dummy_manager = AudioManager::default();
    play_sfx(commands, audio_config, audio_settings, &mut dummy_manager, SfxType::TidalWave, current_time);
}

// ===== INITIALIZATION SYSTEM =====
pub fn setup_audio_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let audio_config = AudioConfig::new(&asset_server);
    commands.insert_resource(audio_config);
    commands.insert_resource(AudioManager::default());
}