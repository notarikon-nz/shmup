// Achievement System for Steam Integration
// Add to resources.rs and create achievements.rs

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::components::*;
use crate::resources::*;

// Achievement System Components and Resources
#[derive(Resource, Default)]
pub struct AchievementManager {
    pub achievements: HashMap<String, Achievement>,
    pub unlocked_achievements: Vec<String>,
    pub progress_tracking: HashMap<String, f32>,
    pub session_stats: SessionStats,
    pub lifetime_stats: LifetimeStats,
    pub steam_integration: Option<SteamAchievements>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlock_condition: UnlockCondition,
    pub category: AchievementCategory,
    pub rarity: AchievementRarity,
    pub steam_id: Option<String>, // For Steam integration
    pub unlock_date: Option<String>,
    pub progress_current: f32,
    pub progress_required: f32,
    pub icon_path: String,
    pub reward: Option<AchievementReward>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum UnlockCondition {
    // Combat Achievements
    ScoreReached(u32),
    EnemiesKilled(u32),
    SurviveTime(f32), // seconds
    ReachWave(u32),
    CriticalHits(u32),
    
    // Evolution Achievements
    ReachEvolutionLevel(String), // "BioluminescentBeam", etc.
    UseAllEvolutions,
    UpgradeMaxHealth(i32),
    CollectATP(u32),
    
    // Environmental Achievements
    SurviveKingTide(u32), // number of king tides
    CleanCorals(u32), // restore corrupted corals
    EcosystemHealth(f32), // maintain ecosystem health above threshold
    ExploreDebris(u32), // discover story fragments
    
    // Challenge Achievements
    WinWithoutUpgrades,
    WinWithBasicWeapon, // never evolve past Cytoplasmic Spray
    SurviveWithoutHealing(f32), // time without health pickups
    PerfectAccuracy(f32), // % of shots hit
    
    // Discovery Achievements
    EncounterAllEnemyTypes,
    UseAllPowerUps,
    VisitAllBiomes, // different environmental zones
    CollectAllStoryFragments,
    
    // Mastery Achievements
    ConsecutiveGamesPlayed(u32),
    TotalPlayTime(f32),
    HighScoreStreak(u32), // consecutive games improving high score
    
    // Compound conditions
    And(Vec<UnlockCondition>),
    Or(Vec<UnlockCondition>),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AchievementCategory {
    Combat,
    Evolution,
    Environmental,
    Discovery,
    Challenge,
    Mastery,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,    // 70%+ of players unlock
    Uncommon,  // 40-70% unlock  
    Rare,      // 10-40% unlock
    Epic,      // 3-10% unlock
    Legendary, // <3% unlock
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AchievementReward {
    ATP(u32),
    UnlockEvolution(String),
    UnlockCosmetic(String),
    UnlockTitle(String),
    StatBoost { stat: String, multiplier: f32 },
}

#[derive(Default, Serialize, Deserialize)]
pub struct SessionStats {
    pub enemies_killed: u32,
    pub score_achieved: u32,
    pub time_survived: f32,
    pub waves_reached: u32,
    pub atp_collected: u32,
    pub critical_hits: u32,
    pub shots_fired: u32,
    pub shots_hit: u32,
    pub evolutions_used: Vec<String>,
    pub powerups_used: Vec<String>,
    pub story_fragments_found: u32,
    pub king_tides_survived: u32,
    pub health_lost: i32,
    pub health_gained: i32,
    pub ecosystems_visited: Vec<String>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LifetimeStats {
    pub total_games_played: u32,
    pub total_playtime: f32,
    pub total_enemies_killed: u32,
    pub total_atp_collected: u32,
    pub highest_score: u32,
    pub longest_survival: f32,
    pub evolutions_mastered: Vec<String>,
    pub achievements_unlocked: u32,
    pub consecutive_games: u32,
    pub high_score_streak: u32,
    pub perfect_games: u32, // no damage taken
    pub story_completion: f32, // % of lore discovered
}

// Steam Integration (optional)
pub struct SteamAchievements {
    // pub client: Option<steamworks::Client>, // When steamworks is available
    pub pending_unlocks: Vec<String>,
}

// Achievement System Implementation
pub fn initialize_achievements() -> AchievementManager {
    let mut achievements = HashMap::new();
    
    // Combat Achievements
    achievements.insert("first_kill".to_string(), Achievement {
        id: "first_kill".to_string(),
        name: "First Contact".to_string(),
        description: "Eliminate your first hostile microorganism".to_string(),
        unlock_condition: UnlockCondition::EnemiesKilled(1),
        category: AchievementCategory::Combat,
        rarity: AchievementRarity::Common,
        steam_id: Some("FIRST_KILL".to_string()),
        unlock_date: None,
        progress_current: 0.0,
        progress_required: 1.0,
        icon_path: "achievements/first_kill.png".to_string(),
        reward: Some(AchievementReward::ATP(50)),
    });
    
    achievements.insert("survivor".to_string(), Achievement {
        id: "survivor".to_string(),
        name: "Cellular Survivor".to_string(),
        description: "Survive for 5 minutes in the tidal pool".to_string(),
        unlock_condition: UnlockCondition::SurviveTime(300.0),
        category: AchievementCategory::Combat,
        rarity: AchievementRarity::Common,
        steam_id: Some("SURVIVOR_5MIN".to_string()),
        unlock_date: None,
        progress_current: 0.0,
        progress_required: 300.0,
        icon_path: "achievements/survivor.png".to_string(),
        reward: Some(AchievementReward::ATP(200)),
    });
    
    achievements.insert("evolution_master".to_string(), Achievement {
        id: "evolution_master".to_string(),
        name: "Evolutionary Pinnacle".to_string(),
        description: "Achieve the ultimate evolution: Electric Discharge".to_string(),
        unlock_condition: UnlockCondition::ReachEvolutionLevel("ElectricDischarge".to_string()),
        category: AchievementCategory::Evolution,
        rarity: AchievementRarity::Rare,
        steam_id: Some("EVOLUTION_MASTER".to_string()),
        unlock_date: None,
        progress_current: 0.0,
        progress_required: 1.0,
        icon_path: "achievements/evolution_master.png".to_string(),
        reward: Some(AchievementReward::StatBoost { 
            stat: "evolution_power".to_string(), 
            multiplier: 1.2 
        }),
    });
    
    achievements.insert("ecosystem_guardian".to_string(), Achievement {
        id: "ecosystem_guardian".to_string(),
        name: "Ecosystem Guardian".to_string(),
        description: "Maintain ecosystem health above 80% for 10 minutes".to_string(),
        unlock_condition: UnlockCondition::EcosystemHealth(0.8),
        category: AchievementCategory::Environmental,
        rarity: AchievementRarity::Uncommon,
        steam_id: Some("ECO_GUARDIAN".to_string()),
        unlock_date: None,
        progress_current: 0.0,
        progress_required: 600.0, // 10 minutes
        icon_path: "achievements/eco_guardian.png".to_string(),
        reward: Some(AchievementReward::UnlockTitle("Guardian of the Deep".to_string())),
    });
    
    achievements.insert("king_tide_master".to_string(), Achievement {
        id: "king_tide_master".to_string(),
        name: "Tide Walker".to_string(),
        description: "Survive 5 King Tide events in a single run".to_string(),
        unlock_condition: UnlockCondition::SurviveKingTide(5),
        category: AchievementCategory::Challenge,
        rarity: AchievementRarity::Epic,
        steam_id: Some("KING_TIDE_MASTER".to_string()),
        unlock_date: None,
        progress_current: 0.0,
        progress_required: 5.0,
        icon_path: "achievements/tide_walker.png".to_string(),
        reward: Some(AchievementReward::UnlockCosmetic("Tidal Aura".to_string())),
    });
    
    achievements.insert("perfect_predator".to_string(), Achievement {
        id: "perfect_predator".to_string(),
        name: "Perfect Predator".to_string(),
        description: "Achieve 95% accuracy in a single game".to_string(),
        unlock_condition: UnlockCondition::PerfectAccuracy(0.95),
        category: AchievementCategory::Challenge,
        rarity: AchievementRarity::Legendary,
        steam_id: Some("PERFECT_PREDATOR".to_string()),
        unlock_date: None,
        progress_current: 0.0,
        progress_required: 0.95,
        icon_path: "achievements/perfect_predator.png".to_string(),
        reward: Some(AchievementReward::UnlockEvolution("PrecisionTargeting".to_string())),
    });
    
    achievements.insert("lore_master".to_string(), Achievement {
        id: "lore_master".to_string(),
        name: "Deep Sea Scholar".to_string(),
        description: "Discover 50 environmental story fragments".to_string(),
        unlock_condition: UnlockCondition::ExploreDebris(50),
        category: AchievementCategory::Discovery,
        rarity: AchievementRarity::Rare,
        steam_id: Some("LORE_MASTER".to_string()),
        unlock_date: None,
        progress_current: 0.0,
        progress_required: 50.0,
        icon_path: "achievements/lore_master.png".to_string(),
        reward: Some(AchievementReward::UnlockTitle("Deep Sea Scholar".to_string())),
    });
    
    AchievementManager {
        achievements,
        unlocked_achievements: Vec::new(),
        progress_tracking: HashMap::new(),
        session_stats: SessionStats::default(),
        lifetime_stats: LifetimeStats::default(),
        steam_integration: None,
    }
}

// Achievement tracking systems
pub fn track_achievements_system(
    mut achievement_manager: ResMut<AchievementManager>,
    mut achievement_events: EventReader<AchievementEvent>,
    player_query: Query<(&Player, &Health, &EvolutionSystem, &ATP)>,
    ecosystem: Res<EcosystemState>,
    game_score: Res<GameScore>,
    time: Res<Time>,
    mut commands: Commands,
) {
    // Update session stats
    if let Ok((player, health, evolution, atp)) = player_query.single() {
        achievement_manager.session_stats.time_survived = time.elapsed_secs();
        achievement_manager.session_stats.score_achieved = game_score.current;
    }
    
    // Process achievement events
    for event in achievement_events.read() {
        match event {
            AchievementEvent::EnemyKilled(enemy_type) => {
                achievement_manager.session_stats.enemies_killed += 1;
                achievement_manager.lifetime_stats.total_enemies_killed += 1;
            }
            AchievementEvent::EvolutionReached(evolution_name) => {
                if !achievement_manager.session_stats.evolutions_used.contains(evolution_name) {
                    achievement_manager.session_stats.evolutions_used.push(evolution_name.clone());
                }
            }
            AchievementEvent::ATPCollected(amount) => {
                achievement_manager.session_stats.atp_collected += amount;
                achievement_manager.lifetime_stats.total_atp_collected += amount;
            }
            AchievementEvent::StoryFragmentFound => {
                achievement_manager.session_stats.story_fragments_found += 1;
            }
            AchievementEvent::KingTideSurvived => {
                achievement_manager.session_stats.king_tides_survived += 1;
            }
            AchievementEvent::ShotFired => {
                achievement_manager.session_stats.shots_fired += 1;
            }
            AchievementEvent::ShotHit => {
                achievement_manager.session_stats.shots_hit += 1;
            }
            AchievementEvent::CriticalHit => {
                achievement_manager.session_stats.critical_hits += 1;
            }
            _ => {
                // PLACEHOLDER
            }
        }
    }
    
    // Check for achievement unlocks
    check_achievement_progress(&mut achievement_manager, &ecosystem, &mut commands);
}

#[derive(Event)]
pub enum AchievementEvent {
    EnemyKilled(String),
    EvolutionReached(String),
    ATPCollected(u32),
    StoryFragmentFound,
    KingTideSurvived,
    ShotFired,
    ShotHit,
    CriticalHit,
    PowerUpUsed(String),
    HealthLost(i32),
    HealthGained(i32),

    WaveCompleted { wave_number: u32, completion_time: f32 },
    FirstEvolution,
    SurvivalMilestone { minutes: u32 },
    BossDefeated,
    PerfectWave, // No damage taken
    SpeedRun { wave_number: u32, time: f32 }, // Completed wave under time limit

}

fn check_achievement_progress(
    mut achievement_manager: &mut AchievementManager,
    ecosystem: &EcosystemState,
    commands: &mut Commands,
) {
    let mut newly_unlocked = Vec::new();
    
    for (id, achievement) in achievement_manager.achievements.iter_mut() {
        if achievement_manager.unlocked_achievements.contains(id) {
            continue; // Already unlocked
        }
        
        let progress = calculate_progress(achievement, &achievement_manager.session_stats, ecosystem);
        achievement.progress_current = progress;
        
        if progress >= achievement.progress_required {
            newly_unlocked.push((id.clone(), achievement.clone()));
        }
    }
    
    // Unlock achievements and spawn notifications
    for (id, achievement) in newly_unlocked {
        unlock_achievement(&mut achievement_manager, id, achievement, commands);
    }
}

fn calculate_progress(
    achievement: &Achievement, 
    session_stats: &SessionStats,
    ecosystem: &EcosystemState
) -> f32 {
    match &achievement.unlock_condition {
        UnlockCondition::ScoreReached(target) => session_stats.score_achieved as f32,
        UnlockCondition::EnemiesKilled(target) => session_stats.enemies_killed as f32,
        UnlockCondition::SurviveTime(target) => session_stats.time_survived,
        UnlockCondition::CollectATP(target) => session_stats.atp_collected as f32,
        UnlockCondition::CriticalHits(target) => session_stats.critical_hits as f32,
        UnlockCondition::SurviveKingTide(target) => session_stats.king_tides_survived as f32,
        UnlockCondition::ExploreDebris(target) => session_stats.story_fragments_found as f32,
        UnlockCondition::PerfectAccuracy(target) => {
            if session_stats.shots_fired > 0 {
                session_stats.shots_hit as f32 / session_stats.shots_fired as f32
            } else {
                0.0
            }
        },
        UnlockCondition::EcosystemHealth(target) => {
            if ecosystem.health >= *target { session_stats.time_survived } else { 0.0 }
        },
        UnlockCondition::ReachEvolutionLevel(evolution) => {
            if session_stats.evolutions_used.contains(evolution) { 1.0 } else { 0.0 }
        },
        _ => 0.0, // Handle other conditions as needed
    }
}

fn unlock_achievement(
    achievement_manager: &mut AchievementManager,
    id: String,
    achievement: Achievement,
    commands: &mut Commands,
) {
    // Mark as unlocked
    achievement_manager.unlocked_achievements.push(id.clone());
    achievement_manager.lifetime_stats.achievements_unlocked += 1;
    
    // Apply rewards
    if let Some(reward) = &achievement.reward {
        apply_achievement_reward(reward, commands);
    }
    
    // Steam integration
    if let Some(steam) = &mut achievement_manager.steam_integration {
        if let Some(steam_id) = &achievement.steam_id {
            steam.pending_unlocks.push(steam_id.clone());
        }
    }
    
    // Spawn achievement notification
    spawn_achievement_notification(commands, &achievement);
    
    println!("🏆 Achievement Unlocked: {}", achievement.name);
}

fn apply_achievement_reward(reward: &AchievementReward, commands: &mut Commands) {
    match reward {
        AchievementReward::ATP(amount) => {
            // Add ATP to player - would need player query here
            println!("Rewarded {} ATP", amount);
        }
        AchievementReward::UnlockEvolution(evolution) => {
            println!("Unlocked evolution: {}", evolution);
        }
        AchievementReward::UnlockCosmetic(cosmetic) => {
            println!("Unlocked cosmetic: {}", cosmetic);
        }
        AchievementReward::UnlockTitle(title) => {
            println!("Unlocked title: {}", title);
        }
        AchievementReward::StatBoost { stat, multiplier } => {
            println!("Stat boost: {} x{}", stat, multiplier);
        }
    }
}

fn spawn_achievement_notification(commands: &mut Commands, achievement: &Achievement) {
    // Spawn UI notification that slides in from the side
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(100.0),
            width: Val::Px(300.0),
            height: Val::Px(80.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.3, 0.5, 0.9)),
        BorderColor(Color::srgb(0.8, 0.6, 0.2)),
        AchievementNotification {
            timer: 5.0, // Display for 5 seconds
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("🏆 ACHIEVEMENT UNLOCKED"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(1.0, 0.8, 0.2)),
        ));
        
        parent.spawn((
            Text::new(&achievement.name),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        parent.spawn((
            Text::new(&achievement.description),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

#[derive(Component)]
pub struct AchievementNotification {
    pub timer: f32,
}

pub fn update_achievement_notifications(
    mut commands: Commands,
    mut notification_query: Query<(Entity, &mut AchievementNotification)>,
    time: Res<Time>,
) {
    for (entity, mut notification) in notification_query.iter_mut() {
        notification.timer -= time.delta_secs();
        
        if notification.timer <= 0.0 {
            commands.entity(entity)
                .insert(AlreadyDespawned)
                .despawn();
        }
    }
}

// Save/Load achievements
pub fn save_achievements(achievement_manager: &AchievementManager) {
    let save_data = AchievementSaveData {
        unlocked_achievements: achievement_manager.unlocked_achievements.clone(),
        lifetime_stats: achievement_manager.lifetime_stats.clone(),
    };
    
    if let Ok(json) = serde_json::to_string_pretty(&save_data) {
        // Save to file system (implementation depends on platform)
        std::fs::write("achievements.json", json).ok();
    }
}

pub fn load_achievements(achievement_manager: &mut AchievementManager) {
    if let Ok(json) = std::fs::read_to_string("achievements.json") {
        if let Ok(save_data) = serde_json::from_str::<AchievementSaveData>(&json) {
            achievement_manager.unlocked_achievements = save_data.unlocked_achievements;
            achievement_manager.lifetime_stats = save_data.lifetime_stats;
        }
    }
}

pub fn load_achievements_from_file() -> PersistentAchievements {
    if let Ok(content) = std::fs::read_to_string("achievements_persistent.json") {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        PersistentAchievements::default()
    }
}


#[derive(Serialize, Deserialize)]
pub struct AchievementSaveData {
    pub unlocked_achievements: Vec<String>,
    pub lifetime_stats: LifetimeStats,
}

pub fn save_achievements_on_exit(achievement_manager: Res<AchievementManager>) {
    save_achievements(&achievement_manager);
}

pub fn load_persistent_achievements(mut achievement_manager: ResMut<AchievementManager>) {
    let persistent = load_achievements_from_file();
    achievement_manager.unlocked_achievements = persistent.unlocked;
    achievement_manager.lifetime_stats.total_games_played = persistent.total_games;
    achievement_manager.lifetime_stats.total_playtime = persistent.total_playtime;
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PersistentAchievements {
    pub unlocked: Vec<String>,
    pub progress: std::collections::HashMap<String, f32>,
    pub last_updated: String,
    pub total_games: u32,
    pub total_playtime: f32,
}