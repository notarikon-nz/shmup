// src/menu_systems.rs - Complete menu system implementation
use bevy::prelude::*;
use bevy::window::{WindowMode, PrimaryWindow};
use bevy::asset::{LoadState};
use bevy::app::AppExit;
use rand::Rng;

use crate::components::*;
use crate::resources::*;
use crate::input::*;
use crate::despawn::*;

// ===== CONSTANTS =====
const LOADING_BAR_WIDTH: f32 = 400.0;
const LOADING_BAR_HEIGHT: f32 = 20.0;
const TITLE_SIZE: f32 = 72.0;
const BUTTON_SIZE: f32 = 32.0;
const SMALL_TEXT: f32 = 16.0;
const TINY_TEXT: f32 = 12.0;
const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 50.0;
const SLIDER_WIDTH: f32 = 200.0;
const SLIDER_HEIGHT: f32 = 20.0;
const PARTICLE_COUNT: usize = 50;
const PARTICLE_SPEED: f32 = 30.0;
const VOLUME_STEP: f32 = 0.1;
const UI_PADDING: f32 = 20.0;
const ANIMATION_SPEED: f32 = 3.0;

// ===== MENU COLORS =====
const BG_COLOR: Color = Color::srgb(0.05, 0.15, 0.25);
const BUTTON_NORMAL: Color = Color::srgb(0.2, 0.4, 0.6);
const BUTTON_HOVER: Color = Color::srgb(0.3, 0.5, 0.7);
const BUTTON_PRESSED: Color = Color::srgb(0.1, 0.3, 0.5);
const TEXT_COLOR: Color = Color::WHITE;
const ACCENT_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.8, 0.2);

// ===== HELPER FUNCTIONS =====
fn spawn_button(parent: &mut ChildSpawnerCommands, text: &str, action: MenuAction, font: Handle<Font>) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(BUTTON_WIDTH),
            height: Val::Px(BUTTON_HEIGHT),
            margin: UiRect::all(Val::Px(10.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        BorderColor(ACCENT_COLOR),
        BorderRadius::all(Val::Px(8.0)),
        MenuButton { action },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont { font, font_size: BUTTON_SIZE, ..default() },
            TextColor(TEXT_COLOR),
        ));
    });
}

fn spawn_text(parent: &mut ChildSpawnerCommands, text: &str, font: Handle<Font>, size: f32, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont { font, font_size: size, ..default() },
        TextColor(color),
        Node { margin: UiRect::all(Val::Px(5.0)), ..default() },
    ));
}

fn spawn_slider(parent: &mut ChildSpawnerCommands, value: f32, slider_type: SliderType, font: Handle<Font>) {
    let slider_type_clone = slider_type.clone();
    parent.spawn((
        Node {
            width: Val::Px(SLIDER_WIDTH + 100.0),
            height: Val::Px(40.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
    )).with_children(|parent| {
        // Label
        parent.spawn((
            Text::new(match slider_type {
                SliderType::Master => "Master",
                SliderType::SFX => "SFX",
                SliderType::Music => "Music",
            }),
            TextFont { font: font.clone(), font_size: SMALL_TEXT, ..default() },
            TextColor(TEXT_COLOR),
            Node { width: Val::Px(80.0), ..default() },
        ));
        
        // Slider background
        parent.spawn((
            Node {
                width: Val::Px(SLIDER_WIDTH),
                height: Val::Px(SLIDER_HEIGHT),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor(ACCENT_COLOR),
            AudioSlider { slider_type: slider_type.clone() },
        )).with_children(|parent| {
            // Slider fill
            parent.spawn((
                Node {
                    width: Val::Px(SLIDER_WIDTH * value),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(ACCENT_COLOR),
                SliderFill { slider_type },
            ));
        });
        
        // Value display
        parent.spawn((
            Text::new(format!("{:.0}%", value * 100.0)),
            TextFont { font, font_size: SMALL_TEXT, ..default() },
            TextColor(TEXT_COLOR),
            Node { width: Val::Px(50.0), ..default() },
            match slider_type_clone {
                SliderType::Master => VolumeText::Master,
                SliderType::SFX => VolumeText::SFX,
                SliderType::Music => VolumeText::Music,
            },
        ));
    });
}

// ===== NEW COMPONENTS =====
#[derive(Component)]
pub struct CursorWorldPosition {
    pub position: Vec2,
}

impl Default for CursorWorldPosition {
    fn default() -> Self { Self { position: Vec2::ZERO } }
}

// ===== LOADING SYSTEMS =====
pub fn load_game_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fonts = GameFonts {
        default_font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    };
    commands.insert_resource(fonts);
}

pub fn load_all_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = vec![
        asset_server.load::<Image>("textures/player.png").untyped(),
        asset_server.load::<Image>("textures/bullet.png").untyped(),
        asset_server.load::<Font>("fonts/FiraSans-Bold.ttf").untyped(),
        asset_server.load::<AudioSource>("audio/organic_pulse.ogg").untyped(),
        // Add other critical assets
    ];
    commands.insert_resource(LoadingAssets { handles });
}

pub fn setup_loading(mut commands: Commands, fonts: Res<GameFonts>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(BG_COLOR),
    )).with_children(|parent| {
        spawn_text(parent, "Loading Cosmic Tidal Pool...", fonts.default_font.clone(), 48.0, ACCENT_COLOR);
        
        // Loading bar container
        parent.spawn((
            Node {
                width: Val::Px(LOADING_BAR_WIDTH),
                height: Val::Px(LOADING_BAR_HEIGHT),
                margin: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor(ACCENT_COLOR),
            LoadingBar,
        )).with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(ACCENT_COLOR),
                LoadingBarFill,
            ));
        });
    });
}

pub fn loading_system(
    mut next_state: ResMut<NextState<GameState>>,
    loading_assets: Option<Res<LoadingAssets>>,
    asset_server: Res<AssetServer>,
    mut fill_query: Query<&mut Node, With<LoadingBarFill>>,
) {
    let Some(loading) = loading_assets else {
        next_state.set(GameState::TitleScreen);
        return;
    };
    
    let loaded = loading.handles.iter()
        .filter(|h| matches!(asset_server.get_load_state(h.id()), Some(LoadState::Loaded)))
        .count();
    
    let progress = loaded as f32 / loading.handles.len() as f32;
    
    if let Ok(mut fill) = fill_query.single_mut() {
        fill.width = Val::Px(LOADING_BAR_WIDTH * progress);
    }
    
    if loaded == loading.handles.len() {
        next_state.set(GameState::TitleScreen);
    }
}

pub fn cleanup_loading(mut commands: Commands, query: Query<Entity, Or<(With<LoadingBar>, With<Node>)>>) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}

// ===== TITLE SCREEN =====
pub fn setup_title_screen(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    game_score: Res<GameScore>,
) {
    // Animated background
    spawn_menu_background(&mut commands);
    
    // Main UI
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::NONE),
        TitleScreen,
    )).with_children(|parent| {
        // High Score display (top)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(UI_PADDING),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|parent| {
            let high_score = game_score.high_scores.first().unwrap_or(&0);
            spawn_text(parent, &format!("High Score: {}", high_score), fonts.default_font.clone(), 24.0, GOLD_COLOR);
        });
        
        // Title with pulsing effect
        parent.spawn((
            Text::new("COSMIC TIDAL POOL"),
            TextFont { font: fonts.default_font.clone(), font_size: TITLE_SIZE, ..default() },
            TextColor(ACCENT_COLOR),
            Node { margin: UiRect::bottom(Val::Px(50.0)), ..default() },
            PulsingText,
        ));
        
        // Menu buttons
        spawn_button(parent, "PLAY", MenuAction::Play, fonts.default_font.clone());
        spawn_button(parent, "SETTINGS", MenuAction::Settings, fonts.default_font.clone());
        spawn_button(parent, "HIGH SCORES", MenuAction::HighScores, fonts.default_font.clone());
        spawn_button(parent, "QUIT", MenuAction::Quit, fonts.default_font.clone());
        
        // Copyright (bottom)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(UI_PADDING),
                left: Val::Px(UI_PADDING),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|parent| {
            spawn_text(parent, "© 2025 Cosmic Tidal Pool", fonts.default_font.clone(), TINY_TEXT, Color::srgba(1.0, 1.0, 1.0, 0.6));
        });
    });
}

fn spawn_menu_background(commands: &mut Commands) {
    let mut rng = rand::rng();
    
    for i in 0..PARTICLE_COUNT {
        let x = (i as f32 / PARTICLE_COUNT as f32) * 1280.0;
        let y = rng.random_range(0.0..720.0);
        let velocity = Vec2::new(
            rng.random_range(-PARTICLE_SPEED * 0.5..PARTICLE_SPEED * 0.5),
            rng.random_range(-PARTICLE_SPEED * 0.5..PARTICLE_SPEED * 0.5),
        );
        
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.8, 1.0, 0.3),
                custom_size: Some(Vec2::splat(rng.random_range(1.0..4.0))),
                ..default()
            },
            Transform::from_xyz(x, y, -10.0),
            AnimatedParticle {
                velocity,
                lifetime: rng.random_range(0.0..10.0),
                pulse_phase: rng.random_range(0.0..6.28),
            },
        ));
    }
}

// ===== SETTINGS MENU =====
pub fn setup_settings_menu(mut commands: Commands, fonts: Res<GameFonts>, audio_settings: Res<AudioMenuSettings>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(BG_COLOR),
        SettingsMenu,
    )).with_children(|parent| {
        spawn_text(parent, "SETTINGS", fonts.default_font.clone(), 48.0, ACCENT_COLOR);
        
        // Audio section
        spawn_text(parent, "Audio", fonts.default_font.clone(), 32.0, TEXT_COLOR);
        spawn_slider(parent, audio_settings.master_volume, SliderType::Master, fonts.default_font.clone());
        spawn_slider(parent, audio_settings.sfx_volume, SliderType::SFX, fonts.default_font.clone());
        spawn_slider(parent, audio_settings.music_volume, SliderType::Music, fonts.default_font.clone());
        
        // Graphics section
        spawn_text(parent, "Graphics", fonts.default_font.clone(), 32.0, TEXT_COLOR);
        spawn_button(parent, "Toggle Fullscreen", MenuAction::ToggleFullscreen, fonts.default_font.clone());
        
        // Controls section
        spawn_text(parent, "Controls", fonts.default_font.clone(), 32.0, TEXT_COLOR);
        spawn_text(parent, "WASD/Arrows: Move | Space: Shoot | Shift+Space: Emergency Spore", fonts.default_font.clone(), SMALL_TEXT, Color::srgb(0.8, 0.8, 0.8));
        spawn_text(parent, "P: Pause | R: Restart | 1-9: Evolve at Chamber", fonts.default_font.clone(), SMALL_TEXT, Color::srgb(0.8, 0.8, 0.8));
        spawn_button(parent, "Reset to Default", MenuAction::ResetControls, fonts.default_font.clone());
        
        spawn_button(parent, "BACK", MenuAction::Back, fonts.default_font.clone());
    });
}

// ===== HIGH SCORES MENU =====
pub fn setup_high_scores_menu(mut commands: Commands, fonts: Res<GameFonts>, game_score: Res<GameScore>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(BG_COLOR),
        HighScoreMenu,
    )).with_children(|parent| {
        spawn_text(parent, "EVOLUTION HALL OF FAME", fonts.default_font.clone(), 48.0, ACCENT_COLOR);
        
        if let Some(high_score_data) = &game_score.high_score_data {
            // Header
            spawn_text(parent, "Rank | Score | Evolution | Date | Waves", fonts.default_font.clone(), SMALL_TEXT, Color::srgb(0.8, 0.8, 0.8));
            
            // Score entries
            let rank_colors = [GOLD_COLOR, Color::srgb(0.8, 0.8, 0.8), Color::srgb(0.8, 0.5, 0.2)];
            for (i, entry) in high_score_data.scores.iter().take(10).enumerate() {
                let color = if i < 3 { rank_colors[i] } else { TEXT_COLOR };
                let rank_text = format!("{:2}. {:>6} | {:18} | {:10} | {:2}", 
                    i + 1, entry.score, entry.evolution_type, entry.date, entry.waves_survived);
                spawn_text(parent, &rank_text, fonts.default_font.clone(), SMALL_TEXT, color);
            }
            
            // Stats summary
            spawn_text(parent, &format!("Total Games: {} | Best Evolution: {} | Longest: {:.0}s", 
                high_score_data.total_games_played, high_score_data.best_evolution_reached, high_score_data.longest_survival_time),
                fonts.default_font.clone(), TINY_TEXT, Color::srgb(0.6, 0.8, 0.6));
        } else {
            spawn_text(parent, "No scores recorded yet!", fonts.default_font.clone(), SMALL_TEXT, Color::srgb(0.8, 0.8, 0.8));
        }
        
        spawn_button(parent, "BACK", MenuAction::Back, fonts.default_font.clone());
    });
}

// ===== BUTTON INTERACTION SYSTEM =====
pub fn menu_button_system(
    mut interaction_query: Query<(&Interaction, &MenuButton, &mut BackgroundColor), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut input_manager: ResMut<InputManager>,
) {
    for (interaction, button, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match button.action {
                    MenuAction::Play => next_state.set(GameState::Playing),
                    MenuAction::Settings => next_state.set(GameState::Settings),
                    MenuAction::HighScores => next_state.set(GameState::HighScores),
                    MenuAction::Back => next_state.set(GameState::TitleScreen),
                    MenuAction::Quit => { exit.write(AppExit::Success); }
                    MenuAction::ToggleFullscreen => {
                        if let Ok(mut window) = windows.single_mut() {
                            window.mode = match window.mode {
                                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                                _ => WindowMode::Windowed,
                            };
                        }
                    },
                    MenuAction::ResetControls => {
                        input_manager.setup_default_bindings();
                    },
                    _ => {},
                }
                *color = BackgroundColor(BUTTON_PRESSED);
            }
            Interaction::Hovered => *color = BackgroundColor(BUTTON_HOVER),
            Interaction::None => *color = BackgroundColor(BUTTON_NORMAL),
        }
    }
}

// ===== SLIDER INTERACTION SYSTEM =====
pub fn audio_slider_system(
    mut slider_query: Query<(&Interaction, &AudioSlider, &Node), Changed<Interaction>>,
    mut fill_query: Query<&mut Node, (With<SliderFill>, Without<AudioSlider>)>,
    mut volume_text_query: Query<&mut Text, With<VolumeText>>,
    mut audio_settings: ResMut<AudioMenuSettings>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !mouse_input.pressed(MouseButton::Left) { return; }
    
    let Ok(window) = window_query.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    
    for (interaction, slider, _node) in slider_query.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        
        if let Ok((camera, camera_transform)) = camera_query.single() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                let slider_progress = ((world_pos.x + SLIDER_WIDTH * 0.5) / SLIDER_WIDTH).clamp(0.0, 1.0);
                
                // Update audio settings
                match slider.slider_type {
                    SliderType::Master => audio_settings.master_volume = slider_progress,
                    SliderType::SFX => audio_settings.sfx_volume = slider_progress,
                    SliderType::Music => audio_settings.music_volume = slider_progress,
                }
                
                // Update visuals
                for mut fill_node in fill_query.iter_mut() {
                    fill_node.width = Val::Px(SLIDER_WIDTH * slider_progress);
                }
                
                for mut text in volume_text_query.iter_mut() {
                    **text = format!("{:.0}%", slider_progress * 100.0);
                }
            }
        }
    }
}

// ===== ANIMATION SYSTEMS =====
pub fn update_menu_animations(
    mut particle_query: Query<(&mut Transform, &mut AnimatedParticle, &mut Sprite)>,
    mut text_query: Query<&mut TextColor, With<PulsingText>>,
    time: Res<Time>,
) {
    let time_secs = time.elapsed_secs();
    
    // Update background particles
    for (mut transform, mut particle, mut sprite) in particle_query.iter_mut() {
        // Move particles
        transform.translation += particle.velocity.extend(0.0) * time.delta_secs();
        
        // Wrap around screen
        if transform.translation.x > 1280.0 { transform.translation.x = -10.0; }
        if transform.translation.x < -10.0 { transform.translation.x = 1280.0; }
        if transform.translation.y > 720.0 { transform.translation.y = -10.0; }
        if transform.translation.y < -10.0 { transform.translation.y = 720.0; }
        
        // Bioluminescent pulse
        particle.pulse_phase += time.delta_secs() * ANIMATION_SPEED;
        let pulse = (particle.pulse_phase).sin() * 0.3 + 0.7;
        sprite.color.set_alpha(pulse * 0.3);
        
        // Gentle organic drift
        let drift = Vec2::new(
            (time_secs * 0.5 + particle.pulse_phase).sin() * 5.0,
            (time_secs * 0.3 + particle.pulse_phase * 1.1).cos() * 3.0,
        );
        particle.velocity = (particle.velocity + drift * time.delta_secs() * 0.1).clamp_length_max(PARTICLE_SPEED);
    }
    
    // Pulse title text
    for mut text_color in text_query.iter_mut() {
        let pulse = (time_secs * 2.0).sin() * 0.2 + 0.8;
        text_color.0 = Color::srgb(0.4 * pulse, 0.8 * pulse, 1.0);
    }
}

// ===== CLEANUP SYSTEMS =====
pub fn cleanup_title_screen(mut commands: Commands, query: Query<Entity, Or<(With<TitleScreen>, With<AnimatedParticle>)>>) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}

pub fn cleanup_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}

pub fn cleanup_high_scores_menu(mut commands: Commands, query: Query<Entity, With<HighScoreMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).try_despawn();
    }
}

// ===== GLOBAL INPUT HANDLING =====
pub fn global_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut audio_settings: ResMut<AudioMenuSettings>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Alt+Enter for fullscreen
    if keyboard.just_pressed(KeyCode::Enter) && (keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight)) {
        if let Ok(mut window) = windows.single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                _ => WindowMode::Windowed,
            };
        }
    }
    
    // Volume control
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        audio_settings.master_volume = (audio_settings.master_volume + VOLUME_STEP).min(1.0);
    }
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        audio_settings.master_volume = (audio_settings.master_volume - VOLUME_STEP).max(0.0);
    }
    
    // ESC key navigation
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Settings | GameState::HighScores => next_state.set(GameState::TitleScreen),
            _ => {}
        }
    }
}

// ===== NEW COMPONENTS FOR ANIMATIONS =====
#[derive(Component)]
pub struct PulsingText;

// ===== ENHANCED GAME STATES =====
// GameState enum is now properly defined in resources.rs

// ===== PLUGIN =====
pub struct MenuSystemsPlugin;

impl Plugin for MenuSystemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AudioMenuSettings>()
            .add_systems(Update, global_input_system)
            
            // Loading state
            .add_systems(OnEnter(GameState::Loading), (load_game_fonts, setup_loading, load_all_assets).chain())
            .add_systems(Update, loading_system.run_if(in_state(GameState::Loading)))
            .add_systems(OnExit(GameState::Loading), cleanup_loading)
            
            // Title screen
            .add_systems(OnEnter(GameState::TitleScreen), setup_title_screen)
            .add_systems(Update, (menu_button_system, update_menu_animations).run_if(in_state(GameState::TitleScreen)))
            .add_systems(OnExit(GameState::TitleScreen), cleanup_title_screen)
            
            // Settings menu
            .add_systems(OnEnter(GameState::Settings), setup_settings_menu)
            .add_systems(Update, (menu_button_system, audio_slider_system).run_if(in_state(GameState::Settings)))
            .add_systems(OnExit(GameState::Settings), cleanup_settings_menu)
            
            // High scores
            .add_systems(OnEnter(GameState::HighScores), setup_high_scores_menu)
            .add_systems(Update, menu_button_system.run_if(in_state(GameState::HighScores)))
            .add_systems(OnExit(GameState::HighScores), cleanup_high_scores_menu);
    }
}