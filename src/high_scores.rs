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

