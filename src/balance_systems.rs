// src/balance_systems.rs - Evolution System Balance Analysis & Tuning
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::components::*;
use crate::resources::*;
use crate::enemy_types::*;
use crate::achievements::*;
use crate::wave_systems::{WaveManager};

// ===== BALANCE CONSTANTS =====
pub const ATP_GENERATION_RATES: [(EnemyType, u32, f32); 9] = [
    (EnemyType::ViralParticle, 1, 0.7),
    (EnemyType::AggressiveBacteria, 2, 0.8),
    (EnemyType::ParasiticProtozoa, 5, 0.9),
    (EnemyType::InfectedMacrophage, 25, 1.0),
    (EnemyType::SuicidalSpore, 3, 0.6),
    (EnemyType::BiofilmColony, 8, 0.8),
    (EnemyType::SwarmCell, 4, 0.7),
    (EnemyType::ReproductiveVesicle, 15, 0.9),
    (EnemyType::Offspring, 1, 0.5),
];

const EVOLUTION_COSTS: [(EvolutionType, u32); 7] = [
    (EvolutionType::CytoplasmicSpray { damage: 10, fire_rate: 0.1 }, 0),
    (EvolutionType::PseudopodNetwork { damage: 8, fire_rate: 0.15, tendril_count: 5, spread_angle: 0.6 }, 50),
    (EvolutionType::BioluminescentBeam { damage: 15, charge_time: 1.0, duration: 2.0, width: 20.0 }, 100),
    (EvolutionType::SymbioticHunters { damage: 25, fire_rate: 0.8, homing_strength: 2.0, blast_radius: 50.0 }, 75),
    (EvolutionType::EnzymeBurst { damage: 6, fire_rate: 0.05, acid_damage: 3.0 }, 60),
    (EvolutionType::ToxinCloud { damage_per_second: 12, cloud_radius: 80.0, duration: 8.0 }, 90),
    (EvolutionType::ElectricDischarge { damage: 30, chain_count: 4, range: 150.0 }, 150),
];

const UPGRADE_COSTS: [(&str, u32, f32); 6] = [
    ("damage", 10, 1.2),
    ("metabolic", 15, 1.3),
    ("cellular", 20, 1.0),
    ("enzyme", 25, 1.0),
    ("bioluminescence", 30, 1.0),
    ("spore", 20, 1.0),
];

const INVINCIBILITY_FRAMES: f32 = 1.0; // Base invincibility duration
const HEALTH_UPGRADE_BASE: i32 = 25;
const MOVEMENT_SPEED_BASE: f32 = 400.0;

// ===== BALANCE ANALYSIS RESOURCES =====

#[derive(Clone, Default, Resource)]
pub struct BalanceAnalyzer {
    pub weapon_stats: HashMap<String, WeaponPerformance>,
    pub atp_economy: ATPEconomyData,
    pub progression_metrics: ProgressionMetrics,
    pub real_time_balance: RealTimeBalance,
    pub debug_mode: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WeaponPerformance {
    pub evolution_name: String,
    pub theoretical_dps: f32,
    pub actual_dps: f32,
    pub atp_cost: u32,
    pub cost_efficiency: f32, // DPS per ATP
    pub late_game_viability: f32, // 0.0-1.0 scale
    pub usage_frequency: u32,
    pub kill_count: u32,
    pub accuracy_rate: f32,
    pub upgrade_impact: f32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ATPEconomyData {
    pub generation_rate_per_second: f32,
    pub spending_rate_per_second: f32,
    pub balance_deficit: f32,
    pub enemy_atp_values: HashMap<String, (u32, f32)>, // (amount, drop_rate)
    pub upgrade_costs: HashMap<String, u32>,
    pub evolution_unlock_times: HashMap<String, f32>, // Average time to unlock
    pub economy_health: f32, // 0.0-1.0, balanced at 0.5
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ProgressionMetrics {
    pub health_scaling: Vec<(i32, f32)>, // (upgrade_level, effectiveness)
    pub movement_balance: MovementBalance,
    pub invincibility_effectiveness: f32,
    pub difficulty_curve: Vec<(u32, f32)>, // (wave, difficulty_multiplier)
    pub player_power_curve: Vec<(f32, f32)>, // (time, power_level)
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MovementBalance {
    pub base_speed: f32,
    pub speed_vs_difficulty: f32, // How speed affects game balance
    pub optimal_speed_range: (f32, f32),
    pub current_effectiveness: f32,
}

#[derive(Clone, Default, Resource)]
pub struct RealTimeBalance {
    pub current_session: BalanceSession,
    pub historical_data: Vec<BalanceSession>,
    pub active_adjustments: Vec<BalanceAdjustment>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BalanceSession {
    pub start_time: f32,
    pub end_time: f32,
    pub waves_reached: u32,
    pub evolutions_used: Vec<String>,
    pub atp_collected: u32,
    pub atp_spent: u32,
    pub upgrades_purchased: Vec<String>,
    pub deaths: u32,
    pub final_score: u32,
    pub balance_issues: Vec<BalanceIssue>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BalanceIssue {
    pub issue_type: BalanceIssueType,
    pub severity: f32, // 0.0-1.0
    pub description: String,
    pub suggested_fix: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum BalanceIssueType {
    #[default]
    Unknown,
    WeaponUnderPowered,
    WeaponOverPowered,
    ATPStarvation,
    ATPAbundance,
    ProgressionTooSlow,
    ProgressionTooFast,
    DifficultySpike,
    InvincibilityTooShort,
    InvincibilityTooLong,
}

#[derive(Clone)]
pub struct BalanceAdjustment {
    pub adjustment_type: AdjustmentType,
    pub target: String,
    pub multiplier: f32,
    pub duration: f32,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub enum AdjustmentType {
    DamageMultiplier,
    FireRateMultiplier,
    ATPGeneration,
    UpgradeCost,
    HealthScaling,
    MovementSpeed,
    InvincibilityDuration,
}

// ===== BALANCE ANALYSIS SYSTEMS =====

pub fn initialize_balance_analyzer(mut commands: Commands) {
    let mut weapon_stats = HashMap::new();
    
    // Initialize weapon performance data
    for evolution in get_all_evolutions() {
        weapon_stats.insert(evolution.get_display_name().to_string(), WeaponPerformance {
            evolution_name: evolution.get_display_name().to_string(),
            theoretical_dps: calculate_theoretical_dps(&evolution),
            actual_dps: 0.0,
            atp_cost: get_evolution_cost(&evolution),
            cost_efficiency: 0.0,
            late_game_viability: 0.5,
            usage_frequency: 0,
            kill_count: 0,
            accuracy_rate: 0.0,
            upgrade_impact: 1.0,
        });
    }
    
    let atp_economy = ATPEconomyData {
        generation_rate_per_second: 2.0, // Initial estimate
        spending_rate_per_second: 1.5,
        balance_deficit: 0.0,
        enemy_atp_values: ATP_GENERATION_RATES.iter()
            .map(|(enemy, amount, rate)| (format!("{:?}", enemy), (*amount, *rate)))
            .collect(),
        upgrade_costs: UPGRADE_COSTS.iter()
            .map(|(name, cost, _)| (name.to_string(), *cost))
            .collect(),
        evolution_unlock_times: HashMap::new(),
        economy_health: 0.5,
    };
    
    let progression_metrics = ProgressionMetrics {
        health_scaling: vec![(0, 1.0), (1, 1.25), (2, 1.56), (3, 1.95)],
        movement_balance: MovementBalance {
            base_speed: MOVEMENT_SPEED_BASE,
            speed_vs_difficulty: 1.0,
            optimal_speed_range: (300.0, 500.0),
            current_effectiveness: 1.0,
        },
        invincibility_effectiveness: 1.0,
        difficulty_curve: vec![(1, 1.0), (5, 1.3), (10, 1.8), (15, 2.5), (20, 3.5)],
        player_power_curve: vec![(0.0, 1.0)],
    };
    
    commands.insert_resource(BalanceAnalyzer {
        weapon_stats,
        atp_economy,
        progression_metrics,
        real_time_balance: RealTimeBalance {
            current_session: BalanceSession {
                start_time: 0.0,
                end_time: 0.0,
                waves_reached: 0,
                evolutions_used: Vec::new(),
                atp_collected: 0,
                atp_spent: 0,
                upgrades_purchased: Vec::new(),
                deaths: 0,
                final_score: 0,
                balance_issues: Vec::new(),
            },
            historical_data: Vec::new(),
            active_adjustments: Vec::new(),
        },
        debug_mode: false,
    });
}

pub fn real_time_balance_analysis(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    player_query: Query<(&Player, &Health, &EvolutionSystem, &ATP)>,
    wave_manager: Res<WaveManager>,
    game_score: Res<GameScore>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_secs();
    
    if let Ok((player, health, evolution_system, atp)) = player_query.single() {
        let session = &mut balance_analyzer.real_time_balance.current_session;
        
        // Update session data
        session.end_time = current_time;
        session.waves_reached = wave_manager.current_wave;
        session.atp_collected = game_score.total_atp_collected as u32;
        session.final_score = game_score.current;
        
        // Track current evolution
        let current_evolution = evolution_system.primary_evolution.get_display_name().to_string();
        if !session.evolutions_used.contains(&current_evolution) {
            session.evolutions_used.push(current_evolution.clone());
            
            // Update weapon usage frequency
            if let Some(weapon_stats) = balance_analyzer.weapon_stats.get_mut(&current_evolution) {
                weapon_stats.usage_frequency += 1;
            }
        }
        
        // Analyze balance issues
        analyze_current_balance_issues(&mut balance_analyzer, player, health, evolution_system, atp, &wave_manager, current_time);
        
        // Apply dynamic adjustments if needed
        apply_dynamic_balance_adjustments(&mut balance_analyzer, &mut commands);
    }
}

pub fn weapon_performance_tracking(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    mut achievement_events: EventReader<AchievementEvent>,
    enemy_query: Query<&Enemy>,
    projectile_query: Query<&Projectile, With<Projectile>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    // Track weapon effectiveness through events
    for event in achievement_events.read() {
        match event {
            AchievementEvent::EnemyKilled(enemy_type) => {
                // Find which weapon type is currently active and credit the kill
                // This would need to be enhanced to track weapon-specific kills
                for weapon_stats in balance_analyzer.weapon_stats.values_mut() {
                    weapon_stats.kill_count += 1; // Simplified - would need weapon identification
                }
            }
            AchievementEvent::ShotFired => {
                // Track accuracy for current weapon
            }
            AchievementEvent::ShotHit => {
                // Update accuracy statistics
            }
            _ => {}
        }
    }
    
    // Calculate real-time DPS for active weapons
    let enemy_count = enemy_query.iter().count() as f32;
    let projectile_count = projectile_query.iter().count() as f32;
    
    // Update theoretical vs actual performance metrics
    for weapon_stats in balance_analyzer.weapon_stats.values_mut() {
        // Calculate cost efficiency
        weapon_stats.cost_efficiency = if weapon_stats.atp_cost > 0 {
            weapon_stats.actual_dps / weapon_stats.atp_cost as f32
        } else {
            weapon_stats.actual_dps
        };
        
        // Calculate late-game viability based on enemy scaling
        weapon_stats.late_game_viability = calculate_late_game_viability(weapon_stats, enemy_count);
    }
}

pub fn atp_economy_analysis(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    player_query: Query<(&ATP, &EvolutionSystem)>,
    mut atp_events: EventReader<AchievementEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let mut atp_gained = 0u32;
    let mut atp_spent = 0u32;
    
    // Track ATP transactions
    for event in atp_events.read() {
        match event {
            AchievementEvent::ATPCollected(amount) => {
                atp_gained += amount;
            }
            _ => {}
        }
    }
    
    let mut balance_analyzer_clone = balance_analyzer.clone();

    if let Ok((atp, evolution_system)) = player_query.single() {
        let economy = &mut balance_analyzer.atp_economy;
        
        // Update generation rate (smoothed over time)
        economy.generation_rate_per_second = economy.generation_rate_per_second * 0.95 
            + (atp_gained as f32 / dt) * 0.05;
        
        // Calculate balance health
        economy.balance_deficit = economy.spending_rate_per_second - economy.generation_rate_per_second;
        economy.economy_health = (0.5 - (economy.balance_deficit / 10.0)).clamp(0.0, 1.0);
        
        // Check for economy issues
        if economy.economy_health < 0.2 {
            balance_analyzer_clone.real_time_balance.current_session.balance_issues.push(BalanceIssue {
                issue_type: BalanceIssueType::ATPStarvation,
                severity: 1.0 - economy.economy_health,
                description: "Player struggling with ATP generation".to_string(),
                suggested_fix: "Increase ATP drop rates or decrease upgrade costs".to_string(),
            });
        } else if economy.economy_health > 0.8 {
            balance_analyzer_clone.real_time_balance.current_session.balance_issues.push(BalanceIssue {
                issue_type: BalanceIssueType::ATPAbundance,
                severity: economy.economy_health - 0.8,
                description: "Player has excess ATP".to_string(),
                suggested_fix: "Add more expensive upgrades or reduce ATP generation".to_string(),
            });
        }
    }
}

pub fn progression_balance_system(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    player_query: Query<(&Player, &Health, &CellularUpgrades)>,
    wave_manager: Res<WaveManager>,
    time: Res<Time>,
) {
    if let Ok((player, health, upgrades)) = player_query.single() {
        let current_time = time.elapsed_secs();
        let current_wave = wave_manager.current_wave;
        
        // Calculate values first to avoid multiple borrows
        let health_effectiveness = health.0 as f32 / upgrades.max_health as f32;
        let invincibility_effectiveness = if player.invincible_timer > 0.0 {
            player.invincible_timer / INVINCIBILITY_FRAMES
        } else {
            1.0
        };
        
        let speed_effectiveness = player.speed / MOVEMENT_SPEED_BASE;
        let estimated_power = calculate_player_power_level(health.0, upgrades, &wave_manager);
        
        // Check if movement is too slow (calculate before borrowing)
        let movement_too_slow = {
            let optimal_min = balance_analyzer.progression_metrics.movement_balance.optimal_speed_range.0;
            speed_effectiveness < optimal_min / MOVEMENT_SPEED_BASE
        };
        
        // Now update progression metrics in one borrow
        {
            let progression = &mut balance_analyzer.progression_metrics;
            progression.invincibility_effectiveness = invincibility_effectiveness;
            progression.movement_balance.current_effectiveness = speed_effectiveness;
            progression.player_power_curve.push((current_time, estimated_power));
            
            // Keep only recent data points
            if progression.player_power_curve.len() > 100 {
                progression.player_power_curve.remove(0);
            }
        }
        
        // Handle balance issues in separate borrow
        if movement_too_slow {
            balance_analyzer.real_time_balance.current_session.balance_issues.push(BalanceIssue {
                issue_type: BalanceIssueType::ProgressionTooSlow,
                severity: 0.6,
                description: "Player movement too slow for current wave".to_string(),
                suggested_fix: "Increase movement speed or flagella power-up effectiveness".to_string(),
            });
        }
    }
}

pub fn balance_debug_ui(
    balance_analyzer: Res<BalanceAnalyzer>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    fonts: Res<GameFonts>,
    debug_ui_query: Query<Entity, With<BalanceDebugUI>>,
) {
    // Toggle debug mode with F12
    if input.just_pressed(KeyCode::F12) {
        if debug_ui_query.is_empty() {
            spawn_balance_debug_ui(&mut commands, &balance_analyzer, &fonts);
        } else {
            // Remove debug UI
            for entity in debug_ui_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

// ===== HELPER FUNCTIONS =====

fn get_all_evolutions() -> Vec<EvolutionType> {
    vec![
        EvolutionType::CytoplasmicSpray { damage: 10, fire_rate: 0.1 },
        EvolutionType::PseudopodNetwork { damage: 8, fire_rate: 0.15, tendril_count: 5, spread_angle: 0.6 },
        EvolutionType::BioluminescentBeam { damage: 15, charge_time: 1.0, duration: 2.0, width: 20.0 },
        EvolutionType::SymbioticHunters { damage: 25, fire_rate: 0.8, homing_strength: 2.0, blast_radius: 50.0 },
        EvolutionType::EnzymeBurst { damage: 6, fire_rate: 0.05, acid_damage: 3.0 },
        EvolutionType::ToxinCloud { damage_per_second: 12, cloud_radius: 80.0, duration: 8.0 },
        EvolutionType::ElectricDischarge { damage: 30, chain_count: 4, range: 150.0 },
    ]
}

fn calculate_theoretical_dps(evolution: &EvolutionType) -> f32 {
    match evolution {
        EvolutionType::CytoplasmicSpray { damage, fire_rate } => {
            (*damage as f32) / fire_rate
        }
        EvolutionType::PseudopodNetwork { damage, fire_rate, tendril_count, .. } => {
            (*damage as f32 * *tendril_count as f32) / fire_rate
        }
        EvolutionType::BioluminescentBeam { damage, charge_time, duration, .. } => {
            (*damage as f32 * 12.0 * duration) / (charge_time + duration) // 12 hits per second
        }
        EvolutionType::SymbioticHunters { damage, fire_rate, .. } => {
            (*damage as f32) / fire_rate
        }
        EvolutionType::EnzymeBurst { damage, fire_rate, .. } => {
            (*damage as f32 * 5.0) / fire_rate // 5 projectiles per burst
        }
        EvolutionType::ToxinCloud { damage_per_second, .. } => {
            *damage_per_second as f32
        }
        EvolutionType::ElectricDischarge { damage, chain_count, .. } => {
            (*damage as f32 * *chain_count as f32) / 1.5 // 1.5s cooldown
        }
    }
}

fn get_evolution_cost(evolution: &EvolutionType) -> u32 {
    match evolution {
        EvolutionType::CytoplasmicSpray { .. } => 0,
        EvolutionType::PseudopodNetwork { .. } => 50,
        EvolutionType::BioluminescentBeam { .. } => 100,
        EvolutionType::SymbioticHunters { .. } => 75,
        EvolutionType::EnzymeBurst { .. } => 60,
        EvolutionType::ToxinCloud { .. } => 90,
        EvolutionType::ElectricDischarge { .. } => 150,
    }
}

fn calculate_late_game_viability(weapon_stats: &WeaponPerformance, enemy_scaling: f32) -> f32 {
    let base_viability = weapon_stats.theoretical_dps / (weapon_stats.atp_cost as f32 + 1.0);
    let scaling_factor = base_viability / (enemy_scaling * 0.1 + 1.0);
    scaling_factor.min(1.0)
}

fn analyze_current_balance_issues(
    balance_analyzer: &mut BalanceAnalyzer,
    player: &Player,
    health: &Health,
    evolution_system: &EvolutionSystem,
    atp: &ATP,
    wave_manager: &WaveManager,
    current_time: f32,
) {
    let session = &mut balance_analyzer.real_time_balance.current_session;
    
    // Check for difficulty spikes
    let expected_wave = (current_time / 30.0) as u32; // Expect 1 wave per 30 seconds
    if wave_manager.current_wave < expected_wave.saturating_sub(2) {
        session.balance_issues.push(BalanceIssue {
            issue_type: BalanceIssueType::ProgressionTooSlow,
            severity: 0.7,
            description: "Player progressing slower than expected".to_string(),
            suggested_fix: "Reduce enemy health or increase weapon damage".to_string(),
        });
    }
    
    // Check ATP economy
    if atp.amount == 0 && current_time > 60.0 {
        session.balance_issues.push(BalanceIssue {
            issue_type: BalanceIssueType::ATPStarvation,
            severity: 0.8,
            description: "Player has no ATP after 1 minute".to_string(),
            suggested_fix: "Increase ATP drop rates".to_string(),
        });
    }
    
    // Check health balance
    let health_percentage = health.0 as f32 / 100.0;
    if health_percentage < 0.2 && player.invincible_timer == 0.0 {
        session.balance_issues.push(BalanceIssue {
            issue_type: BalanceIssueType::InvincibilityTooShort,
            severity: 0.6,
            description: "Player at low health with no protection".to_string(),
            suggested_fix: "Increase invincibility duration or health regeneration".to_string(),
        });
    }
}

fn apply_dynamic_balance_adjustments(
    balance_analyzer: &mut BalanceAnalyzer,
    commands: &mut Commands,
) {
    // Apply active adjustments (this would modify game parameters)
    for adjustment in &mut balance_analyzer.real_time_balance.active_adjustments {
        if adjustment.active {
            adjustment.duration -= 1.0/60.0; // Assume 60 FPS
            
            if adjustment.duration <= 0.0 {
                adjustment.active = false;
            }
        }
    }
    
    // Remove inactive adjustments
    balance_analyzer.real_time_balance.active_adjustments.retain(|adj| adj.active);
}

fn calculate_player_power_level(health: i32, upgrades: &CellularUpgrades, wave_manager: &WaveManager) -> f32 {
    let health_factor = health as f32 / 100.0;
    let upgrade_factor = (upgrades.damage_amplification + upgrades.movement_efficiency + upgrades.metabolic_rate) / 3.0;
    let wave_factor = wave_manager.current_wave as f32 / 10.0;
    
    (health_factor + upgrade_factor + wave_factor) / 3.0
}

fn spawn_balance_debug_ui(commands: &mut Commands, balance_analyzer: &BalanceAnalyzer, fonts: &GameFonts) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(150.0),
            width: Val::Px(400.0),
            height: Val::Px(500.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        BorderColor(Color::srgb(0.6, 0.8, 1.0)),
        BalanceDebugUI,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("BALANCE DEBUG (F12 to close)"),
            TextFont { font: fonts.default_font.clone(), font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 1.0, 0.8)),
        ));
        
        // ATP Economy Status
        parent.spawn((
            Text::new(format!(
                "ATP Economy Health: {:.1}%\nGeneration: {:.1}/s\nSpending: {:.1}/s",
                balance_analyzer.atp_economy.economy_health * 100.0,
                balance_analyzer.atp_economy.generation_rate_per_second,
                balance_analyzer.atp_economy.spending_rate_per_second
            )),
            TextFont { font: fonts.default_font.clone(), font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Weapon Performance
        for (name, stats) in &balance_analyzer.weapon_stats {
            let color = if stats.cost_efficiency > 2.0 { 
                Color::srgb(0.8, 1.0, 0.8) 
            } else if stats.cost_efficiency < 1.0 { 
                Color::srgb(1.0, 0.6, 0.6) 
            } else { 
                Color::WHITE 
            };
            
            parent.spawn((
                Text::new(format!(
                    "{}: DPS {:.1} | Eff {:.2} | Use {}",
                    name, stats.theoretical_dps, stats.cost_efficiency, stats.usage_frequency
                )),
                TextFont { font: fonts.default_font.clone(), font_size: 12.0, ..default() },
                TextColor(color),
            ));
        }
        
        // Balance Issues
        if !balance_analyzer.real_time_balance.current_session.balance_issues.is_empty() {
            parent.spawn((
                Text::new("BALANCE ISSUES:"),
                TextFont { font: fonts.default_font.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
            ));
            
            for issue in &balance_analyzer.real_time_balance.current_session.balance_issues {
                let color = match issue.severity {
                    s if s > 0.8 => Color::srgb(1.0, 0.3, 0.3),
                    s if s > 0.5 => Color::srgb(1.0, 0.8, 0.3),
                    _ => Color::srgb(0.8, 0.8, 0.8),
                };
                
                parent.spawn((
                    Text::new(format!("• {}", issue.description)),
                    TextFont { font: fonts.default_font.clone(), font_size: 11.0, ..default() },
                    TextColor(color),
                ));
            }
        }
    });
}

// ===== BALANCE ADJUSTMENT EVENTS =====

#[derive(Event)]
pub struct BalanceAdjustmentEvent {
    pub adjustment_type: AdjustmentType,
    pub target: String,
    pub multiplier: f32,
    pub duration: f32,
}

pub fn handle_balance_adjustments(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    mut adjustment_events: EventReader<BalanceAdjustmentEvent>,
) {
    for event in adjustment_events.read() {
        balance_analyzer.real_time_balance.active_adjustments.push(BalanceAdjustment {
            adjustment_type: event.adjustment_type.clone(),
            target: event.target.clone(),
            multiplier: event.multiplier,
            duration: event.duration,
            active: true,
        });
    }
}

// ===== BALANCE TESTING FRAMEWORK =====

pub fn run_balance_tests(
    balance_analyzer: &BalanceAnalyzer,
    commands: &mut Commands,
) -> BalanceTestResults {
    let mut results = BalanceTestResults::new();
    
    // Test 1: Weapon DPS vs Cost Analysis
    results.weapon_balance = test_weapon_balance(&balance_analyzer.weapon_stats);
    
    // Test 2: ATP Economy Health Check
    results.atp_economy = test_atp_economy(&balance_analyzer.atp_economy);
    
    // Test 3: Progression Curve Analysis
    results.progression_balance = test_progression_balance(&balance_analyzer.progression_metrics);
    
    // Test 4: Late Game Viability
    results.late_game_balance = test_late_game_viability(&balance_analyzer.weapon_stats);
    
    results
}

#[derive(Clone)]
pub struct BalanceTestResults {
    pub weapon_balance: WeaponBalanceTest,
    pub atp_economy: ATPEconomyTest,
    pub progression_balance: ProgressionBalanceTest,
    pub late_game_balance: LateGameTest,
    pub overall_score: f32,
}

#[derive(Clone)]
pub struct WeaponBalanceTest {
    pub overpowered_weapons: Vec<String>,
    pub underpowered_weapons: Vec<String>,
    pub cost_efficiency_issues: Vec<(String, f32)>,
    pub balance_score: f32,
}

#[derive(Clone)]
pub struct ATPEconomyTest {
    pub generation_issues: Vec<String>,
    pub cost_issues: Vec<String>,
    pub unlock_time_problems: Vec<(String, f32)>,
    pub economy_score: f32,
}

#[derive(Clone)]
pub struct ProgressionBalanceTest {
    pub health_scaling_issues: Vec<String>,
    pub movement_balance_issues: Vec<String>,
    pub invincibility_issues: Vec<String>,
    pub progression_score: f32,
}

#[derive(Clone)]
pub struct LateGameTest {
    pub viable_weapons: Vec<String>,
    pub obsolete_weapons: Vec<String>,
    pub scaling_issues: Vec<String>,
    pub late_game_score: f32,
}

impl BalanceTestResults {
    fn new() -> Self {
        Self {
            weapon_balance: WeaponBalanceTest {
                overpowered_weapons: Vec::new(),
                underpowered_weapons: Vec::new(),
                cost_efficiency_issues: Vec::new(),
                balance_score: 0.0,
            },
            atp_economy: ATPEconomyTest {
                generation_issues: Vec::new(),
                cost_issues: Vec::new(),
                unlock_time_problems: Vec::new(),
                economy_score: 0.0,
            },
            progression_balance: ProgressionBalanceTest {
                health_scaling_issues: Vec::new(),
                movement_balance_issues: Vec::new(),
                invincibility_issues: Vec::new(),
                progression_score: 0.0,
            },
            late_game_balance: LateGameTest {
                viable_weapons: Vec::new(),
                obsolete_weapons: Vec::new(),
                scaling_issues: Vec::new(),
                late_game_score: 0.0,
            },
            overall_score: 0.0,
        }
    }
    
    pub fn calculate_overall_score(&mut self) {
        self.overall_score = (
            self.weapon_balance.balance_score +
            self.atp_economy.economy_score +
            self.progression_balance.progression_score +
            self.late_game_balance.late_game_score
        ) / 4.0;
    }
}

fn test_weapon_balance(weapon_stats: &HashMap<String, WeaponPerformance>) -> WeaponBalanceTest {
    let mut test = WeaponBalanceTest {
        overpowered_weapons: Vec::new(),
        underpowered_weapons: Vec::new(),
        cost_efficiency_issues: Vec::new(),
        balance_score: 0.0,
    };
    
    let average_dps: f32 = weapon_stats.values().map(|w| w.theoretical_dps).sum::<f32>() / weapon_stats.len() as f32;
    let average_efficiency: f32 = weapon_stats.values().map(|w| w.cost_efficiency).sum::<f32>() / weapon_stats.len() as f32;
    
    for (name, stats) in weapon_stats {
        // Check for overpowered weapons (DPS > 150% of average)
        if stats.theoretical_dps > average_dps * 1.5 {
            test.overpowered_weapons.push(name.clone());
        }
        
        // Check for underpowered weapons (DPS < 70% of average)
        if stats.theoretical_dps < average_dps * 0.7 {
            test.underpowered_weapons.push(name.clone());
        }
        
        // Check cost efficiency issues
        if stats.cost_efficiency < average_efficiency * 0.6 {
            test.cost_efficiency_issues.push((name.clone(), stats.cost_efficiency));
        }
    }
    
    // Calculate balance score (1.0 = perfect, 0.0 = terrible)
    let issue_count = test.overpowered_weapons.len() + test.underpowered_weapons.len() + test.cost_efficiency_issues.len();
    test.balance_score = (1.0 - (issue_count as f32 / weapon_stats.len() as f32)).max(0.0);
    
    test
}

fn test_atp_economy(atp_economy: &ATPEconomyData) -> ATPEconomyTest {
    let mut test = ATPEconomyTest {
        generation_issues: Vec::new(),
        cost_issues: Vec::new(),
        unlock_time_problems: Vec::new(),
        economy_score: 0.0,
    };
    
    // Check generation rate
    if atp_economy.generation_rate_per_second < 1.0 {
        test.generation_issues.push("ATP generation too slow".to_string());
    }
    if atp_economy.generation_rate_per_second > 5.0 {
        test.generation_issues.push("ATP generation too fast".to_string());
    }
    
    // Check balance deficit
    if atp_economy.balance_deficit > 2.0 {
        test.generation_issues.push("Player spending ATP faster than earning".to_string());
    }
    if atp_economy.balance_deficit < -2.0 {
        test.generation_issues.push("Player earning ATP much faster than spending".to_string());
    }
    
    // Check upgrade costs
    for (upgrade, cost) in &atp_economy.upgrade_costs {
        let expected_unlock_time = *cost as f32 / atp_economy.generation_rate_per_second;
        if expected_unlock_time > 120.0 { // More than 2 minutes
            test.cost_issues.push(format!("{} takes too long to unlock ({:.1}s)", upgrade, expected_unlock_time));
        }
    }
    
    // Check evolution unlock times
    for (evolution, time) in &atp_economy.evolution_unlock_times {
        if *time > 180.0 { // More than 3 minutes
            test.unlock_time_problems.push((evolution.clone(), *time));
        }
    }
    
    test.economy_score = atp_economy.economy_health;
    test
}

fn test_progression_balance(progression: &ProgressionMetrics) -> ProgressionBalanceTest {
    let mut test = ProgressionBalanceTest {
        health_scaling_issues: Vec::new(),
        movement_balance_issues: Vec::new(),
        invincibility_issues: Vec::new(),
        progression_score: 0.0,
    };
    
    // Check health scaling
    for (level, effectiveness) in &progression.health_scaling {
        if *effectiveness < 1.0 {
            test.health_scaling_issues.push(format!("Health upgrade level {} has negative effectiveness", level));
        }
        if *effectiveness > 3.0 {
            test.health_scaling_issues.push(format!("Health upgrade level {} too powerful", level));
        }
    }
    
    // Check movement balance
    if progression.movement_balance.current_effectiveness < 0.8 {
        test.movement_balance_issues.push("Player movement too slow".to_string());
    }
    if progression.movement_balance.current_effectiveness > 1.5 {
        test.movement_balance_issues.push("Player movement too fast".to_string());
    }
    
    // Check invincibility effectiveness
    if progression.invincibility_effectiveness < 0.5 {
        test.invincibility_issues.push("Invincibility frames too short".to_string());
    }
    if progression.invincibility_effectiveness > 2.0 {
        test.invincibility_issues.push("Invincibility frames too long".to_string());
    }
    
    let issue_count = test.health_scaling_issues.len() + test.movement_balance_issues.len() + test.invincibility_issues.len();
    test.progression_score = (1.0 - (issue_count as f32 / 10.0)).max(0.0);
    
    test
}

fn test_late_game_viability(weapon_stats: &HashMap<String, WeaponPerformance>) -> LateGameTest {
    let mut test = LateGameTest {
        viable_weapons: Vec::new(),
        obsolete_weapons: Vec::new(),
        scaling_issues: Vec::new(),
        late_game_score: 0.0,
    };
    
    for (name, stats) in weapon_stats {
        if stats.late_game_viability > 0.7 {
            test.viable_weapons.push(name.clone());
        } else if stats.late_game_viability < 0.3 {
            test.obsolete_weapons.push(name.clone());
        }
        
        // Check for scaling issues
        if stats.usage_frequency > 0 && stats.late_game_viability < 0.4 {
            test.scaling_issues.push(format!("{} becomes obsolete despite usage", name));
        }
    }
    
    test.late_game_score = if weapon_stats.len() > 0 {
        test.viable_weapons.len() as f32 / weapon_stats.len() as f32
    } else {
        0.0
    };
    
    test
}

// ===== BALANCE TUNING SYSTEM =====

pub fn auto_balance_tuning_system(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    mut adjustment_events: EventWriter<BalanceAdjustmentEvent>,
    mut commands: Commands,
    time: Res<Time>,
    mut tuning_timer: Local<f32>,
) {
    *tuning_timer += time.delta_secs();
    
    // Run balance analysis every 10 seconds
    if *tuning_timer >= 10.0 {
        *tuning_timer = 0.0;
        
        let test_results = run_balance_tests(&balance_analyzer, &mut commands);
        
        // Apply automatic adjustments based on test results
        apply_automatic_balance_adjustments(&test_results, &mut adjustment_events);
        
        // Store results for historical analysis
        // This could be expanded to save to file for persistent analysis
    }
}

fn apply_automatic_balance_adjustments(
    test_results: &BalanceTestResults,
    adjustment_events: &mut EventWriter<BalanceAdjustmentEvent>,
) {
    // Adjust overpowered weapons
    for weapon in &test_results.weapon_balance.overpowered_weapons {
        adjustment_events.write( BalanceAdjustmentEvent {
            adjustment_type: AdjustmentType::DamageMultiplier,
            target: weapon.clone(),
            multiplier: 0.9, // Reduce damage by 10%
            duration: 30.0, // For 30 seconds
        });
    }
    
    // Boost underpowered weapons
    for weapon in &test_results.weapon_balance.underpowered_weapons {
        adjustment_events.write( BalanceAdjustmentEvent {
            adjustment_type: AdjustmentType::DamageMultiplier,
            target: weapon.clone(),
            multiplier: 1.15, // Increase damage by 15%
            duration: 30.0,
        });
    }
    
    // Adjust ATP generation if economy is unhealthy
    if test_results.atp_economy.economy_score < 0.3 {
        adjustment_events.write( BalanceAdjustmentEvent {
            adjustment_type: AdjustmentType::ATPGeneration,
            target: "global".to_string(),
            multiplier: 1.3, // Increase ATP generation by 30%
            duration: 60.0,
        });
    } else if test_results.atp_economy.economy_score > 0.8 {
        adjustment_events.write( BalanceAdjustmentEvent {
            adjustment_type: AdjustmentType::ATPGeneration,
            target: "global".to_string(),
            multiplier: 0.8, // Decrease ATP generation by 20%
            duration: 60.0,
        });
    }
}

// ===== BALANCE REPORTING SYSTEM =====

pub fn generate_balance_report(balance_analyzer: &BalanceAnalyzer) -> String {
    let mut report = String::new();
    report.push_str("=== EVOLUTION SYSTEM BALANCE REPORT ===\n\n");
    
    // Weapon Performance Summary
    report.push_str("WEAPON PERFORMANCE:\n");
    for (name, stats) in &balance_analyzer.weapon_stats {
        report.push_str(&format!(
            "  {}: DPS {:.1} | Cost {} ATP | Efficiency {:.2} | Viability {:.1}%\n",
            name, stats.theoretical_dps, stats.atp_cost, stats.cost_efficiency, stats.late_game_viability * 100.0
        ));
    }
    
    // ATP Economy Summary
    report.push_str("\nATP ECONOMY:\n");
    report.push_str(&format!(
        "  Health: {:.1}%\n  Generation: {:.2}/s\n  Spending: {:.2}/s\n  Balance: {:.2}/s\n",
        balance_analyzer.atp_economy.economy_health * 100.0,
        balance_analyzer.atp_economy.generation_rate_per_second,
        balance_analyzer.atp_economy.spending_rate_per_second,
        balance_analyzer.atp_economy.balance_deficit
    ));
    
    // Balance Issues
    report.push_str("\nCURRENT ISSUES:\n");
    for issue in &balance_analyzer.real_time_balance.current_session.balance_issues {
        report.push_str(&format!(
            "  • [{}%] {}\n    Fix: {}\n",
            (issue.severity * 100.0) as u32,
            issue.description,
            issue.suggested_fix
        ));
    }
    
    // Active Adjustments
    report.push_str("\nACTIVE ADJUSTMENTS:\n");
    for adjustment in &balance_analyzer.real_time_balance.active_adjustments {
        if adjustment.active {
            report.push_str(&format!(
                "  • {:?} on {} (x{:.2}) for {:.1}s\n",
                adjustment.adjustment_type,
                adjustment.target,
                adjustment.multiplier,
                adjustment.duration
            ));
        }
    }
    
    report
}

// ===== DEBUG COMMANDS =====

pub fn balance_debug_commands(
    mut balance_analyzer: ResMut<BalanceAnalyzer>,
    input: Res<ButtonInput<KeyCode>>,
    mut adjustment_events: EventWriter<BalanceAdjustmentEvent>,
    mut commands: Commands,
) {
    // F1: Generate and print balance report
    if input.just_pressed(KeyCode::F1) {
        let report = generate_balance_report(&balance_analyzer);
        println!("{}", report);
    }
    
    // F2: Run full balance test suite
    if input.just_pressed(KeyCode::F2) {
        let mut test_results = run_balance_tests(&balance_analyzer, &mut commands);
        test_results.calculate_overall_score();
        println!("Balance Test Results - Overall Score: {:.1}%", test_results.overall_score * 100.0);
        
        // Print detailed results
        if !test_results.weapon_balance.overpowered_weapons.is_empty() {
            println!("Overpowered weapons: {:?}", test_results.weapon_balance.overpowered_weapons);
        }
        if !test_results.weapon_balance.underpowered_weapons.is_empty() {
            println!("Underpowered weapons: {:?}", test_results.weapon_balance.underpowered_weapons);
        }
    }
    
    // F3: Reset all balance adjustments
    if input.just_pressed(KeyCode::F3) {
        balance_analyzer.real_time_balance.active_adjustments.clear();
        println!("All balance adjustments reset");
    }
    
    // F4: Force ATP economy boost (testing)
    if input.just_pressed(KeyCode::F4) {
        adjustment_events.write( BalanceAdjustmentEvent {
            adjustment_type: AdjustmentType::ATPGeneration,
            target: "debug".to_string(),
            multiplier: 2.0,
            duration: 15.0,
        });
        println!("Applied 2x ATP generation boost for 15s");
    }
}

// ===== COMPONENT MARKERS =====

#[derive(Component)]
pub struct BalanceDebugUI;

#[derive(Component)]
pub struct BalanceTestRunner;

// ===== SAVE/LOAD BALANCE DATA =====

#[derive(Serialize, Deserialize)]
pub struct BalanceDataSave {
    pub weapon_performance_history: Vec<HashMap<String, WeaponPerformance>>,
    pub balance_sessions: Vec<BalanceSession>,
    pub optimization_recommendations: Vec<String>,
}

pub fn save_balance_data(balance_analyzer: &BalanceAnalyzer) {
    let save_data = BalanceDataSave {
        weapon_performance_history: vec![balance_analyzer.weapon_stats.clone()],
        balance_sessions: balance_analyzer.real_time_balance.historical_data.clone(),
        optimization_recommendations: generate_optimization_recommendations(balance_analyzer),
    };
    
    if let Ok(json) = serde_json::to_string_pretty(&save_data) {
        if let Err(e) = std::fs::write("balance_data.json", json) {
            eprintln!("Failed to save balance data: {}", e);
        }
    }
}

pub fn load_balance_data(balance_analyzer: &mut BalanceAnalyzer) {
    if let Ok(json) = std::fs::read_to_string("balance_data.json") {
        if let Ok(save_data) = serde_json::from_str::<BalanceDataSave>(&json) {
            balance_analyzer.real_time_balance.historical_data = save_data.balance_sessions;
            println!("Loaded {} historical balance sessions", balance_analyzer.real_time_balance.historical_data.len());
        }
    }
}

fn generate_optimization_recommendations(balance_analyzer: &BalanceAnalyzer) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Analyze weapon usage patterns
    let total_usage: u32 = balance_analyzer.weapon_stats.values().map(|w| w.usage_frequency).sum();
    if total_usage > 0 {
        for (name, stats) in &balance_analyzer.weapon_stats {
            let usage_percentage = (stats.usage_frequency as f32 / total_usage as f32) * 100.0;
            
            if usage_percentage < 5.0 && stats.atp_cost > 0 {
                recommendations.push(format!("Consider reducing cost of {} (only {:.1}% usage)", name, usage_percentage));
            }
            if usage_percentage > 50.0 {
                recommendations.push(format!("Consider nerfing {} or buffing alternatives ({:.1}% usage)", name, usage_percentage));
            }
        }
    }
    
    // Analyze economy health
    if balance_analyzer.atp_economy.economy_health < 0.4 {
        recommendations.push("Increase ATP generation rates across all enemies".to_string());
        recommendations.push("Consider reducing upgrade costs by 10-20%".to_string());
    }
    
    // Analyze progression balance
    if balance_analyzer.progression_metrics.movement_balance.current_effectiveness < 0.9 {
        recommendations.push("Increase base movement speed or flagella effectiveness".to_string());
    }
    
    recommendations
}