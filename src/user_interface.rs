use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::components::*;
use crate::resources::*;
use crate::wave_systems::*;
use crate::enemy_types::{Enemy};

// ===== CONSTANTS =====
const UI_FONT_SIZE_LARGE: f32 = 48.0;
const UI_FONT_SIZE_MEDIUM: f32 = 24.0;
const UI_FONT_SIZE_SMALL: f32 = 16.0;
const UI_FONT_SIZE_TINY: f32 = 12.0;
const UI_PADDING: f32 = 20.0;
const UI_MARGIN: f32 = 10.0;
const HEALTH_BAR_WIDTH: f32 = 200.0;
const EVOLUTION_CHAMBER_DISTANCE: f32 = 60.0;

// ===== COLOR CONSTANTS =====
const COLOR_HEALTHY: Color = Color::srgb(0.2, 0.8, 0.4);
const COLOR_BACKGROUND: Color = Color::srgb(0.1, 0.2, 0.3);
const COLOR_BORDER: Color = Color::srgb(0.4, 0.8, 0.6);
const COLOR_TEXT_PRIMARY: Color = Color::srgb(0.8, 1.0, 0.9);
const COLOR_TEXT_SECONDARY: Color = Color::srgb(0.6, 0.8, 0.7);
const COLOR_ATP: Color = Color::srgb(1.0, 1.0, 0.3);
const COLOR_WARNING: Color = Color::srgb(1.0, 0.3, 0.3);

// ===== UI SPAWN HELPERS =====
fn spawn_ui_text(parent: &mut ChildSpawnerCommands, text: &str, font: Handle<Font>, size: f32, color: Color, component: impl Component) {
    parent.spawn((
        Text::new(text),
        TextFont { font, font_size: size, ..default() },
        TextColor(color),
        component,
    ));
}

fn spawn_positioned_text(commands: &mut Commands, text: &str, font: Handle<Font>, size: f32, color: Color, 
                        position: (Val, Val, Val, Val), component: impl Component) {
    commands.spawn((
        Text::new(text),
        Node {
            position_type: PositionType::Absolute,
            left: position.0, right: position.1, top: position.2, bottom: position.3,
            ..default()
        },
        TextFont { font, font_size: size, ..default() },
        TextColor(color),
        component,
    ));
}

// ===== MAIN UI SETUP =====
pub fn setup_biological_ui(mut commands: Commands, fonts: Res<GameFonts>) {
    let font = fonts.default_font.clone();
    
    // Health bar background
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(UI_PADDING), bottom: Val::Px(60.0),
            width: Val::Px(HEALTH_BAR_WIDTH + 4.0), height: Val::Px(24.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(COLOR_BACKGROUND), BorderColor(COLOR_BORDER),
        HealthBar,
    ));
    
    // Health bar fill
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(UI_PADDING + 2.0), bottom: Val::Px(62.0),
            width: Val::Px(HEALTH_BAR_WIDTH), height: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(COLOR_HEALTHY),
        HealthBarFill,
    ));

    // Primary UI elements
    spawn_positioned_text(&mut commands, "Lives: 3", font.clone(), UI_FONT_SIZE_SMALL, COLOR_TEXT_PRIMARY,
        (Val::Px(UI_PADDING + 4.0), Val::Auto, Val::Auto, Val::Px(24.0)), LivesText);
    
    spawn_positioned_text(&mut commands, "Score: 0", font.clone(), UI_FONT_SIZE_MEDIUM, COLOR_TEXT_PRIMARY,
        (Val::Auto, Val::Px(UI_PADDING), Val::Px(UI_PADDING), Val::Auto), ScoreText);
    
    spawn_positioned_text(&mut commands, "High: 0", font.clone(), UI_FONT_SIZE_SMALL, COLOR_TEXT_SECONDARY,
        (Val::Auto, Val::Px(UI_PADDING), Val::Px(50.0), Val::Auto), HighScoreText);
    
    spawn_positioned_text(&mut commands, "ATP: 0", font.clone(), 18.0, COLOR_ATP,
        (Val::Px(UI_PADDING), Val::Auto, Val::Px(UI_PADDING), Val::Auto), ATPText);
    
    spawn_positioned_text(&mut commands, "Evolution: Cytoplasmic Spray", font.clone(), UI_FONT_SIZE_SMALL, COLOR_TEXT_SECONDARY,
        (Val::Px(UI_PADDING), Val::Auto, Val::Px(50.0), Val::Auto), EvolutionText);
    
    spawn_positioned_text(&mut commands, "Tide: Normal", font.clone(), 14.0, Color::srgb(0.6, 0.9, 1.0),
        (Val::Px(UI_PADDING), Val::Auto, Val::Px(110.0), Val::Auto), TidalStatusText);
    
    spawn_positioned_text(&mut commands, "Emergency Spores: 3", font.clone(), UI_FONT_SIZE_SMALL, Color::srgb(0.8, 0.8, 1.0),
        (Val::Px(250.0), Val::Auto, Val::Auto, Val::Px(UI_PADDING)), SporeText);
    
    spawn_positioned_text(&mut commands, "SPACE: Emergency Spore | Near Evolution Chamber: 1-9 to evolve", font.clone(), UI_FONT_SIZE_TINY, COLOR_TEXT_SECONDARY,
        (Val::Px(UI_PADDING), Val::Auto, Val::Auto, Val::Px(100.0)), ControlsText);
    
    spawn_positioned_text(&mut commands, "", font.clone(), 18.0, Color::srgb(1.0, 0.8, 0.2),
        (Val::Auto, Val::Px(UI_PADDING), Val::Px(80.0), Val::Auto), MultiplierText);
    
    spawn_positioned_text(&mut commands, "pH: 7.0 | O2: Normal", font.clone(), 14.0, Color::srgb(0.6, 0.9, 0.8),
        (Val::Px(UI_PADDING), Val::Auto, Val::Px(80.0), Val::Auto), EnvironmentText);
    
    spawn_positioned_text(&mut commands, "", font.clone(), UI_FONT_SIZE_SMALL, Color::srgb(0.4, 1.0, 0.8),
        (Val::Px(UI_PADDING), Val::Auto, Val::Auto, Val::Px(130.0)), CellWallTimerText);
    
    spawn_positioned_text(&mut commands, "Ecosystem: Healthy", font.clone(), UI_FONT_SIZE_SMALL, Color::srgb(0.4, 1.0, 0.6),
        (Val::Auto, Val::Px(UI_PADDING), Val::Auto, Val::Px(80.0)), EcosystemStatusText);
    
    spawn_positioned_text(&mut commands, "", font.clone(), 14.0, Color::srgb(1.0, 0.8, 0.3),
        (Val::Px(UI_PADDING), Val::Auto, Val::Px(140.0), Val::Auto), ContaminationWarningText);
}

// ===== OPTIMIZED UPDATE SYSTEM =====
pub fn update_biological_ui(
    game_score: Res<GameScore>,
    player_query: Query<(&Player, &ATP, &EvolutionSystem)>,
    environment: Res<ChemicalEnvironment>,
    ecosystem: Res<EcosystemState>,
    mut atp_text: Query<&mut Text, With<ATPText>>,
    mut evolution_text: Query<&mut Text, (With<EvolutionText>, Without<ATPText>)>,
    mut spore_text: Query<&mut Text, (With<SporeText>, Without<ATPText>, Without<EvolutionText>)>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>)>,
    mut high_score_text: Query<&mut Text, (With<HighScoreText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>)>,
    mut multiplier_text: Query<&mut Text, (With<MultiplierText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>)>,
    mut lives_text: Query<&mut Text, (With<LivesText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>)>,
    mut environment_text: Query<&mut Text, (With<EnvironmentText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut ecosystem_text: Query<&mut Text, (With<EcosystemStatusText>, Without<EnvironmentText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
    mut contamination_text: Query<&mut Text, (With<ContaminationWarningText>, Without<EcosystemStatusText>, Without<EnvironmentText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
) {
    // Update player-dependent UI
    if let Ok((player, atp, evolution_system)) = player_query.single() {
        if let Ok(mut text) = atp_text.single_mut() { **text = format!("ATP: {}", atp.amount); }
        if let Ok(mut text) = evolution_text.single_mut() { **text = format!("Evolution: {}", evolution_system.primary_evolution.get_display_name()); }
        if let Ok(mut text) = spore_text.single_mut() { **text = format!("Emergency Spores: {}", evolution_system.emergency_spores); }
        if let Ok(mut text) = lives_text.single_mut() { **text = format!("Lives: {}", player.lives); }
    }
    
    // Update score UI
    if let Ok(mut text) = score_text.single_mut() { **text = format!("Score: {}", game_score.current); }
    if let Ok(mut text) = high_score_text.single_mut() { 
        let high_score = game_score.high_scores.first().unwrap_or(&0);
        **text = format!("High: {}", high_score);
    }

    // Update multiplier
    if let Ok(mut text) = multiplier_text.single_mut() {
        **text = if game_score.score_multiplier > 1.0 {
            format!("{}x Symbiosis ({:.1}s)", game_score.score_multiplier, game_score.multiplier_timer)
        } else { String::new() };
    }

    // Update environment
    if let Ok(mut text) = environment_text.single_mut() {
        **text = format!("pH: {:.1} | O2: {:.0}%", environment.base_ph, environment.base_oxygen * 100.0);
    }

    // Update ecosystem status
    if let Ok(mut text) = ecosystem_text.single_mut() {
        **text = match ecosystem.health {
            h if h > 0.8 => "Ecosystem: Thriving",
            h if h > 0.6 => "Ecosystem: Stable", 
            h if h > 0.4 => "Ecosystem: Stressed",
            h if h > 0.2 => "Ecosystem: Degraded",
            _ => "Ecosystem: Critical",
        }.to_string();
    }
    
    // Update contamination warnings
    if let Ok(mut text) = contamination_text.single_mut() {
        let avg_ph = if !environment.ph_zones.is_empty() {
            environment.ph_zones.iter().map(|z| z.ph_level * z.intensity).sum::<f32>() / environment.ph_zones.len() as f32
        } else { environment.base_ph };
            
        **text = if avg_ph < 5.5 { "ACIDIC CONTAMINATION DETECTED".to_string() }
        else if avg_ph > 8.5 { "ALKALINE CONTAMINATION DETECTED".to_string() }
        else if ecosystem.infection_level > 0.7 { "HIGH PATHOGEN CONCENTRATION".to_string() }
        else { String::new() };
    }
}

// ===== HEALTH BAR UPDATE =====
pub fn update_health_bar(
    player_query: Query<(&Health, &CellularUpgrades), With<Player>>,
    mut health_fill_query: Query<&mut Node, With<HealthBarFill>>,
) {
    if let Ok((health, upgrades)) = player_query.single() {
        let percent = (health.0 as f32 / upgrades.max_health as f32).clamp(0.0, 1.0);
        if let Ok(mut fill) = health_fill_query.single_mut() {
            fill.width = Val::Px(HEALTH_BAR_WIDTH * percent);
        }
    }
}

// ===== CELL WALL TIMER =====
pub fn update_cell_wall_timer_ui(
    cell_wall_query: Query<&CellWallReinforcement>,
    mut timer_text_query: Query<&mut Text, With<CellWallTimerText>>,
) {
    if let Ok(mut text) = timer_text_query.single_mut() {
        **text = if let Ok(cell_wall) = cell_wall_query.single() {
            let remaining = cell_wall.timer.max(0.0);
            let icon = if remaining < 3.0 { "⚠️" } else { "🛡️" };
            format!("{} Cell Wall: {:.1}s", icon, remaining)
        } else { String::new() };
    }
}

// ===== TIDAL UI =====
pub fn update_tidal_ui(
    tidal_physics: Res<TidalPoolPhysics>,
    mut tidal_text_query: Query<&mut Text, With<TidalStatusText>>,
) {
    if let Ok(mut text) = tidal_text_query.single_mut() {
        let tide_strength = tidal_physics.tide_level.sin();
        **text = (if tidal_physics.king_tide_active { "KING TIDE!" }
        else if tide_strength > 0.8 { "Tide: High" }
        else if tide_strength < -0.8 { "Tide: Low" }  
        else if tide_strength > 0.0 { "Tide: Rising" }
        else { "Tide: Falling" }).to_string();
    }
}

// ===== EVOLUTION UI MANAGEMENT =====
pub fn update_evolution_ui(
    mut commands: Commands,
    chamber_query: Query<&Transform, With<EvolutionChamber>>,
    player_query: Query<(&Transform, &ATP), With<Player>>,
    existing_ui_query: Query<Entity, With<EvolutionUI>>,
    fonts: Res<GameFonts>,
) {
    if let Ok((player_transform, atp)) = player_query.single() {
        let near_chamber = chamber_query.iter().any(|chamber_transform| {
            player_transform.translation.distance(chamber_transform.translation) < EVOLUTION_CHAMBER_DISTANCE
        });

        match (near_chamber, existing_ui_query.single()) {
            (true, Err(_)) => spawn_evolution_ui(&mut commands, atp.amount, &fonts),
            (false, Ok(entity)) => { commands.entity(entity).despawn(); },
            _ => {},
        }
    }
}

fn spawn_evolution_ui(commands: &mut Commands, atp_amount: u32, fonts: &GameFonts) {
    let evolutions = [
        ("1 - Membrane Reinforcement (10 ATP)", 10, "Increases projectile damage by 20%"),
        ("2 - Metabolic Enhancement (15 ATP)", 15, "+30% movement speed & fire rate"),
        ("3 - Cellular Integrity (20 ATP)", 20, "+25 Maximum Health Points"),
        ("4 - Enzyme Production (25 ATP)", 25, "Immunity to environmental toxins"),
        ("5 - Bioluminescence (30 ATP)", 30, "Enhanced coordination abilities"),
        ("6 - Emergency Spore (20 ATP)", 20, "+1 Emergency reproductive blast"),
        ("7 - Pseudopod Network (50 ATP)", 50, "Multi-directional tendril weapon"),
        ("8 - Symbiotic Hunters (75 ATP)", 75, "Homing cooperative organisms"),
        ("9 - Bioluminescent Beam (100 ATP)", 100, "Concentrated energy discharge"),
    ];

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(UI_PADDING), top: Val::Px(120.0),
            width: Val::Px(420.0), padding: UiRect::all(Val::Px(UI_MARGIN)),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.2, 0.15, 0.95)),
        BorderColor(COLOR_BORDER),
        EvolutionUI,
    )).with_children(|parent| {
        spawn_ui_text(parent, "EVOLUTION CHAMBER", fonts.default_font.clone(), 22.0, 
                     Color::srgb(0.3, 1.0, 0.7), Node { margin: UiRect::bottom(Val::Px(UI_MARGIN)), ..default() });
        
        spawn_ui_text(parent, &format!("ATP Available: {}⚡", atp_amount), fonts.default_font.clone(), UI_FONT_SIZE_SMALL,
                     COLOR_ATP, Node { margin: UiRect::bottom(Val::Px(15.0)), ..default() });

        for (title, cost, effect) in evolutions {
            let color = if atp_amount >= cost { Color::srgb(0.9, 1.0, 0.9) } else { Color::srgb(0.5, 0.6, 0.5) };
            spawn_ui_text(parent, title, fonts.default_font.clone(), 14.0, color,
                         Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() });
            spawn_ui_text(parent, effect, fonts.default_font.clone(), UI_FONT_SIZE_TINY, Color::srgb(0.8, 0.9, 0.8),
                         Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() });
        }

        spawn_ui_text(parent, "Stand near chamber and press number keys to evolve", fonts.default_font.clone(), UI_FONT_SIZE_TINY,
                     Color::srgb(0.6, 0.9, 0.8), Node { margin: UiRect::top(Val::Px(UI_MARGIN)), ..default() });
    });
}

// ===== GAME OVER UI =====
pub fn enhanced_game_over_ui(mut commands: Commands, game_score: Res<GameScore>, fonts: Res<GameFonts>) {
    let high_score_data = game_score.high_score_data.as_ref().unwrap();
    let is_new_high_score = game_score.current > game_score.high_scores.first().cloned().unwrap_or(0);
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        GameOverUI,
    )).with_children(|parent| {
        if is_new_high_score {
            spawn_ui_text(parent, "NEW HIGH SCORE!", fonts.default_font.clone(), 52.0,
                         Color::srgb(1.0, 0.8, 0.2), Node { margin: UiRect::bottom(Val::Px(UI_MARGIN)), ..default() });
        }
        
        spawn_ui_text(parent, "CELLULAR BREAKDOWN", fonts.default_font.clone(), 42.0,
                     if is_new_high_score { Color::srgb(1.0, 0.8, 0.2) } else { COLOR_WARNING },
                     Node { margin: UiRect::bottom(Val::Px(UI_PADDING)), ..default() });
        
        spawn_ui_text(parent, &format!("Final Score: {}", game_score.current), fonts.default_font.clone(), 28.0,
                     Color::WHITE, Node { margin: UiRect::bottom(Val::Px(15.0)), ..default() });
        
        spawn_ui_text(parent, &format!("ATP Collected: {}", game_score.total_atp_collected), fonts.default_font.clone(), UI_FONT_SIZE_SMALL,
                     COLOR_ATP, Node { margin: UiRect::bottom(Val::Px(UI_MARGIN)), ..default() });
        
        spawn_ui_text(parent, &format!("Organisms Defeated: {}", game_score.enemies_defeated), fonts.default_font.clone(), UI_FONT_SIZE_SMALL,
                     Color::srgb(0.8, 1.0, 0.8), Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() });
        
        spawn_ui_text(parent, "EVOLUTION HALL OF FAME", fonts.default_font.clone(), UI_FONT_SIZE_MEDIUM,
                     Color::srgb(0.6, 1.0, 0.8), Node { margin: UiRect::bottom(Val::Px(15.0)), ..default() });
        
        let rank_colors = [Color::srgb(1.0, 0.8, 0.2), Color::srgb(0.8, 0.8, 0.8), Color::srgb(0.8, 0.5, 0.2), Color::srgb(0.7, 0.7, 0.7)];
        for (i, entry) in high_score_data.scores.iter().take(5).enumerate() {
            let grey = Color::srgb(0.7, 0.7, 0.7);
            let color = rank_colors.get(i).unwrap_or(&grey);
            spawn_ui_text(parent, &format!("{}. {} - {} ({})", i + 1, entry.score, entry.evolution_type, entry.date),
                         fonts.default_font.clone(), UI_FONT_SIZE_SMALL, *color, Node { margin: UiRect::bottom(Val::Px(5.0)), ..default() });
        }
        
        spawn_ui_text(parent, &format!("Total Games: {} | Longest Survival: {:.0}s | Best Evolution: {}",
                                      high_score_data.total_games_played, high_score_data.longest_survival_time, high_score_data.best_evolution_reached),
                     fonts.default_font.clone(), 14.0, Color::srgb(0.6, 0.8, 0.6), Node { margin: UiRect::all(Val::Px(UI_PADDING)), ..default() });
        
        // Restart button
        parent.spawn((
            Button, Node {
                width: Val::Px(200.0), height: Val::Px(50.0),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(UI_MARGIN)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.7, 0.4)),
            RestartButton,
        )).with_children(|button| {
            button.spawn((
                Text::new("EVOLVE AGAIN"),
                TextFont { font: fonts.default_font.clone(), font_size: UI_FONT_SIZE_SMALL, ..default() },
                TextColor(Color::WHITE),
            ));
        });
        
        spawn_ui_text(parent, "Press R to restart or click button above", fonts.default_font.clone(), UI_FONT_SIZE_SMALL, Color::srgb(0.7, 0.7, 0.7), Node::default());
    });
}

// ===== PAUSE UI =====
pub fn setup_pause_ui(mut commands: Commands, fonts: Res<GameFonts>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        PauseOverlay,
    )).with_children(|parent| {
        spawn_ui_text(parent, "PAUSED", fonts.default_font.clone(), 64.0, Color::WHITE, Node { margin: UiRect::bottom(Val::Px(UI_PADDING)), ..default() });
        spawn_ui_text(parent, "Press ESC to resume", fonts.default_font.clone(), UI_FONT_SIZE_MEDIUM, Color::srgb(0.8, 0.8, 0.8), Node::default());
    });
}

// ===== CLEANUP FUNCTIONS =====
pub fn cleanup_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

pub fn cleanup_pause_ui(mut commands: Commands, query: Query<Entity, With<PauseOverlay>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

// ===== FPS COUNTER =====
pub fn setup_fps_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        top: Val::Px(5.0), left: Val::Px(5.0),
        ..default()
    }).with_children(|parent| {
        spawn_ui_text(parent, "FPS: ... | ms: ... | Entities: ...", asset_server.load("fonts/FiraSans-Bold.ttf"),
                     UI_FONT_SIZE_SMALL, Color::WHITE, PerfHudText);
    });
}

pub fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    all_entities: Query<Entity>,
    mut query: Query<&mut Text, With<PerfHudText>>,
) {
    if let Ok(mut text) = query.single_mut() {
        let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed()).unwrap_or(0.0);
        let frametime = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|ft| ft.smoothed()).map(|s| s * 1000.0).unwrap_or(0.0);
        let entity_count = all_entities.iter().len();
        
        **text = format!("FPS: {:>5.0} | {:>5.0} ms | Entities: {}", fps, frametime, entity_count);
    }
}

// Wave information UI system
pub fn setup_wave_ui(mut commands: Commands, fonts: Res<GameFonts>) {
    let font = fonts.default_font.clone();
    
    // Wave information text (top right)
    commands.spawn((
        Text::new("Wave 1 - Starting..."),
        TextFont { font: font.clone(), font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.8, 1.0, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(UI_PADDING),
            top: Val::Px(UI_PADDING + 30.0),
            ..default()
        },
        WaveInfoText,
    ));
    
    // Wave progress bar background
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(UI_PADDING), 
            top: Val::Px(UI_PADDING + 60.0),
            width: Val::Px(204.0), 
            height: Val::Px(14.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(COLOR_BACKGROUND), 
        BorderColor(COLOR_BORDER),
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(0.0), 
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 1.0, 0.6)),
            WaveProgressBar,
        ));
    });
}

pub fn wave_ui_system(
    wave_manager: Res<WaveManager>,
    enemy_query: Query<&Enemy>,
    mut wave_text_query: Query<&mut Text, With<WaveInfoText>>,
    mut progress_bar_query: Query<&mut Node, With<WaveProgressBar>>,
) {
    // Update wave text
    for mut text in wave_text_query.iter_mut() {
        let enemies_remaining = enemy_query.iter().count();
        **text = if wave_manager.wave_active {
            format!("Wave {} - Enemies: {}", wave_manager.current_wave, enemies_remaining)
        } else {
            format!("Preparing Wave {}...", wave_manager.current_wave)
        };
    }

    // Update progress bar
    for mut progress_bar in progress_bar_query.iter_mut() {
        if wave_manager.wave_active && wave_manager.enemies_remaining > 0 {
            let enemies_remaining = enemy_query.iter().count() as f32;
            let initial_enemies = wave_manager.enemies_remaining as f32;
            let progress = if initial_enemies > 0.0 {
                1.0 - (enemies_remaining / initial_enemies)
            } else { 1.0 };
            progress_bar.width = Val::Px(200.0 * progress.clamp(0.0, 1.0));
        } else {
            progress_bar.width = Val::Px(0.0);
        }
    }
}