use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::sprite::Anchor;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin};

use crate::components::*;
use crate::resources::*;
use crate::systems::*;
use crate::events::*;
use crate::high_scores::*;

// Biological UI setup with updated terminology
pub fn setup_biological_ui(mut commands: Commands) {
    // Cellular integrity bar (health bar)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(60.0),
            width: Val::Px(204.0),
            height: Val::Px(24.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
        BorderColor(Color::srgb(0.4, 0.8, 0.6)),
        HealthBar,
    ));
    
    // Cellular integrity fill
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(22.0),
            bottom: Val::Px(62.0),
            width: Val::Px(200.0),
            height: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.8, 0.4)), // Healthy green
        HealthBarFill,
    ));

    // Lives text
    commands.spawn((
        Text::new("Lives: 3"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.8, 1.0, 0.9)),
        LivesText,
    ));

    // Score text
    commands.spawn((
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgb(0.8, 1.0, 0.9)),
        ScoreText,
    ));
    
    // High score text
    commands.spawn((
        Text::new("High: 0"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(50.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.6, 0.8, 0.7)),
        HighScoreText,
    ));

    // ATP text (currency)
    commands.spawn((
        Text::new("ATP: 0⚡"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.3)),
        ATPText,
    ));

    // Evolution info text
    commands.spawn((
        Text::new("Evolution: Cytoplasmic Spray"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(50.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.7, 1.0, 0.7)),
        EvolutionText,
    ));

    // Tidal State
    commands.spawn((
        Text::new("Tide: Normal"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(110.0),
            ..default()
        },
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.6, 0.9, 1.0)),
        TidalStatusText,
    ));

    // Emergency spore counter
    commands.spawn((
        Text::new("Emergency Spores: 3"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(250.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 1.0)),
        SporeText,
    ));

    // Controls help
    commands.spawn((
        Text::new("SPACE: Emergency Spore | Near Evolution Chamber: 1-9 to evolve"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(100.0),
            ..default()
        },
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.5, 0.7, 0.6)),
        ControlsText,
    ));

    // Symbiotic multiplier text
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(80.0),
            ..default()
        },
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        MultiplierText,
    ));

    // Environmental status (new)
    commands.spawn((
        Text::new("pH: 7.0 | O₂: Normal"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(80.0),
            ..default()
        },
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.6, 0.9, 0.8)),
        EnvironmentText,
    ));

    // set up cellwall timer
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(130.0),
            ..default()
        },
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.4, 1.0, 0.8)),
        CellWallTimerText,
    ));

    commands.spawn((
        Text::new("FPS: --"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0), // Changed from bottom: Val::Px(40.0)
            ..default()
        },
        TextFont { 
            font_size: 18.0, // Increased size
            ..default() 
        },
        TextColor(Color::srgb(0.9, 1.0, 0.9)), // Brighter color
        FpsText,
    ));  
}

// Enhanced UI update system with biological terminology
pub fn update_biological_ui(
    game_score: Res<GameScore>,
    player_query: Query<(&Player, &ATP, &EvolutionSystem)>,
    environment: Res<ChemicalEnvironment>,
    mut atp_query: Query<&mut Text, (With<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut evolution_query: Query<&mut Text, (With<EvolutionText>, Without<ATPText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut spore_query: Query<&mut Text, (With<SporeText>, Without<ATPText>, Without<EvolutionText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut high_score_query: Query<&mut Text, (With<HighScoreText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<MultiplierText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut multiplier_query: Query<&mut Text, (With<MultiplierText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<LivesText>, Without<EnvironmentText>)>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<EnvironmentText>)>,
    mut environment_query: Query<&mut Text, (With<EnvironmentText>, Without<ATPText>, Without<EvolutionText>, Without<SporeText>, Without<ScoreText>, Without<HighScoreText>, Without<MultiplierText>, Without<LivesText>)>,
) {
    if let Ok((player, atp, evolution_system)) = player_query.single() {
        // Update ATP display
        if let Ok(mut atp_text) = atp_query.single_mut() {
            **atp_text = format!("ATP: {}⚡", atp.amount);
        }
        
        // Update evolution display
        if let Ok(mut evolution_text) = evolution_query.single_mut() {
            **evolution_text = format!("Evolution: {}", evolution_system.primary_evolution.get_display_name());
        }
        
        // Update emergency spore counter
        if let Ok(mut spore_text) = spore_query.single_mut() {
            **spore_text = format!("Emergency Spores: {}", evolution_system.emergency_spores);
        }
        
        // Update lives
        if let Ok(mut lives_text) = lives_query.single_mut() {
            **lives_text = format!("Lives: {}", player.lives);
        }
    }
    
    // Update score
    if let Ok(mut score_text) = score_query.single_mut() {
        **score_text = format!("Score: {}", game_score.current);
    }
    
    // Update high score
    if let Ok(mut high_score_text) = high_score_query.single_mut() {
        let high_score = game_score.high_scores.first().unwrap_or(&0);
        **high_score_text = format!("High: {}", high_score);
    }

    // Update symbiotic multiplier
    if let Ok(mut multiplier_text) = multiplier_query.single_mut() {
        if game_score.score_multiplier > 1.0 {
            **multiplier_text = format!("{}x Symbiosis ({:.1}s)", game_score.score_multiplier, game_score.multiplier_timer);
        } else {
            **multiplier_text = String::new(); // This should clear it
        }
    }

    // Update environment status
    if let Ok(mut env_text) = environment_query.single_mut() {
        **env_text = format!("pH: {:.1} | O₂: {:.0}%", 
            environment.base_ph, 
            environment.base_oxygen * 100.0
        );
    }
}

pub fn update_cell_wall_timer_ui(
    cell_wall_query: Query<&CellWallReinforcement>,
    mut timer_text_query: Query<&mut Text, With<CellWallTimerText>>,
) {
    if let Ok(mut text) = timer_text_query.single_mut() {
        if let Ok(cell_wall) = cell_wall_query.single() {
            let remaining = cell_wall.timer.max(0.0);
            let color_intensity = if remaining < 3.0 { "⚠️" } else { "🛡️" };
            **text = format!("{} Cell Wall: {:.1}s", color_intensity, remaining);
        } else {
            **text = String::new();
        }
    }
}

// Tidal UI
pub fn update_tidal_ui(
    tidal_physics: Res<TidalPoolPhysics>,
    mut tidal_text_query: Query<&mut Text, With<TidalStatusText>>,
) {
    if let Ok(mut text) = tidal_text_query.single_mut() {
        let tide_strength = tidal_physics.tide_level.sin();
        let status = if tidal_physics.king_tide_active {
            "🌊 KING TIDE! 🌊"
        } else if tide_strength > 0.8 {
            "Tide: High"
        } else if tide_strength < -0.8 {
            "Tide: Low"
        } else if tide_strength > 0.0 {
            "Tide: Rising"
        } else {
            "Tide: Falling"
        };
        **text = status.to_string();
    }
}

pub fn setup_game_over_ui(
    mut commands: Commands,
    mut game_score: ResMut<GameScore>,
) {
    save_high_score(&mut game_score);
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GameOverUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::srgb(1.0, 0.3, 0.3)),
            GameOverText,
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new(format!("Final Score: {}", game_score.current)),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
            FinalScoreText,
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        
        let high_score = game_score.high_scores.first().unwrap_or(&0);
        parent.spawn((
            Text::new(format!("High Score: {}", high_score)),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
        ));
        
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            RestartButton,
        )).with_children(|button| {
            button.spawn((
                Text::new("RESTART"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
        
        parent.spawn((
            Text::new("Press R to restart or click button above"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

pub fn cleanup_game_over_ui(
    mut commands: Commands,
    game_over_query: Query<Entity, (With<GameOverUI>, Without<AlreadyDespawned>)>,
) {
    for entity in game_over_query.iter() {
        commands.entity(entity)
            .try_insert(AlreadyDespawned)
            .despawn();
    }
}

pub fn setup_pause_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        PauseOverlay,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("PAUSED"),
            TextFont { font_size: 64.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new("Press ESC to resume"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

pub fn cleanup_pause_ui(
    mut commands: Commands,
    pause_query: Query<Entity, (With<PauseOverlay>, Without<AlreadyDespawned>)>,
) {
    for entity in pause_query.iter() {
        commands.entity(entity)
            .try_insert(AlreadyDespawned)
            .despawn();
    }
}

fn spawn_evolution_ui(commands: &mut Commands, atp_amount: u32) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(120.0),
            width: Val::Px(420.0), // Wider for descriptions
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.2, 0.15, 0.95)),
        BorderColor(Color::srgb(0.3, 0.8, 0.6)),
        EvolutionUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("🧬 EVOLUTION CHAMBER"),
            TextFont { font_size: 22.0, ..default() },
            TextColor(Color::srgb(0.3, 1.0, 0.7)),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
        ));

        parent.spawn((
            Text::new(format!("ATP Available: {}⚡", atp_amount)),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 0.4)),
            Node { margin: UiRect::bottom(Val::Px(15.0)), ..default() },
        ));

        // Evolution upgrades with detailed explanations
        let evolutions = [
            (
                "1 - Membrane Reinforcement (10 ATP)",
                10,
                "🧱 Increases projectile damage by 20%",
                "Strengthens cellular membrane for more effective attacks"
            ),
            (
                "2 - Metabolic Enhancement (15 ATP)",
                15,
                "⚡ +30% movement speed & fire rate",
                "Optimizes ATP synthesis for faster cellular processes"
            ),
            (
                "3 - Cellular Integrity (20 ATP)",
                20,
                "❤️ +25 Maximum Health Points",
                "Reinforces cell structure - increases total health capacity"
            ),
            (
                "4 - Enzyme Production (25 ATP)",
                25,
                "🧪 Immunity to environmental toxins",
                "Develops extremophile traits for hostile environments"
            ),
            (
                "5 - Bioluminescence (30 ATP)",
                30,
                "💡 Enhanced coordination abilities",
                "Enables biofilm formation for defensive structures"
            ),
            (
                "6 - Emergency Spore (20 ATP)",
                20,
                "💥 +1 Emergency reproductive blast",
                "Develops additional spore for area-effect emergency defense"
            ),
            (
                "7 - Pseudopod Network (50 ATP)",
                50,
                "🕷️ Multi-directional tendril weapon",
                "Evolves spread-shot capability with 5 organic projectiles"
            ),
            (
                "8 - Symbiotic Hunters (75 ATP)",
                75,
                "🎯 Homing cooperative organisms",
                "Self-guided missiles with blast radius and target tracking"
            ),
            (
                "9 - Bioluminescent Beam (100 ATP)",
                100,
                "🌟 Concentrated energy discharge",
                "Sustained beam weapon with charging mechanism"
            ),
        ];

        for (title, cost, effect, description) in evolutions {
            let color = if atp_amount >= cost {
                Color::srgb(0.9, 1.0, 0.9)
            } else {
                Color::srgb(0.5, 0.6, 0.5)
            };

            parent.spawn((
                Text::new(title),
                TextFont { font_size: 14.0, ..default() },
                TextColor(color),
                Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() },
            ));

            parent.spawn((
                Text::new(effect),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.8, 0.9, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(description),
                TextFont { font_size: 10.0, ..default() },
                TextColor(Color::srgb(0.6, 0.7, 0.6)),
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));
        }

        parent.spawn((
            Text::new("💡 Tip: Stand near chamber and press number keys to evolve"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.6, 0.9, 0.8)),
            Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
        ));
    });
}

// Update evolution chamber UI with biological terminology
pub fn update_evolution_ui(
    mut commands: Commands,
    chamber_query: Query<&Transform, With<EvolutionChamber>>,
    player_query: Query<(&Transform, &ATP), With<Player>>,
    existing_ui_query: Query<Entity, (With<EvolutionUI>, Without<AlreadyDespawned>)>,
) {
    if let Ok((player_transform, atp)) = player_query.single() {
        let near_chamber = chamber_query.iter().any(|chamber_transform| {
            player_transform.translation.distance(chamber_transform.translation) < 60.0
        });

        if near_chamber {
            // Show evolution UI if not already showing
            if existing_ui_query.is_empty() {
                spawn_evolution_ui(&mut commands, atp.amount);
            }
        } else {
            // Hide evolution UI if showing
            for entity in existing_ui_query.iter() {
                commands.entity(entity)
                    .try_insert(AlreadyDespawned)
                    .despawn();
            }
        }
    }
}

// Enhanced game over UI with detailed stats
pub fn enhanced_game_over_ui(
    mut commands: Commands,
    game_score: Res<GameScore>,
) {
    let high_score_data = game_score.high_score_data.as_ref().unwrap();
    let is_new_high_score = game_score.current > game_score.high_scores.first().cloned().unwrap_or(0);
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        GameOverUI,
    )).with_children(|parent| {
        // Title with special effect for new high score
        if is_new_high_score {
            parent.spawn((
                Text::new("🏆 NEW HIGH SCORE! 🏆"),
                TextFont { font_size: 52.0, ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
            ));
        }
        
        parent.spawn((
            Text::new("CELLULAR BREAKDOWN"),
            TextFont { font_size: 42.0, ..default() },
            TextColor(if is_new_high_score { Color::srgb(1.0, 0.8, 0.2) } else { Color::srgb(1.0, 0.3, 0.3) }),
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        
        // Current game stats
        parent.spawn((
            Text::new(format!("Final Score: {}", game_score.current)),
            TextFont { font_size: 28.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(15.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new(format!("ATP Collected: {}", game_score.total_atp_collected)),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 0.4)),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
        ));
        
        parent.spawn((
            Text::new(format!("Organisms Defeated: {}", game_score.enemies_defeated)),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.8, 1.0, 0.8)),
            Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
        ));
        
        // High score table
        parent.spawn((
            Text::new("🧬 EVOLUTION HALL OF FAME 🧬"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.6, 1.0, 0.8)),
            Node { margin: UiRect::bottom(Val::Px(15.0)), ..default() },
        ));
        
        for (i, entry) in high_score_data.scores.iter().take(5).enumerate() {
            let rank_color = match i {
                0 => Color::srgb(1.0, 0.8, 0.2), // Gold
                1 => Color::srgb(0.8, 0.8, 0.8), // Silver  
                2 => Color::srgb(0.8, 0.5, 0.2), // Bronze
                _ => Color::srgb(0.7, 0.7, 0.7), // Regular
            };
            
            parent.spawn((
                Text::new(format!(
                    "{}. {} - {} ({})",
                    i + 1,
                    entry.score,
                    entry.evolution_type,
                    entry.date
                )),
                TextFont { font_size: 16.0, ..default() },
                TextColor(rank_color),
                Node { margin: UiRect::bottom(Val::Px(5.0)), ..default() },
            ));
        }
        
        // Overall stats
        parent.spawn((
            Text::new(format!(
                "📈 Total Games: {} | Longest Survival: {:.0}s | Best Evolution: {}",
                high_score_data.total_games_played,
                high_score_data.longest_survival_time,
                high_score_data.best_evolution_reached
            )),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.6, 0.8, 0.6)),
            Node { margin: UiRect::all(Val::Px(20.0)), ..default() },
        ));
        
        // Restart button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.7, 0.4)),
            RestartButton,
        )).with_children(|button| {
            button.spawn((
                Text::new("EVOLVE AGAIN"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
        
        parent.spawn((
            Text::new("Press R to restart or click button above"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}