// daily_challenges.rs - Standalone, transferable challenge system
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChallengeSystem {
    pub challenges: HashMap<String, Challenge>,
    pub player_progress: HashMap<String, ChallengeProgress>,
    pub daily_seed: u64,
    pub last_refresh: String, // "2025-01-15" format
    pub completion_streaks: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Challenge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub challenge_type: ChallengeType,
    pub target_value: f32,
    pub reward: ChallengeReward,
    pub difficulty: ChallengeDifficulty,
    pub tags: Vec<String>, // For filtering/categorization
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChallengeType {
    // Combat challenges
    KillEnemies { enemy_type: Option<String>, count: u32 },
    DealDamage { amount: u32 },
    SurviveTime { seconds: u32 },
    AchieveScore { target: u32 },
    CriticalHits { count: u32 },
    
    // Evolution challenges  
    UseWeapon { weapon_name: String, kills: u32 },
    CollectATP { amount: u32 },
    CompleteWaves { count: u32 },
    UpgradeTimes { count: u32 },
    
    // Efficiency challenges
    WinWithoutUpgrades,
    PerfectAccuracy { min_percentage: f32 },
    NoHealthLoss { waves: u32 },
    SpeedRun { waves: u32, time_limit: u32 },
    
    // Custom game-specific (extensible)
    Custom { key: String, target: f32 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChallengeReward {
    pub atp: Option<u32>,
    pub unlock_cosmetic: Option<String>,
    pub unlock_title: Option<String>,
    pub multiplier_bonus: Option<f32>, // Score/ATP multiplier for next game
    pub custom_reward: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChallengeDifficulty {
    Easy,    // 90% completion rate
    Medium,  // 60% completion rate  
    Hard,    // 30% completion rate
    Expert,  // 10% completion rate
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChallengeProgress {
    pub current_value: f32,
    pub completed: bool,
    pub completion_date: Option<String>,
    pub attempts: u32,
}

impl ChallengeSystem {
    pub fn new() -> Self {
        Self {
            challenges: HashMap::new(),
            player_progress: HashMap::new(),
            daily_seed: Self::generate_daily_seed(),
            last_refresh: Self::today_string(),
            completion_streaks: HashMap::new(),
        }
    }
    
    // Auto-refresh daily challenges
    pub fn update(&mut self) -> bool {
        let today = Self::today_string();
        if today != self.last_refresh {
            self.refresh_daily_challenges();
            self.last_refresh = today;
            self.daily_seed = Self::generate_daily_seed();
            true
        } else {
            false
        }
    }
    
    // Generate 3 daily challenges with balanced difficulty
    pub fn refresh_daily_challenges(&mut self) {
        self.challenges.clear();
        self.player_progress.clear();
        
        let mut rng = self.create_seeded_rng();
        let templates = self.get_challenge_templates();
        
        // Generate 1 easy, 1 medium, 1 hard challenge
        let difficulties = [
            ChallengeDifficulty::Easy,
            ChallengeDifficulty::Medium, 
            ChallengeDifficulty::Hard
        ];
        
        for (i, difficulty) in difficulties.iter().enumerate() {
            let template = &templates[rng.gen_range(0..templates.len())];
            let challenge = self.generate_challenge_from_template(template, difficulty.clone(), &mut rng);
            
            self.challenges.insert(challenge.id.clone(), challenge.clone());
            self.player_progress.insert(challenge.id, ChallengeProgress {
                current_value: 0.0,
                completed: false,
                completion_date: None,
                attempts: 0,
            });
        }
    }
    
    // Track progress (call from game events)
    pub fn track_progress(&mut self, event_type: &str, value: f32, context: Option<&str>) {
        for (challenge_id, challenge) in &self.challenges {
            if let Some(progress) = self.player_progress.get_mut(challenge_id) {
                if progress.completed { continue; }
                
                let matches = match &challenge.challenge_type {
                    ChallengeType::KillEnemies { enemy_type, .. } => {
                        event_type == "enemy_killed" && 
                        (enemy_type.is_none() || enemy_type.as_ref() == context)
                    },
                    ChallengeType::CollectATP { .. } => event_type == "atp_collected",
                    ChallengeType::DealDamage { .. } => event_type == "damage_dealt",
                    ChallengeType::SurviveTime { .. } => event_type == "time_survived",
                    ChallengeType::Custom { key, .. } => event_type == key,
                    _ => false,
                };
                
                if matches {
                    progress.current_value += value;
                    
                    // Check completion
                    if progress.current_value >= challenge.target_value {
                        progress.completed = true;
                        progress.completion_date = Some(Self::today_string());
                        
                        // Update streak
                        let streak = self.completion_streaks.entry(challenge_id.clone()).or_insert(0);
                        *streak += 1;
                    }
                }
            }
        }
    }
    
    // Get pending rewards
    pub fn get_pending_rewards(&self) -> Vec<(String, ChallengeReward)> {
        self.challenges.iter()
            .filter(|(id, _)| {
                self.player_progress.get(*id)
                    .map(|p| p.completed)
                    .unwrap_or(false)
            })
            .map(|(id, challenge)| (id.clone(), challenge.reward.clone()))
            .collect()
    }
    
    // Utility functions
    fn generate_daily_seed() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let days_since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() / 86400; // seconds per day
        days_since_epoch
    }
    
    fn today_string() -> String {
        // In real implementation, use chrono or similar
        "2025-01-15".to_string() // Placeholder
    }
    
    // External data loading
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }
    
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

/*

// Add to main.rs resources
.init_resource::<ChallengeSystem>()

// Add single update system
.add_systems(Update, daily_challenge_system)

// Integration system
pub fn daily_challenge_system(
    mut challenges: ResMut<ChallengeSystem>,
    mut achievement_events: EventReader<AchievementEvent>,
    mut commands: Commands,
    mut player_query: Query<&mut ATP, With<Player>>,
    time: Res<Time>,
) {
    // Auto-refresh daily challenges
    if challenges.update() {
        println!("New daily challenges available!");
        // Could spawn notification UI here
    }
    
    // Track progress from existing events
    for event in achievement_events.read() {
        match event {
            AchievementEvent::EnemyKilled(enemy_type) => {
                challenges.track_progress("enemy_killed", 1.0, Some(enemy_type));
            },
            AchievementEvent::ATPCollected(amount) => {
                challenges.track_progress("atp_collected", *amount as f32, None);
            },
            AchievementEvent::ShotHit => {
                challenges.track_progress("damage_dealt", 10.0, None); // Estimate
            },
            _ => {}
        }
    }
    
    // Apply pending rewards
    let rewards = challenges.get_pending_rewards();
    if !rewards.is_empty() {
        if let Ok(mut atp) = player_query.single_mut() {
            for (challenge_id, reward) in rewards {
                if let Some(atp_reward) = reward.atp {
                    atp.amount += atp_reward;
                    println!("Daily challenge '{}' completed! +{} ATP", challenge_id, atp_reward);
                }
                // Handle other reward types...
            }
        }
    }
}


// Add to existing UI systems
pub fn daily_challenge_ui(
    challenges: Res<ChallengeSystem>,
    mut commands: Commands,
    fonts: Res<GameFonts>,
    input: Res<ButtonInput<KeyCode>>,
    existing_ui: Query<Entity, With<DailyChallengeUI>>,
) {
    if input.just_pressed(KeyCode::Tab) {
        if existing_ui.is_empty() {
            spawn_daily_challenge_ui(&mut commands, &challenges, &fonts);
        } else {
            for entity in existing_ui.iter() {
                commands.entity(entity).safe_despawn();
            }
        }
    }
}

#[derive(Component)]
struct DailyChallengeUI;

*/