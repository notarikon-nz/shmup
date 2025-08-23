use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use std::fs;
use std::path::Path;

pub fn load_high_scores_from_file(mut game_score: ResMut<GameScore>) {
    let save_path = get_save_path();
    
    match fs::read_to_string(&save_path) {
        Ok(content) => {
            match serde_json::from_str::<HighScoreData>(&content) {
                Ok(data) => {
                    game_score.high_scores = data.scores.iter().map(|entry| entry.score).collect();
                    game_score.high_score_data = Some(data);
                    println!("Loaded {} high scores from {}", game_score.high_scores.len(), save_path);
                }
                Err(e) => {
                    println!("Failed to parse high scores: {}", e);
                    game_score.high_score_data = Some(HighScoreData::default());
                    game_score.high_scores = game_score.high_score_data.as_ref().unwrap().scores.iter().map(|e| e.score).collect();
                }
            }
        }
        Err(_) => {
            println!("No save file found, creating default high scores");
            game_score.high_score_data = Some(HighScoreData::default());
            game_score.high_scores = game_score.high_score_data.as_ref().unwrap().scores.iter().map(|e| e.score).collect();
        }
    }
}

pub fn save_high_score_to_file(
    mut game_score: ResMut<GameScore>,
    player_query: Query<&EvolutionSystem, With<Player>>,
    time: Res<Time>,
) {
    if game_score.current == 0 {
        return; // Don't save zero scores
    }
    
    let mut high_score_data = game_score.high_score_data.take().unwrap_or_default();
    
    // Get current game stats
    let current_evolution = player_query.single()
        .map(|evo| evo.primary_evolution.get_display_name().to_string())
        .unwrap_or_else(|_| "Cytoplasmic Spray".to_string());
    
    let current_time = time.elapsed_secs();
    let waves_survived = (current_time / 30.0) as u32; // Estimate waves based on time
    
    // Create new high score entry
    let new_entry = HighScoreEntry {
        score: game_score.current,
        date: get_current_date(),
        evolution_type: current_evolution.clone(),
        waves_survived,
        time_played: current_time,
    };
    
    // Add to scores and sort
    high_score_data.scores.push(new_entry);
    high_score_data.scores.sort_by(|a, b| b.score.cmp(&a.score));
    high_score_data.scores.truncate(10); // Keep top 10
    
    // Update statistics
    high_score_data.total_games_played += 1;
    high_score_data.total_atp_collected += game_score.total_atp_collected;
    high_score_data.enemies_defeated += game_score.enemies_defeated;
    
    if current_time > high_score_data.longest_survival_time {
        high_score_data.longest_survival_time = current_time;
    }
    
    // Update best evolution if this is a high score with advanced evolution
    if game_score.current > high_score_data.scores.first().map(|e| e.score).unwrap_or(0) {
        high_score_data.best_evolution_reached = current_evolution;
    }
    
    // Update game score resource
    game_score.high_scores = high_score_data.scores.iter().map(|entry| entry.score).collect();
    game_score.high_score_data = Some(high_score_data.clone());
    
    // Save to file
    let save_path = get_save_path();
    match serde_json::to_string_pretty(&high_score_data) {
        Ok(json) => {
            if let Some(parent) = Path::new(&save_path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            
            match fs::write(&save_path, json) {
                Ok(_) => println!("High scores saved to {}", save_path),
                Err(e) => println!("Failed to save high scores: {}", e),
            }
        }
        Err(e) => println!("Failed to serialize high scores: {}", e),
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

// Helper functions
fn get_save_path() -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{}\\Cosmic_Tidal_Pool\\high_scores.json", 
                std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string()))
    }
    #[cfg(target_os = "macos")]
    {
        format!("{}/.config/cosmic_tidal_pool/high_scores.json",
                std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        format!("{}/.local/share/cosmic_tidal_pool/high_scores.json",
                std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    }
}

fn get_current_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Simple date formatting (days since epoch)
    let days = timestamp / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    
    format!("{:04}-{:02}-{:02}", year, month, day)
}

