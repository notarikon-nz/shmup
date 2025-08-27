// src/menu_systems.rs - Minimal game states implementation
use bevy::prelude::*;
use bevy::window::{WindowMode, PrimaryWindow};
use bevy::asset::{LoadState, RecursiveDependencyLoadState};
use bevy::app::AppExit;
// use bevy::hierarchy::ChildSpawnerCommands;
use rand::Rng;

use crate::components::*;
use crate::resources::*;
use crate::input::*;
use crate::user_interface::*;

// ===== CONSTANTS =====
const LOADING_BAR_WIDTH: f32 = 400.0;
const LOADING_BAR_HEIGHT: f32 = 20.0;
const TITLE_FONT_SIZE: f32 = 72.0;
const MENU_FONT_SIZE: f32 = 32.0;
const COPYRIGHT_SIZE: f32 = 16.0;
const HIGH_SCORE_SIZE: f32 = 24.0;
const PARTICLE_COUNT: usize = 50;
const PARTICLE_SPEED: f32 = 30.0;
const VOLUME_STEP: f32 = 0.1;

/// Load game fonts for UI display
pub fn load_game_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fonts = GameFonts {
        default_font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    };
    commands.insert_resource(fonts);
}

// ===== LOADING SYSTEMS =====
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
        BackgroundColor(Color::srgb(0.05, 0.15, 0.25)),
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Loading..."),
            TextFont { font: fonts.default_font.clone(), font_size: 48.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        parent.spawn((
            Node {
                width: Val::Px(LOADING_BAR_WIDTH),
                height: Val::Px(LOADING_BAR_HEIGHT),
                margin: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor(Color::srgb(0.8, 0.8, 0.8)),
            LoadingBar,
        )).with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.4, 0.8, 1.0)),
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
    
    let mut loaded = 0;
    let total = loading.handles.len();
    
    for handle in &loading.handles {
        if matches!(asset_server.get_load_state(handle.id()), Some(LoadState::Loaded)) {
            loaded += 1;
        }
    }
    
    let progress = loaded as f32 / total as f32;
    
    if let Ok(mut fill) = fill_query.single_mut() {
        fill.width = Val::Px(LOADING_BAR_WIDTH * progress);
    }
    
    if loaded == total {
        next_state.set(GameState::TitleScreen);
    }
}

pub fn cleanup_loading(mut commands: Commands, query: Query<Entity, Or<(With<LoadingBar>, With<Node>)>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ===== TITLE SCREEN SYSTEMS =====
pub fn setup_title_screen(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    game_score: Res<GameScore>,
) {
    // Background
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.15, 0.25)),
        TitleScreen,
    )).with_children(|parent| {
        // High Score (top center)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|parent| {
            parent.spawn((
                Text::new(format!("High Score: {}", game_score.high_scores.first().unwrap_or(&0))),
                TextFont { font: fonts.default_font.clone(), font_size: HIGH_SCORE_SIZE, ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                HighScoreDisplay,
            ));
        });
        
        // Title
        parent.spawn((
            Text::new("COSMIC TIDAL POOL"),
            TextFont { font: fonts.default_font.clone(), font_size: TITLE_FONT_SIZE, ..default() },
            TextColor(Color::srgb(0.4, 0.8, 1.0)),
            Node { margin: UiRect::bottom(Val::Px(50.0)), ..default() },
        ));
        
        // Menu buttons
        spawn_menu_button(parent, "PLAY", MenuAction::Play, &fonts);
        spawn_menu_button(parent, "CONTROLS", MenuAction::Options, &fonts);
        spawn_menu_button(parent, "QUIT", MenuAction::Quit, &fonts);
        
        // Copyright (bottom left)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("© 2025 Cosmic Tidal Pool"),
                TextFont { font: fonts.default_font.clone(), font_size: COPYRIGHT_SIZE, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                CopyrightText,
            ));
        });
    });
    
    // Spawn background particles
    spawn_background_particles(&mut commands);
}

fn spawn_menu_button(parent: &mut ChildSpawnerCommands, text: &str, action: MenuAction, fonts: &GameFonts) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(50.0),
            margin: UiRect::all(Val::Px(10.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.4, 0.6)),
        BorderColor(Color::srgb(0.4, 0.8, 1.0)),
        BorderRadius::all(Val::Px(5.0)),
        MenuButton { action },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont { font: fonts.default_font.clone(), font_size: MENU_FONT_SIZE, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn spawn_background_particles(commands: &mut Commands) {
    let mut rng = rand::thread_rng();
    
    for i in 0..PARTICLE_COUNT {
        let x = (i as f32 / PARTICLE_COUNT as f32) * 1280.0;
        let y = rng.gen_range(0.0..720.0);
        let velocity = Vec2::new(
            rng.gen_range(-PARTICLE_SPEED * 0.5..PARTICLE_SPEED * 0.5),
            rng.gen_range(-PARTICLE_SPEED * 0.5..PARTICLE_SPEED * 0.5),
        );
        
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.8, 1.0, 0.3),
                custom_size: Some(Vec2::splat(rng.gen_range(1.0..4.0))),
                ..default()
            },
            Transform::from_xyz(x, y, -10.0),
            BackgroundParticle { velocity },
        ));
    }
}

pub fn menu_button_system(
    mut interaction_query: Query<(&Interaction, &MenuButton, &mut BackgroundColor), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match button.action {
                    MenuAction::Play => next_state.set(GameState::Playing),
                    MenuAction::Options => next_state.set(GameState::Controls),
                    MenuAction::Quit => { exit.send(AppExit::Success); }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.5, 0.7));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.4, 0.6));
            }
        }
    }
}

pub fn update_menu_background_particles(
    mut particle_query: Query<(&mut Transform, &mut BackgroundParticle)>,
    time: Res<Time>,
) {
    for (mut transform, mut particle) in particle_query.iter_mut() {
        transform.translation += particle.velocity.extend(0.0) * time.delta_secs();
        
        // Wrap around screen
        if transform.translation.x > 1280.0 { transform.translation.x = -10.0; }
        if transform.translation.x < -10.0 { transform.translation.x = 1280.0; }
        if transform.translation.y > 720.0 { transform.translation.y = -10.0; }
        if transform.translation.y < -10.0 { transform.translation.y = 720.0; }
        
        // Gentle drift
        let drift = Vec2::new(
            (time.elapsed_secs() * 0.5 + transform.translation.x * 0.01).sin() * 5.0,
            (time.elapsed_secs() * 0.3 + transform.translation.y * 0.01).cos() * 3.0,
        );
        particle.velocity = (particle.velocity + drift * time.delta_secs() * 0.1).clamp_length_max(PARTICLE_SPEED);
    }
}

pub fn cleanup_title_screen(
    mut commands: Commands, 
    title_query: Query<Entity, With<TitleScreen>>,
    particle_query: Query<Entity, With<BackgroundParticle>>,
) {
    for entity in title_query.iter() {
        commands.entity(entity).despawn();
    }
    
    for entity in particle_query.iter() {
        commands.entity(entity).despawn();
    }
}

// ===== CONTROLS SCREEN SYSTEMS =====
pub fn setup_controls_screen(mut commands: Commands, fonts: Res<GameFonts>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.15, 0.25)),
        ControlsScreen,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("CONTROLS"),
            TextFont { font: fonts.default_font.clone(), font_size: 48.0, ..default() },
            TextColor(Color::srgb(0.4, 0.8, 1.0)),
            Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() },
        ));
        
        let controls_text = [
            "WASD / Arrow Keys - Move",
            "Space - Shoot",
            "Shift+Space - Emergency Spore",
            "P - Pause",
            "R - Restart",
            "1-9 - Evolution Chamber Upgrades",
            "",
            "Alt+Enter - Toggle Fullscreen",
            "+/- or D-pad L/R - Volume",
            "",
            "Press ESC to return to menu",
        ];
        
        for line in controls_text {
            parent.spawn((
                Text::new(line),
                TextFont { font: fonts.default_font.clone(), font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
                Node { margin: UiRect::vertical(Val::Px(5.0)), ..default() },
            ));
        }
    });
}

pub fn controls_input_system(
    input_manager: Res<InputManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input_manager.just_pressed(InputAction::Pause) {
        next_state.set(GameState::TitleScreen);
    }
}

pub fn cleanup_controls_screen(mut commands: Commands, query: Query<Entity, With<ControlsScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ===== GLOBAL INPUT SYSTEMS =====
pub fn global_input_system(
    input_manager: Res<InputManager>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut audio_settings: ResMut<AudioMenuSettings>,
) {
    // Alt+Enter for fullscreen toggle
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
        audio_settings.volume = (audio_settings.volume + VOLUME_STEP).min(1.0);
    }
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        audio_settings.volume = (audio_settings.volume - VOLUME_STEP).max(0.0);
    }
}

// ===== ASSET LOADING HELPER =====
pub fn load_all_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = vec![
        asset_server.load::<Image>("textures/player.png").untyped(),
        asset_server.load::<Image>("textures/bullet.png").untyped(),
        asset_server.load::<Font>("fonts/planetary_contact.ttf").untyped(),
        asset_server.load::<AudioSource>("audio/organic_pulse.ogg").untyped(),
        // Add all other assets here
    ];
    commands.insert_resource(LoadingAssets { handles });
}

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
            
            // Title screen state
            .add_systems(OnEnter(GameState::TitleScreen), setup_title_screen)
            .add_systems(Update, (
                menu_button_system,
                update_menu_background_particles,
            ).run_if(in_state(GameState::TitleScreen)))
            .add_systems(OnExit(GameState::TitleScreen), (
                cleanup_title_screen,
                setup_biological_ui,
                setup_fps_ui,
            ).chain())
            
            // Controls state
            .add_systems(OnEnter(GameState::Controls), setup_controls_screen)
            .add_systems(Update, controls_input_system.run_if(in_state(GameState::Controls)))
            .add_systems(OnExit(GameState::Controls), cleanup_controls_screen);
    }
}