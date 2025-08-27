// src/wave_systems.rs - Comprehensive wave design and progression
use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;
use rand::Rng;

// ===== CONSTANTS =====
const WAVE_BASE_HEALTH_MULTIPLIER: f32 = 1.15;
const WAVE_BASE_SPEED_MULTIPLIER: f32 = 1.08;
const TUTORIAL_WAVE_COUNT: u32 = 5;
const MIXED_FORMATION_WAVE_COUNT: u32 = 10;
const ENVIRONMENTAL_WAVE_COUNT: u32 = 15;
const MINI_BOSS_WAVE_COUNT: u32 = 20;
const ENDLESS_START_WAVE: u32 = 21;

const ATP_BASE_REWARD: u32 = 50;
const ATP_WAVE_MULTIPLIER: f32 = 1.12;
const POWERUP_BASE_CHANCE: f32 = 0.15;
const POWERUP_WAVE_REDUCTION: f32 = 0.008;

// ===== WAVE CONFIGURATION =====
#[derive(Resource, Clone)]
pub struct WaveManager {
    pub current_wave: u32,
    pub wave_active: bool,
    pub enemies_remaining: u32,
    pub wave_start_time: f32,
    pub wave_complete_time: f32,
    pub difficulty_multiplier: f32,
    pub environmental_hazards_active: bool,
    pub wave_patterns: Vec<WavePattern>,
}

#[derive(Clone)]
pub struct WavePattern {
    pub wave_number: u32,
    pub wave_type: WaveType,
    pub enemy_spawns: Vec<EnemySpawn>,
    pub environmental_effects: Vec<EnvironmentalHazard>,
    pub narrative_context: String,
    pub completion_rewards: WaveRewards,
    pub spawn_timing: SpawnTiming,
}

#[derive(Clone)]
pub enum WaveType {
    Tutorial { focus_enemy: EnemyType },
    MixedFormation { primary_enemies: Vec<EnemyType>, support_ratio: f32 },
    Environmental { hazard_type: HazardType, enemy_adaptation: bool },
    MiniBoss { boss_type: EnemyType, minion_types: Vec<EnemyType> },
    Endless { scaling_factor: f32 },
}

#[derive(Clone)]
pub struct EnemySpawn {
    pub enemy_type: EnemyType,
    pub spawn_count: u32,
    pub spawn_positions: Vec<SpawnPosition>,
    pub ai_override: Option<EnemyAI>,
    pub health_multiplier: f32,
    pub speed_multiplier: f32,
    pub spawn_delay: f32,
}

#[derive(Clone)]
pub enum SpawnPosition {
    TopCenter,
    TopLeft,
    TopRight,
    SidesAlternating,
    DiagonalApproach { angle: f32 },
    SpiralFormation { radius: f32, arms: u32 },
    RandomScattered { area: f32 },
}

#[derive(Clone)]
pub struct EnvironmentalHazard {
    pub hazard_type: HazardType,
    pub intensity: f32,
    pub duration: f32,
    pub affected_area: f32,
    pub spawn_delay: f32,
}

#[derive(Clone, Copy)]
pub enum HazardType {
    AcidZone,
    ToxicCurrent,
    OxygenDepletion,
    ThermalVent,
    ChemicalSpill,
    KingTide,
}

#[derive(Clone)]
pub struct WaveRewards {
    pub atp_bonus: u32,
    pub powerup_guarantee: Option<PowerUpType>,
    pub evolution_unlock: Option<String>,
    pub score_multiplier: f32,
}

#[derive(Clone)]
pub struct SpawnTiming {
    pub initial_delay: f32,
    pub spawn_interval: f32,
    pub burst_spawning: bool,
    pub adaptive_timing: bool,
}

// ===== WAVE INITIALIZATION =====
impl WaveManager {
    pub fn new() -> Self {
        let wave_patterns = Self::create_wave_patterns();
        Self {
            current_wave: 1,
            wave_active: false,
            enemies_remaining: 0,
            wave_start_time: 0.0,
            wave_complete_time: 0.0,
            difficulty_multiplier: 1.0,
            environmental_hazards_active: false,
            wave_patterns,
        }
    }

    fn create_wave_patterns() -> Vec<WavePattern> {
        let mut patterns = Vec::new();
        
        // ===== TUTORIAL WAVES (1-5) =====
        patterns.push(WavePattern {
            wave_number: 1,
            wave_type: WaveType::Tutorial { focus_enemy: EnemyType::ViralParticle },
            enemy_spawns: vec![EnemySpawn {
                enemy_type: EnemyType::ViralParticle,
                spawn_count: 5,
                spawn_positions: vec![SpawnPosition::TopCenter],
                ai_override: Some(EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) }),
                health_multiplier: 1.0,
                speed_multiplier: 0.8,
                spawn_delay: 0.0,
            }],
            environmental_effects: vec![],
            narrative_context: "First viral particles detected. Basic cellular defenses required.".to_string(),
            completion_rewards: WaveRewards {
                atp_bonus: ATP_BASE_REWARD,
                powerup_guarantee: None,
                evolution_unlock: None,
                score_multiplier: 1.0,
            },
            spawn_timing: SpawnTiming {
                initial_delay: 2.0,
                spawn_interval: 1.2,
                burst_spawning: false,
                adaptive_timing: false,
            },
        });

        patterns.push(WavePattern {
            wave_number: 2,
            wave_type: WaveType::Tutorial { focus_enemy: EnemyType::AggressiveBacteria },
            enemy_spawns: vec![EnemySpawn {
                enemy_type: EnemyType::AggressiveBacteria,
                spawn_count: 4,
                spawn_positions: vec![SpawnPosition::SidesAlternating],
                ai_override: None,
                health_multiplier: 1.0,
                speed_multiplier: 1.0,
                spawn_delay: 0.0,
            }],
            environmental_effects: vec![],
            narrative_context: "Bacterial infection spreading. Enhanced mobility required.".to_string(),
            completion_rewards: WaveRewards {
                atp_bonus: (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER) as u32,
                powerup_guarantee: Some(PowerUpType::Flagella { multiplier: 1.3, duration: 10.0 }),
                evolution_unlock: None,
                score_multiplier: 1.1,
            },
            spawn_timing: SpawnTiming {
                initial_delay: 1.5,
                spawn_interval: 1.8,
                burst_spawning: false,
                adaptive_timing: false,
            },
        });

        // Continue with remaining tutorial waves
        Self::add_tutorial_waves(&mut patterns);
        
        // ===== MIXED FORMATION WAVES (6-10) =====
        Self::add_mixed_formation_waves(&mut patterns);
        
        // ===== ENVIRONMENTAL HAZARD WAVES (11-15) =====
        Self::add_environmental_waves(&mut patterns);
        
        // ===== MINI-BOSS WAVES (16-20) =====
        Self::add_mini_boss_waves(&mut patterns);
        
        patterns
    }

    fn add_tutorial_waves(patterns: &mut Vec<WavePattern>) {
        // Wave 3: Parasitic Protozoa
        patterns.push(WavePattern {
            wave_number: 3,
            wave_type: WaveType::Tutorial { focus_enemy: EnemyType::ParasiticProtozoa },
            enemy_spawns: vec![EnemySpawn {
                enemy_type: EnemyType::ParasiticProtozoa,
                spawn_count: 3,
                spawn_positions: vec![SpawnPosition::DiagonalApproach { angle: 45.0 }],
                ai_override: Some(EnemyAI::Chemotaxis {
                    target_chemical: ChemicalType::PlayerPheromones,
                    sensitivity: 0.8,
                    current_direction: Vec2::new(0.0, -1.0),
                }),
                health_multiplier: 1.1,
                speed_multiplier: 0.9,
                spawn_delay: 0.0,
            }],
            environmental_effects: vec![],
            narrative_context: "Heavy armored parasites detected. Sustained firepower needed.".to_string(),
            completion_rewards: WaveRewards {
                atp_bonus: (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER.powi(2)) as u32,
                powerup_guarantee: Some(PowerUpType::CellularRegeneration { amount: 25 }),
                evolution_unlock: None,
                score_multiplier: 1.2,
            },
            spawn_timing: SpawnTiming {
                initial_delay: 2.5,
                spawn_interval: 3.0,
                burst_spawning: false,
                adaptive_timing: false,
            },
        });

        // Wave 4: Suicidal Spores
        patterns.push(WavePattern {
            wave_number: 4,
            wave_type: WaveType::Tutorial { focus_enemy: EnemyType::SuicidalSpore },
            enemy_spawns: vec![EnemySpawn {
                enemy_type: EnemyType::SuicidalSpore,
                spawn_count: 6,
                spawn_positions: vec![SpawnPosition::RandomScattered { area: 300.0 }],
                ai_override: Some(EnemyAI::Kamikaze {
                    target_pos: Vec2::ZERO,
                    dive_speed: 250.0,
                    acquired_target: false,
                }),
                health_multiplier: 0.8,
                speed_multiplier: 1.2,
                spawn_delay: 0.0,
            }],
            environmental_effects: vec![],
            narrative_context: "Explosive spores incoming. Maintain distance and mobility.".to_string(),
            completion_rewards: WaveRewards {
                atp_bonus: (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER.powi(3)) as u32,
                powerup_guarantee: Some(PowerUpType::CellWall { duration: 15.0 }),
                evolution_unlock: None,
                score_multiplier: 1.3,
            },
            spawn_timing: SpawnTiming {
                initial_delay: 1.0,
                spawn_interval: 1.0,
                burst_spawning: true,
                adaptive_timing: false,
            },
        });

        // Wave 5: Biofilm Colony Introduction
        patterns.push(WavePattern {
            wave_number: 5,
            wave_type: WaveType::Tutorial { focus_enemy: EnemyType::BiofilmColony },
            enemy_spawns: vec![EnemySpawn {
                enemy_type: EnemyType::BiofilmColony,
                spawn_count: 2,
                spawn_positions: vec![SpawnPosition::TopLeft, SpawnPosition::TopRight],
                ai_override: Some(EnemyAI::Turret {
                    rotation: 0.0,
                    shoot_timer: 0.0,
                    detection_range: 280.0,
                }),
                health_multiplier: 1.2,
                speed_multiplier: 0.0,
                spawn_delay: 3.0,
            }],
            environmental_effects: vec![],
            narrative_context: "Stationary biofilm detected. Strategic positioning critical.".to_string(),
            completion_rewards: WaveRewards {
                atp_bonus: (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER.powi(4)) as u32,
                powerup_guarantee: Some(PowerUpType::MitochondriaOvercharge { rate_multiplier: 1.5, duration: 12.0 }),
                evolution_unlock: Some("PseudopodNetwork".to_string()),
                score_multiplier: 1.5,
            },
            spawn_timing: SpawnTiming {
                initial_delay: 3.0,
                spawn_interval: 8.0,
                burst_spawning: false,
                adaptive_timing: false,
            },
        });
    }

    fn add_mixed_formation_waves(patterns: &mut Vec<WavePattern>) {
        for wave_num in 6..=10 {

            let primary_enemies = match wave_num {
                6 => vec![EnemyType::ViralParticle, EnemyType::AggressiveBacteria],
                7 => vec![EnemyType::AggressiveBacteria, EnemyType::ParasiticProtozoa],
                8 => vec![EnemyType::SwarmCell, EnemyType::ViralParticle],
                9 => vec![EnemyType::ParasiticProtozoa, EnemyType::SuicidalSpore],
                10 => vec![EnemyType::BiofilmColony, EnemyType::SwarmCell, EnemyType::AggressiveBacteria],
                _ => vec![EnemyType::ViralParticle],
            };

            let mut enemy_spawns = Vec::new();

            for (i, &enemy_type) in primary_enemies.iter().enumerate() {
                let spawn_count = match enemy_type {
                    EnemyType::BiofilmColony => 1,
                    EnemyType::ParasiticProtozoa => 2,
                    EnemyType::InfectedMacrophage => 1,
                    _ => 4 + (wave_num - 6) / 2,
                };

                let spawn_positions = match i % 3 {
                    0 => vec![SpawnPosition::SpiralFormation { radius: 150.0, arms: 3 }],
                    1 => vec![SpawnPosition::DiagonalApproach { angle: if i % 2 == 0 { 30.0 } else { -30.0 } }],
                    _ => vec![SpawnPosition::SidesAlternating],
                };

                enemy_spawns.push(EnemySpawn {
                    enemy_type,
                    spawn_count,
                    spawn_positions,
                    ai_override: None,
                    health_multiplier: 1.0 + (wave_num - 6) as f32 * 0.1,
                    speed_multiplier: 1.0 + (wave_num - 6) as f32 * 0.05,
                    spawn_delay: i as f32 * 2.0,
                });
            }

            patterns.push(WavePattern {
                wave_number: wave_num,
                wave_type: WaveType::MixedFormation {
                    primary_enemies: primary_enemies.clone(),
                    support_ratio: 0.6,
                },
                enemy_spawns,
                environmental_effects: vec![],
                narrative_context: format!("Mixed pathogen assault detected. Wave {} ecosystem disruption.", wave_num),
                completion_rewards: WaveRewards {
                    atp_bonus: (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER.powi(wave_num as i32 - 1)) as u32,
                    powerup_guarantee: if wave_num % 2 == 0 { Some(PowerUpType::SymbioticBoost { multiplier: 1.4, duration: 8.0 }) } else { None },
                    evolution_unlock: if wave_num == 10 { Some("BioluminescentBeam".to_string()) } else { None },
                    score_multiplier: 1.0 + (wave_num as f32 * 0.1),
                },
                spawn_timing: SpawnTiming {
                    initial_delay: 1.5,
                    spawn_interval: 1.2 - (wave_num - 6) as f32 * 0.1,
                    burst_spawning: wave_num >= 9,
                    adaptive_timing: true,
                },
            });
        }
    }

    fn add_environmental_waves(patterns: &mut Vec<WavePattern>) {
        let hazard_types = [
            HazardType::AcidZone,
            HazardType::ToxicCurrent,
            HazardType::OxygenDepletion,
            HazardType::ThermalVent,
            HazardType::ChemicalSpill,
        ];

        for wave_num in 11..=15 {
            let hazard_type = hazard_types[(wave_num - 11) as usize];
            let adapted_enemies = Self::get_adapted_enemies(hazard_type);

            let environmental_effects = vec![EnvironmentalHazard {
                hazard_type,
                intensity: 0.8 + (wave_num - 11) as f32 * 0.05,
                duration: 45.0,
                affected_area: 200.0 + (wave_num - 11) as f32 * 20.0,
                spawn_delay: 5.0,
            }];

            let enemy_spawns = adapted_enemies.into_iter().enumerate().map(|(i, enemy_type)| {
                EnemySpawn {
                    enemy_type,
                    spawn_count: 3 + (wave_num - 11) / 2,
                    spawn_positions: vec![SpawnPosition::DiagonalApproach { angle: (i as f32 * 60.0) - 90.0 }],
                    ai_override: Self::get_hazard_adapted_ai(hazard_type),
                    health_multiplier: 1.2 + (wave_num - 11) as f32 * 0.15,
                    speed_multiplier: if matches!(hazard_type, HazardType::ToxicCurrent) { 1.3 } else { 1.1 },
                    spawn_delay: i as f32 * 1.5,
                }
            }).collect();

            patterns.push(WavePattern {
                wave_number: wave_num,
                wave_type: WaveType::Environmental {
                    hazard_type,
                    enemy_adaptation: true,
                },
                enemy_spawns,
                environmental_effects,
                narrative_context: Self::get_hazard_narrative(hazard_type, wave_num),
                completion_rewards: WaveRewards {
                    atp_bonus: (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER.powi(wave_num as i32 - 1)) as u32 + 100,
                    powerup_guarantee: Some(Self::get_hazard_counter_powerup(hazard_type)),
                    evolution_unlock: if wave_num == 15 { Some("SymbioticHunters".to_string()) } else { None },
                    score_multiplier: 1.8 + (wave_num as f32 * 0.1),
                },
                spawn_timing: SpawnTiming {
                    initial_delay: 2.0,
                    spawn_interval: 1.0,
                    burst_spawning: false,
                    adaptive_timing: true,
                },
            });
        }
    }

    fn add_mini_boss_waves(patterns: &mut Vec<WavePattern>) {
        let boss_configs = [
            (EnemyType::InfectedMacrophage, vec![EnemyType::ViralParticle, EnemyType::AggressiveBacteria]),
            (EnemyType::ReproductiveVesicle, vec![EnemyType::Offspring, EnemyType::SwarmCell]),
            (EnemyType::InfectedMacrophage, vec![EnemyType::ParasiticProtozoa, EnemyType::SuicidalSpore]),
            (EnemyType::BiofilmColony, vec![EnemyType::SwarmCell, EnemyType::AggressiveBacteria]),
            (EnemyType::InfectedMacrophage, vec![EnemyType::ReproductiveVesicle, EnemyType::ParasiticProtozoa]),
        ];

        for (i, wave_num) in (16..=20).enumerate() {
            let (boss_type, minion_types) = boss_configs[i].clone();

            let boss_type_clone = boss_type.clone();
            let boss_ai_override = Self::get_boss_ai(boss_type.clone());
            let boss_type_description = Self::get_boss_description(boss_type.clone());

            let mut enemy_spawns = vec![
                EnemySpawn {
                    enemy_type: boss_type,
                    spawn_count: 1,
                    spawn_positions: vec![SpawnPosition::TopCenter],
                    ai_override: boss_ai_override,
                    health_multiplier: 2.0 + (wave_num - 16) as f32 * 0.5,
                    speed_multiplier: 1.2,
                    spawn_delay: 8.0,
                }
            ];

            for (j, &minion_type) in minion_types.iter().enumerate() {
                enemy_spawns.push(EnemySpawn {
                    enemy_type: minion_type,
                    spawn_count: 6 + (wave_num - 16),
                    spawn_positions: vec![SpawnPosition::SpiralFormation { radius: 180.0, arms: 4 }],
                    ai_override: Some(EnemyAI::Formation {
                        formation_id: wave_num * 100 + j as u32,
                        position_in_formation: Vec2::ZERO,
                        leader_offset: Vec2::ZERO,
                        formation_timer: 0.0,
                    }),
                    health_multiplier: 1.5 + (wave_num - 16) as f32 * 0.2,
                    speed_multiplier: 1.1 + (wave_num - 16) as f32 * 0.05,
                    spawn_delay: j as f32 * 3.0,
                });
            }

            patterns.push(WavePattern {
                wave_number: wave_num,
                wave_type: WaveType::MiniBoss {
                    boss_type: boss_type_clone,
                    minion_types,
                },
                enemy_spawns,
                environmental_effects: if wave_num >= 18 {
                    vec![EnvironmentalHazard {
                        hazard_type: HazardType::KingTide,
                        intensity: 1.2,
                        duration: 30.0,
                        affected_area: 500.0,
                        spawn_delay: 15.0,
                    }]
                } else { vec![] },
                narrative_context: format!("Major pathogen detected: {}. Coordinated response required.", 
                    boss_type_description),
                completion_rewards: WaveRewards {
                    atp_bonus: (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER.powi(wave_num as i32 - 1)) as u32 * 2,
                    powerup_guarantee: Some(PowerUpType::BinaryFission { clone_duration: 10.0 }),
                    evolution_unlock: Self::get_boss_evolution_unlock(wave_num),
                    score_multiplier: 3.0 + (wave_num as f32 * 0.2),
                },
                spawn_timing: SpawnTiming {
                    initial_delay: 3.0,
                    spawn_interval: 2.0,
                    burst_spawning: false,
                    adaptive_timing: true,
                },
            });
        }
    }

    // ===== HELPER METHODS =====
    fn get_adapted_enemies(hazard_type: HazardType) -> Vec<EnemyType> {
        match hazard_type {
            HazardType::AcidZone => vec![EnemyType::AggressiveBacteria, EnemyType::BiofilmColony],
            HazardType::ToxicCurrent => vec![EnemyType::ParasiticProtozoa, EnemyType::SwarmCell],
            HazardType::OxygenDepletion => vec![EnemyType::SuicidalSpore, EnemyType::ViralParticle],
            HazardType::ThermalVent => vec![EnemyType::SwarmCell, EnemyType::AggressiveBacteria],
            HazardType::ChemicalSpill => vec![EnemyType::BiofilmColony, EnemyType::ParasiticProtozoa],
            HazardType::KingTide => vec![EnemyType::InfectedMacrophage, EnemyType::ReproductiveVesicle],
        }
    }

    fn get_hazard_adapted_ai(hazard_type: HazardType) -> Option<EnemyAI> {
        match hazard_type {
            HazardType::ToxicCurrent => Some(EnemyAI::FluidFlow {
                flow_sensitivity: 2.0,
                base_direction: Vec2::new(0.0, -1.0),
            }),
            HazardType::OxygenDepletion => Some(EnemyAI::Chemotaxis {
                target_chemical: ChemicalType::OxygenSeeker,
                sensitivity: 1.5,
                current_direction: Vec2::new(0.0, -1.0),
            }),
            _ => None,
        }
    }

    fn get_hazard_narrative(hazard_type: HazardType, wave_num: u32) -> String {
        let base = match hazard_type {
            HazardType::AcidZone => "Acidic contamination detected",
            HazardType::ToxicCurrent => "Toxic current flows disrupting ecosystem",
            HazardType::OxygenDepletion => "Oxygen levels critically low",
            HazardType::ThermalVent => "Thermal vents destabilizing environment",
            HazardType::ChemicalSpill => "Industrial chemical spill detected",
            HazardType::KingTide => "Massive tidal disruption incoming",
        };
        format!("{}. Wave {} environmental crisis.", base, wave_num)
    }

    fn get_hazard_counter_powerup(hazard_type: HazardType) -> PowerUpType {
        match hazard_type {
            HazardType::AcidZone => PowerUpType::Osmoregulation { immunity_duration: 20.0 },
            HazardType::ToxicCurrent => PowerUpType::CellWall { duration: 25.0 },
            HazardType::OxygenDepletion => PowerUpType::Photosynthesis { energy_regen: 15.0, duration: 30.0 },
            HazardType::ThermalVent => PowerUpType::CellularRegeneration { amount: 40 },
            HazardType::ChemicalSpill => PowerUpType::Osmoregulation { immunity_duration: 25.0 },
            HazardType::KingTide => PowerUpType::Flagella { multiplier: 1.8, duration: 15.0 },
        }
    }

    fn get_boss_ai(boss_type: EnemyType) -> Option<EnemyAI> {
        match boss_type {
            EnemyType::InfectedMacrophage => Some(EnemyAI::MiniBoss { pattern: 0, timer: 0.0 }),
            EnemyType::ReproductiveVesicle => Some(EnemyAI::Spawner {
                spawn_timer: 2.0,
                spawn_rate: 3.0,
                minions_spawned: 0,
                max_minions: 12,
            }),
            EnemyType::BiofilmColony => Some(EnemyAI::Turret {
                rotation: 0.0,
                shoot_timer: 0.0,
                detection_range: 400.0,
            }),
            _ => None,
        }
    }

    fn get_boss_description(boss_type: EnemyType) -> &'static str {
        match boss_type {
            EnemyType::InfectedMacrophage => "Corrupted immune cell",
            EnemyType::ReproductiveVesicle => "Spawning organism",
            EnemyType::BiofilmColony => "Fortified bacterial cluster",
            _ => "Unknown pathogen",
        }
    }

    fn get_boss_evolution_unlock(wave_num: u32) -> Option<String> {
        match wave_num {
            16 => Some("EnzymeBurst".to_string()),
            18 => Some("ToxinCloud".to_string()),
            20 => Some("ElectricDischarge".to_string()),
            _ => None,
        }
    }

    // ===== WAVE PROGRESSION =====
    pub fn get_current_wave_pattern(&self) -> Option<&WavePattern> {
        if self.current_wave >= ENDLESS_START_WAVE {
            return self.get_endless_wave_pattern();
        }
        self.wave_patterns.iter().find(|p| p.wave_number == self.current_wave)
    }

    fn get_endless_wave_pattern(&self) -> Option<&WavePattern> {
        // Return a procedurally generated endless wave pattern
        // For now, return None to indicate endless mode needs special handling
        None
    }

    pub fn calculate_difficulty_multipliers(&self) -> (f32, f32) {
        let health_mult = WAVE_BASE_HEALTH_MULTIPLIER.powf((self.current_wave - 1) as f32);
        let speed_mult = WAVE_BASE_SPEED_MULTIPLIER.powf((self.current_wave - 1) as f32);
        (health_mult.min(3.0), speed_mult.min(2.0))
    }

    pub fn calculate_atp_reward(&self) -> u32 {
        (ATP_BASE_REWARD as f32 * ATP_WAVE_MULTIPLIER.powf((self.current_wave - 1) as f32)) as u32
    }

    pub fn calculate_powerup_spawn_rate(&self) -> f32 {
        (POWERUP_BASE_CHANCE - (self.current_wave as f32 * POWERUP_WAVE_REDUCTION)).max(0.05)
    }
}

impl Default for WaveManager {
    fn default() -> Self {
        Self::new()
    }
}

// ===== SPAWN POSITION HELPERS =====
impl SpawnPosition {
    pub fn get_world_positions(&self, count: u32) -> Vec<Vec3> {
        let mut positions = Vec::new();
        let screen_width = 1280.0;
        let spawn_y = 400.0;

        match self {
            SpawnPosition::TopCenter => {
                for i in 0..count {
                    let x_offset = (i as f32 - (count - 1) as f32 / 2.0) * 40.0;
                    positions.push(Vec3::new(x_offset, spawn_y, 0.0));
                }
            }
            SpawnPosition::TopLeft => {
                for i in 0..count {
                    positions.push(Vec3::new(-screen_width * 0.3 - i as f32 * 30.0, spawn_y, 0.0));
                }
            }
            SpawnPosition::TopRight => {
                for i in 0..count {
                    positions.push(Vec3::new(screen_width * 0.3 + i as f32 * 30.0, spawn_y, 0.0));
                }
            }
            SpawnPosition::SidesAlternating => {
                for i in 0..count {
                    let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                    let x = side * (screen_width * 0.4 + (i / 2) as f32 * 20.0);
                    positions.push(Vec3::new(x, spawn_y - i as f32 * 15.0, 0.0));
                }
            }
            SpawnPosition::DiagonalApproach { angle } => {
                let rad = angle.to_radians();
                for i in 0..count {
                    let distance = 300.0 + i as f32 * 50.0;
                    let x = rad.sin() * distance;
                    let y = spawn_y + rad.cos() * distance * 0.3;
                    positions.push(Vec3::new(x, y, 0.0));
                }
            }
            SpawnPosition::SpiralFormation { radius, arms } => {
                let points_per_arm = (count + arms - 1) / arms;
                for arm in 0..*arms {
                    for point in 0..points_per_arm {
                        if arm * points_per_arm + point >= count { break; }
                        
                        let angle = (arm as f32 * std::f32::consts::TAU / *arms as f32) + 
                                   (point as f32 * 0.5);
                        let r = *radius * (0.5 + point as f32 * 0.5 / points_per_arm as f32);
                        let x = angle.cos() * r;
                        let y = spawn_y + angle.sin() * r * 0.3;
                        positions.push(Vec3::new(x, y, 0.0));
                    }
                }
            }
            SpawnPosition::RandomScattered { area } => {
                let mut rng = rand::rng();
                for _ in 0..count {
                    let x = rng.random_range(-area / 2.0..area / 2.0);
                    let y = spawn_y + rng.random_range(-50.0..100.0);
                    positions.push(Vec3::new(x, y, 0.0));
                }
            }
        }
        positions
    }
}

// ===== WAVE EXECUTION SYSTEMS =====
pub fn wave_progression_system(
    mut wave_manager: ResMut<WaveManager>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    enemy_query: Query<&Enemy>,
    time: Res<Time>,
) {
    // Check if current wave is complete
    if wave_manager.wave_active {
        let living_enemies = enemy_query.iter().count();
        if living_enemies == 0 && wave_manager.enemies_remaining == 0 {
            complete_current_wave(&mut wave_manager, time.elapsed_secs());
        }
        return;
    }

    // Start next wave if ready
    if should_start_next_wave(&wave_manager, &enemy_spawner, time.elapsed_secs()) {
        start_wave(&mut wave_manager, &mut spawn_events, time.elapsed_secs());
    }
}

pub fn wave_spawning_system(
    wave_manager: Res<WaveManager>,
    mut spawn_events: EventWriter<SpawnEnemy>,
    mut local_timer: Local<f32>,
    mut spawn_queue: Local<Vec<QueuedSpawn>>,
    time: Res<Time>,
) {
    if !wave_manager.wave_active {
        return;
    }

    *local_timer += time.delta_secs();

    // Process spawn queue
    spawn_queue.retain(|queued| {
        if *local_timer >= queued.spawn_time {
            spawn_events.write(SpawnEnemy {
                position: queued.position,
                ai_type: queued.ai_type.clone(),
                enemy_type: queued.enemy_type.clone(),
            });
            false // Remove from queue
        } else {
            true // Keep in queue
        }
    });
}

pub fn environmental_hazard_system(
    mut commands: Commands,
    wave_manager: Res<WaveManager>,
    mut hazard_timer: Local<f32>,
    mut active_hazards: Local<Vec<ActiveHazard>>,
    mut hazard_sprites: Query<&mut Sprite, With<EnvironmentalZone>>,
    chemical_environment: ResMut<ChemicalEnvironment>,
    mut tidal_events: EventWriter<TidalEvent>,
    time: Res<Time>,
) {
    if !wave_manager.wave_active {
        return;
    }

    *hazard_timer += time.delta_secs();

    // Spawn new hazards
    if let Some(pattern) = wave_manager.get_current_wave_pattern() {
        for hazard in &pattern.environmental_effects {
            if *hazard_timer >= hazard.spawn_delay && 
               !active_hazards.iter().any(|h| h.hazard_type as u8 == hazard.hazard_type as u8) {
                
                spawn_environmental_hazard(
                    &mut commands,
                    hazard,
                    &mut active_hazards,
                    &mut tidal_events,
                    time.elapsed_secs(),
                );
            }
        }
    }

    // Update active hazards
    active_hazards.retain_mut(|hazard| {
        hazard.remaining_time -= time.delta_secs();
        if hazard.remaining_time <= 0.0 {
            cleanup_hazard(&mut commands, hazard);
            false
        } else {
            update_hazard_effects(hazard, &mut hazard_sprites, time.delta_secs());
            true
        }
    });
}

// ===== HELPER STRUCTURES =====
#[derive(Clone)]
pub struct QueuedSpawn {
    position: Vec3,
    ai_type: EnemyAI,
    enemy_type: EnemyType,
    spawn_time: f32,
}

pub struct ActiveHazard {
    hazard_type: HazardType,
    position: Vec3,
    intensity: f32,
    remaining_time: f32,
    affected_area: f32,
    entity: Option<Entity>,
}

// ===== HELPER FUNCTIONS =====
pub fn complete_current_wave(wave_manager: &mut WaveManager, current_time: f32) {
    wave_manager.wave_active = false;
    wave_manager.wave_complete_time = current_time;
    wave_manager.current_wave += 1;
    
    if wave_manager.current_wave >= ENDLESS_START_WAVE {
        wave_manager.difficulty_multiplier += 0.2;
    }
}

fn should_start_next_wave(wave_manager: &WaveManager, enemy_spawner: &EnemySpawner, current_time: f32) -> bool {
    let time_since_complete = current_time - wave_manager.wave_complete_time;
    let min_delay = if wave_manager.current_wave <= TUTORIAL_WAVE_COUNT { 5.0 } else { 3.0 };
    
    time_since_complete >= min_delay
}

fn start_wave(
    wave_manager: &mut WaveManager,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    current_time: f32,
) {
    wave_manager.wave_active = true;
    wave_manager.wave_start_time = current_time;
    wave_manager.enemies_remaining = 0;

    if let Some(pattern) = wave_manager.clone().get_current_wave_pattern() {
        schedule_wave_spawns(pattern, spawn_events, current_time, wave_manager);
    } else if wave_manager.current_wave >= ENDLESS_START_WAVE {
        generate_endless_wave(wave_manager, spawn_events, current_time);
    }
}

fn schedule_wave_spawns(
    pattern: &WavePattern,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    start_time: f32,
    wave_manager: &mut WaveManager,
) {
    let (health_mult, speed_mult) = wave_manager.calculate_difficulty_multipliers();
    
    for enemy_spawn in &pattern.enemy_spawns {
        let positions = enemy_spawn.spawn_positions[0].get_world_positions(enemy_spawn.spawn_count);
        
        for (i, position) in positions.into_iter().enumerate() {
            let spawn_delay = enemy_spawn.spawn_delay + 
                            pattern.spawn_timing.initial_delay + 
                            i as f32 * pattern.spawn_timing.spawn_interval;

            let enemy_spawn_clone = enemy_spawn.clone();
            let enemy_type = enemy_spawn_clone.enemy_type;
            spawn_events.write(SpawnEnemy {
                position,
                ai_type: enemy_spawn.ai_override.clone().unwrap_or_else(|| {
                    get_default_ai_for_enemy(enemy_spawn_clone.enemy_type)
                }),
                enemy_type,
            });
            
            wave_manager.enemies_remaining += 1;
        }
    }
}

fn generate_endless_wave(
    wave_manager: &mut WaveManager,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    current_time: f32,
) {
    // Endless mode generation
    let wave_excess = wave_manager.current_wave - ENDLESS_START_WAVE;
    let enemy_count = 8 + wave_excess * 2;
    let enemy_types = [
        EnemyType::ViralParticle,
        EnemyType::AggressiveBacteria,
        EnemyType::ParasiticProtozoa,
        EnemyType::SuicidalSpore,
        EnemyType::SwarmCell,
        EnemyType::BiofilmColony,
    ];

    let mut rng = rand::rng();
    for i in 0..enemy_count {
        let enemy_type = enemy_types[rng.random_range(0..enemy_types.len())];
        let spawn_pos = SpawnPosition::RandomScattered { area: 600.0 };
        let positions = spawn_pos.get_world_positions(1);
        let enemy_type_clone = enemy_type.clone();

        spawn_events.write(SpawnEnemy {
            position: positions[0],
            ai_type: get_default_ai_for_enemy(enemy_type),
            enemy_type: enemy_type_clone,
        });
        
        wave_manager.enemies_remaining += 1;
    }

    // Spawn mini-boss every 5 waves in endless mode
    if wave_excess % 5 == 0 {
        spawn_events.write(SpawnEnemy {
            position: Vec3::new(0.0, 400.0, 0.0),
            ai_type: EnemyAI::MiniBoss { pattern: 0, timer: 0.0 },
            enemy_type: EnemyType::InfectedMacrophage,
        });
        wave_manager.enemies_remaining += 1;
    }
}

fn get_default_ai_for_enemy(enemy_type: EnemyType) -> EnemyAI {
    match enemy_type {
        EnemyType::ViralParticle => EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) },
        EnemyType::AggressiveBacteria => EnemyAI::Sine { amplitude: 60.0, frequency: 1.5, phase: 0.0 },
        EnemyType::ParasiticProtozoa => EnemyAI::Chemotaxis {
            target_chemical: ChemicalType::PlayerPheromones,
            sensitivity: 1.0,
            current_direction: Vec2::new(0.0, -1.0),
        },
        EnemyType::SuicidalSpore => EnemyAI::Kamikaze {
            target_pos: Vec2::ZERO,
            dive_speed: 200.0,
            acquired_target: false,
        },
        EnemyType::BiofilmColony => EnemyAI::Turret {
            rotation: 0.0,
            shoot_timer: 0.0,
            detection_range: 250.0,
        },
        EnemyType::SwarmCell => EnemyAI::Formation {
            formation_id: 0,
            position_in_formation: Vec2::ZERO,
            leader_offset: Vec2::ZERO,
            formation_timer: 0.0,
        },
        EnemyType::ReproductiveVesicle => EnemyAI::Spawner {
            spawn_timer: 3.0,
            spawn_rate: 4.0,
            minions_spawned: 0,
            max_minions: 8,
        },
        EnemyType::Offspring => EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) },
        EnemyType::InfectedMacrophage => EnemyAI::MiniBoss { pattern: 0, timer: 0.0 },
    }
}

fn spawn_environmental_hazard(
    commands: &mut Commands,
    hazard: &EnvironmentalHazard,
    active_hazards: &mut Vec<ActiveHazard>,
    tidal_events: &mut EventWriter<TidalEvent>,
    current_time: f32,
) {
    match hazard.hazard_type {
        HazardType::KingTide => {
            tidal_events.write(TidalEvent::KingTideBegin {
                intensity: hazard.intensity,
                duration: hazard.duration,
            });
        }
        _ => {
            // Spawn visual hazard entity
            let entity = commands.spawn((
                Sprite {
                    color: get_hazard_color(hazard.hazard_type),
                    custom_size: Some(Vec2::splat(hazard.affected_area)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -50.0),
                EnvironmentalZone {
                    zone_type: hazard_to_zone_type(hazard.hazard_type),
                    intensity: hazard.intensity,
                    effect_radius: hazard.affected_area * 0.5,
                },
            )).id();

            active_hazards.push(ActiveHazard {
                hazard_type: hazard.hazard_type,
                position: Vec3::ZERO,
                intensity: hazard.intensity,
                remaining_time: hazard.duration,
                affected_area: hazard.affected_area,
                entity: Some(entity),
            });
        }
    }
}

fn cleanup_hazard(commands: &mut Commands, hazard: &ActiveHazard) {
    if let Some(entity) = hazard.entity {
        commands.entity(entity).despawn();
    }
}

fn multiply_color(color: Color, factor: f32) -> Color {
    let linear_color: LinearRgba = color.into();
    let vec = linear_color.to_vec4() * factor;
    LinearRgba::from_vec4(vec).into()
}

fn update_hazard_effects(
    hazard: &mut ActiveHazard,
    hazard_sprites: &mut Query<&mut Sprite, With<EnvironmentalZone>>,
    delta: f32,
) {
    // Update hazard visual effects based on remaining time
    let fade_factor = (hazard.remaining_time / 45.0).clamp(0.3, 1.0);
    
    if let Some(entity) = hazard.entity {
        if let Ok(mut sprite) = hazard_sprites.get_mut(entity) {
            let mut base_color = get_hazard_color(hazard.hazard_type);
            base_color.set_alpha(base_color.alpha() * fade_factor);
            sprite.color = base_color;
            
            // Add pulsing effect for more dangerous hazards
            if hazard.intensity > 1.0 {
                let pulse = (hazard.remaining_time * 4.0).sin() * 0.2 + 0.8;
                sprite.color = multiply_color(sprite.color, pulse);
            }
        }
    }
}


fn get_hazard_color(hazard_type: HazardType) -> Color {
    match hazard_type {
        HazardType::AcidZone => Color::srgba(1.0, 0.3, 0.3, 0.4),
        HazardType::ToxicCurrent => Color::srgba(0.5, 1.0, 0.3, 0.3),
        HazardType::OxygenDepletion => Color::srgba(0.3, 0.3, 1.0, 0.5),
        HazardType::ThermalVent => Color::srgba(1.0, 0.6, 0.2, 0.6),
        HazardType::ChemicalSpill => Color::srgba(0.8, 0.2, 0.8, 0.4),
        HazardType::KingTide => Color::srgba(0.2, 0.8, 1.0, 0.7),
    }
}

fn hazard_to_zone_type(hazard_type: HazardType) -> ZoneType {
    match hazard_type {
        HazardType::AcidZone => ZoneType::Acidic,
        HazardType::ToxicCurrent => ZoneType::Toxic,
        HazardType::OxygenDepletion => ZoneType::Hypoxic,
        HazardType::ThermalVent => ZoneType::Thermal,
        HazardType::ChemicalSpill => ZoneType::Toxic,
        HazardType::KingTide => ZoneType::Current,
    }
}