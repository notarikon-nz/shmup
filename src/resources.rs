// src/resources.rs - Updated with complete menu system support
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap,HashSet};
use crate::pause_menu::*;
use crate::stage_summary::*;

// ===== FONTS =====
#[derive(Resource)]
pub struct GameFonts {
    pub default_font: Handle<Font>,
}

// ===== CORE ASSETS =====
#[derive(Resource)]
pub struct GameAssets {
    // Player & General
    pub player_texture: Handle<Image>,
    pub projectile_texture: Handle<Image>,
    pub explosion_texture: Handle<Image>,
    pub particle_texture: Handle<Image>,
    pub barrier_texture: Handle<Image>,

    // Enemy Textures -> Unique Sprites
    pub enemy_texture: Handle<Image>,
    pub viral_particle_texture: Handle<Image>,
    pub aggressive_bacteria_texture: Handle<Image>,
    pub parasitic_protozoa_texture: Handle<Image>,
    pub infected_macrophage_texture: Handle<Image>,
    pub suicidal_spore_texture: Handle<Image>,
    pub biofilm_colony_texture: Handle<Image>,
    pub swarm_cell_texture: Handle<Image>,
    pub reproductive_vesicle_texture: Handle<Image>,
    pub offspring_texture: Handle<Image>,

    // PowerUp Textures
    pub health_powerup_texture: Handle<Image>,
    pub shield_powerup_texture: Handle<Image>,
    pub speed_powerup_texture: Handle<Image>,
    pub multiplier_powerup_texture: Handle<Image>,
    pub rapidfire_powerup_texture: Handle<Image>,

    // Audio
    pub sfx_shoot: Handle<AudioSource>,
    pub sfx_explosion: Handle<AudioSource>,
    pub sfx_powerup: Handle<AudioSource>,
    pub music: Handle<AudioSource>,

    // Card System
    pub permanent_card_texture: Handle<Image>,
    pub temporal_card_texture: Handle<Image>,
    pub green_box_texture: Handle<Image>,
    
    // Infrastructure textures
    pub infrastructure_pipe_texture: Handle<Image>,
    pub infrastructure_vat_texture: Handle<Image>,
    pub infrastructure_emitter_texture: Handle<Image>,
    pub infrastructure_factory_texture: Handle<Image>,
    
    // Wing cannon and missile textures
    pub wing_cannon_texture: Handle<Image>,
    pub missile_texture: Handle<Image>,    
}

// ===== GAME STATES (Complete Menu System) =====
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    TitleScreen,
    Settings,
    HighScores,
    Playing,
    StageSummary, // New state
    Paused,
    GameOver,
    Controls, // Legacy - now used for Settings
    None,
}

// In this case, instead of deriving `States`, we derive `SubStates`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
// And we need to add an attribute to let us know what the source state is
// and what value it needs to have. This will ensure that unless we're
// in [`AppState::InGame`], the [`IsPaused`] state resource
// will not exist.
#[source(GameState = GameState::Playing)]
#[states(scoped_entities)]
pub enum IsPaused {
    #[default]
    Running,
    Paused,
}

// ===== MENU SYSTEM RESOURCES =====
#[derive(Resource)]
pub struct LoadingAssets { 
    pub handles: Vec<UntypedHandle> 
}

#[derive(Resource)]
pub struct AudioMenuSettings { 
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for AudioMenuSettings {
    fn default() -> Self { 
        Self { 
            master_volume: 0.7,
            sfx_volume: 0.8,
            music_volume: 0.6,
        } 
    }
}

#[derive(Resource)]
pub struct MenuSettings {
    pub fullscreen: bool,
    pub resolution: (f32, f32),
    pub show_fps: bool,
    pub particles_enabled: bool,
}

impl Default for MenuSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            resolution: (1280.0, 720.0),
            show_fps: false,
            particles_enabled: true,
        }
    }
}

// ===== PARTICLE SYSTEM =====
#[derive(Resource)]
pub struct ParticlePool {
    pub entities: Vec<Entity>,
    pub index: usize,
}

// ===== GAME CORE RESOURCES =====
#[derive(Resource, Default)]
pub struct OldInputState {
    pub movement: Vec2,
    pub shooting: bool,
    pub shoot_timer: f32,
    pub gamepad: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct EnemySpawner {
    pub spawn_timer: f32,
    pub wave_timer: f32,
    pub enemies_spawned: u32,
    pub powerup_timer: f32,
}

#[derive(Resource, Default)]
pub struct GameStarted(pub bool);

#[derive(Resource, Default)]
pub struct ShootingState {
    pub rate_multiplier: f32,
    pub base_rate: f32,
}

// ===== BIOLOGICAL ENVIRONMENT =====
#[derive(Resource)]
pub struct FluidEnvironment {
    pub current_field: Vec<Vec2>,
    pub grid_size: usize,      // Single dimension for square grid
    pub cell_size: f32,        // 20.0 matches your biological systems
    pub tidal_phase: f32,
    pub turbulence_intensity: f32,
}


impl Default for FluidEnvironment {
   fn default() -> Self {
       Self {
           current_field: vec![Vec2::ZERO; 64 * 48], // 64x48 grid for 1024x768 screen
           grid_size: 64 * 48,
           cell_size: 16.0,
           tidal_phase: 0.0, 
           turbulence_intensity: 0.0,
       }
   }
}

#[derive(Clone)]
pub struct TemperatureZone {
    pub center: Vec2,
    pub radius: f32,
    pub temperature: f32, // Kelvin
    pub intensity: f32,
}

#[derive(Clone, Default)]
pub enum EnvironmentType {
    #[default]
    OpenOcean,
    TidalPool,
    DeepSea,
    CoralReef,
    PollutedWater,
    ThermalVent,
}


#[derive(Resource, Clone)]
pub struct ChemicalEnvironment {
    pub ph_zones: Vec<ChemicalZone>,
    pub oxygen_zones: Vec<OxygenZone>,
    pub base_ph: f32,
    pub base_oxygen: f32,
    pub diffusion_rate: f32,
}

#[derive(Clone)]
pub struct ChemicalZone {
    pub position: Vec2,
    pub radius: f32,
    pub ph_level: f32,
    pub intensity: f32,

    pub center: Vec2,
    pub toxicity: f32,
    pub oxygen_level: f32,    
}

#[derive(Clone)]
pub struct OxygenZone {
    pub position: Vec2,
    pub radius: f32,
    pub oxygen_level: f32,
    pub depletion_rate: f32,
}

impl Default for ChemicalEnvironment {
    fn default() -> Self {
        Self {
            ph_zones: Vec::new(),
            oxygen_zones: Vec::new(),
            base_ph: 7.0,
            base_oxygen: 0.5,
            diffusion_rate: 0.1,
        }
    }
}

// ===== CURRENT GENERATION =====
#[derive(Resource)]
pub struct CurrentGenerator {
    pub noise_offset: Vec2,
    pub tidal_cycle: f32,
    pub thermal_vents: Vec<ThermalVent>,
    pub major_currents: Vec<MajorCurrent>,
    pub update_timer: f32,
}

#[derive(Clone)]
pub struct ThermalVent {
    pub position: Vec2,
    pub strength: f32,
    pub temperature: f32,
    pub active: bool,
}

#[derive(Clone)]
pub struct MajorCurrent {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub strength: f32,
    pub width: f32,
}

impl Default for CurrentGenerator {
    fn default() -> Self {
        Self {
            noise_offset: Vec2::ZERO,
            tidal_cycle: 0.0,
            thermal_vents: vec![
                ThermalVent {
                    position: Vec2::new(-300.0, 200.0),
                    strength: 150.0,
                    temperature: 40.0,
                    active: true,
                },
                ThermalVent {
                    position: Vec2::new(400.0, -150.0),
                    strength: 200.0,
                    temperature: 35.0,
                    active: true,
                },
            ],
            major_currents: vec![
                MajorCurrent {
                    start_pos: Vec2::new(-600.0, 300.0),
                    end_pos: Vec2::new(600.0, -300.0),
                    strength: 100.0,
                    width: 150.0,
                },
            ],
            update_timer: 0.0,
        }
    }
}

// ===== BIOLUMINESCENCE =====
#[derive(Resource)]
pub struct BioluminescenceManager {
    pub ambient_glow: f32,
    pub pulse_sources: Vec<BioluminescentSource>,
    pub player_glow_intensity: f32,
    pub environment_reactivity: f32,
}

#[derive(Clone)]
pub struct BioluminescentSource {
    pub position: Vec2,
    pub color: Color,
    pub intensity: f32,
    pub pulse_frequency: f32,
    pub radius: f32,
    pub organic_variation: f32,
}

impl Default for BioluminescenceManager {
    fn default() -> Self {
        Self {
            ambient_glow: 0.1,
            pulse_sources: Vec::new(),
            player_glow_intensity: 0.8,
            environment_reactivity: 1.0,
        }
    }
}

// ===== ECOSYSTEM STATE =====
#[derive(Resource)]
pub struct EcosystemState {
    pub health: f32,
    pub infection_level: f32,
    pub symbiotic_activity: f32,
    pub nutrient_density: f32,
    pub ph_stability: f32,
    pub oxygen_circulation: f32,
    pub population_balance: BiomePopulation,
}

#[derive(Clone)]
pub struct BiomePopulation {
    pub beneficial_microbes: u32,
    pub neutral_organisms: u32,
    pub pathogenic_threats: u32,
    pub symbiotic_pairs: u32,
}

impl Default for EcosystemState {
    fn default() -> Self {
        Self {
            health: 1.0,
            infection_level: 0.0,
            symbiotic_activity: 0.5,
            nutrient_density: 0.7,
            ph_stability: 0.8,
            oxygen_circulation: 0.9,
            population_balance: BiomePopulation {
                beneficial_microbes: 50,
                neutral_organisms: 30,
                pathogenic_threats: 20,
                symbiotic_pairs: 10,
            },
        }
    }
}

// ===== TIDAL PHYSICS =====
#[derive(Resource)]
pub struct TidalPoolPhysics {
    pub tide_level: f32,
    pub tide_cycle_speed: f32,
    pub wave_intensity: f32,
    pub current_strength: f32,
    pub surface_tension: f32,
    pub water_viscosity: f32,
    pub temperature: f32,
    pub salinity: f32,
    pub king_tide_active: bool,
    pub king_tide_timer: f32,
    pub king_tide_intensity: f32,
}

impl Default for TidalPoolPhysics {
    fn default() -> Self {
        Self {
            tide_level: 0.0,
            tide_cycle_speed: 0.02,
            wave_intensity: 0.5,
            current_strength: 1.0,
            surface_tension: 0.8,
            water_viscosity: 0.9,
            temperature: 20.0,
            salinity: 3.5,
            king_tide_active: false,
            king_tide_timer: 0.0,
            king_tide_intensity: 1.0,
        }
    }
}

// ===== TIDAL STATE =====
use crate::components::TidePhase;

#[derive(Resource)]
pub struct TidalState {
    pub last_king_tide: f32,
    pub current_tide_phase: TidePhase,
    pub debris_active: bool,
}

impl Default for TidalState {
    fn default() -> Self {
        Self {
            last_king_tide: 0.0,
            current_tide_phase: TidePhase::Rising,
            debris_active: false,
        }
    }
}

// ===== SCREEN SHAKE =====
#[derive(Resource)]
pub struct ScreenShakeResource {
    pub trauma: f32,
    pub max_trauma: f32,
    pub decay_rate: f32,
    pub shake_intensity: f32,
    pub rotation_factor: f32,
}

impl Default for ScreenShakeResource {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            max_trauma: 1.0,
            decay_rate: 2.5,
            shake_intensity: 25.0,
            rotation_factor: 0.02,
        }
    }
}

// ===== HIGH SCORES =====
#[derive(Resource, Clone, Default)]
pub struct GameScore {
    pub current: u32,
    pub high_scores: Vec<u32>,
    pub score_multiplier: f32,
    pub multiplier_timer: f32,
    
    // Stage-specific scoring
    pub stage_score: u32,
    pub stage_atp_collected: u32,
    pub stage_atp_total: u32,
    
    // Enhanced statistics
    pub total_atp_collected: u64,
    pub enemies_defeated: u32,
    pub stages_completed: u32,
    pub perfect_stages: u32,
    pub cards_collected: u32,
    pub infrastructure_destroyed: u32,
    
    // High score data with more detail
    pub high_score_data: Option<HighScoreData>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScoreData {
    pub scores: Vec<HighScoreEntry>,
    pub total_games_played: u32,
    pub total_play_time: f32,
    pub longest_survival_time: f32,
    pub best_evolution_reached: String,
    pub total_cards_found: u32,
    pub favorite_weapon_type: String,
    pub total_atp_collected: u64,
    pub enemies_defeated: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScoreEntry {
    pub score: u32,
    pub date: String,
    pub evolution_type: String,
    pub stages_reached: u32,
    pub cards_collected: u32,
    pub survival_time: f32,
    pub waves_survived: u32,
    pub time_played: f32,
}

impl Default for HighScoreData {
    fn default() -> Self {
        Self {
            scores: vec![
                HighScoreEntry {
                    score: 10000,
                    date: "2025-01-01".to_string(),
                    evolution_type: "Cytoplasmic Spray".to_string(),
                    stages_reached: 10,
                    cards_collected: 3,
                    survival_time: 300.0,
                    waves_survived: 50,
                    time_played: 300.0,
                },
                HighScoreEntry {
                    score: 7500,
                    date: "2025-01-01".to_string(),
                    evolution_type: "Pseudopod Network".to_string(),
                    stages_reached: 8,
                    cards_collected: 2,
                    survival_time: 240.0,
                    waves_survived: 40,
                    time_played: 240.0,
                },
                HighScoreEntry {
                    score: 5000,
                    date: "2025-01-01".to_string(),
                    evolution_type: "Bioluminescent Beam".to_string(),
                    stages_reached: 6,
                    cards_collected: 1,
                    survival_time: 180.0,
                    waves_survived: 30,
                    time_played: 180.0,
                },
            ],
            total_games_played: 3,
            best_evolution_reached: "Bioluminescent Beam".to_string(),
            longest_survival_time: 300.0,
            total_atp_collected: 2500,
            enemies_defeated: 150,
            favorite_weapon_type: "None".to_string(),
            total_cards_found: 0,
            total_play_time: 0.0,
        }
    }
}

// ===== SCALE MANAGEMENT =====
#[derive(Resource)]
pub struct ScaleManager {
    pub current_scale: f32,
    pub target_scale: f32,
    pub transition_speed: f32,
    pub scale_levels: Vec<ScaleLevel>,
    pub physics_scale_factor: f32,
}

#[derive(Clone)]
pub struct ScaleLevel {
    pub scale: f32,
    pub name: String,
    pub physics_multiplier: f32,
    pub enemy_spawn_rates: Vec<f32>,
    pub environmental_effects: Vec<EnvironmentalEffect>,
}

#[derive(Clone)]
pub struct EnvironmentalEffect {
    pub effect_type: String,
    pub intensity: f32,
    pub radius: f32,
}

impl Default for ScaleManager {
    fn default() -> Self {
        Self {
            current_scale: 1.0,
            target_scale: 1.0,
            transition_speed: 2.0,
            scale_levels: vec![
                ScaleLevel {
                    scale: 0.1,
                    name: "Molecular Level".to_string(),
                    physics_multiplier: 3.0,
                    enemy_spawn_rates: vec![0.5, 1.0, 2.0],
                    environmental_effects: vec![],
                },
                ScaleLevel {
                    scale: 1.0,
                    name: "Cellular Level".to_string(),
                    physics_multiplier: 1.0,
                    enemy_spawn_rates: vec![1.0, 1.5, 2.5],
                    environmental_effects: vec![],
                },
                ScaleLevel {
                    scale: 10.0,
                    name: "Tissue Level".to_string(),
                    physics_multiplier: 0.3,
                    enemy_spawn_rates: vec![2.0, 3.0, 4.0],
                    environmental_effects: vec![],
                },
            ],
            physics_scale_factor: 1.0,
        }
    }
}

// ===== MICROSCOPIC PHYSICS =====
#[derive(Resource)]
pub struct MicroscopicPhysics {
    pub brownian_motion_intensity: f32,
    pub molecular_collision_rate: f32,
    pub diffusion_coefficients: Vec<DiffusionData>,
    pub electrostatic_forces: bool,
    pub van_der_waals_forces: bool,
    pub surface_adhesion: f32,
}

#[derive(Clone)]
pub struct DiffusionData {
    pub substance_name: String,
    pub diffusion_rate: f32,
    pub molecular_weight: f32,
}

impl Default for MicroscopicPhysics {
    fn default() -> Self {
        Self {
            brownian_motion_intensity: 1.0,
            molecular_collision_rate: 0.8,
            diffusion_coefficients: vec![
                DiffusionData {
                    substance_name: "Oxygen".to_string(),
                    diffusion_rate: 2.1,
                    molecular_weight: 32.0,
                },
                DiffusionData {
                    substance_name: "Glucose".to_string(),
                    diffusion_rate: 0.67,
                    molecular_weight: 180.0,
                },
                DiffusionData {
                    substance_name: "Toxin".to_string(),
                    diffusion_rate: 1.2,
                    molecular_weight: 150.0,
                },
            ],
            electrostatic_forces: true,
            van_der_waals_forces: true,
            surface_adhesion: 0.3,
        }
    }
}

// ===== EVOLUTION PROGRESSION =====
#[derive(Resource)]
pub struct EvolutionProgression {
    pub current_evolutionary_stage: u32,
    pub adaptation_points: u32,
    pub unlocked_evolutions: Vec<String>,
    pub environmental_pressures: Vec<EnvironmentalPressure>,
    pub mutation_rate: f32,
    pub natural_selection_factor: f32,
}

#[derive(Clone)]
pub struct EnvironmentalPressure {
    pub pressure_type: String,
    pub intensity: f32,
    pub duration: f32,
    pub adaptive_response: String,
}

impl Default for EvolutionProgression {
    fn default() -> Self {
        Self {
            current_evolutionary_stage: 1,
            adaptation_points: 0,
            unlocked_evolutions: vec!["CytoplasmicSpray".to_string()],
            environmental_pressures: Vec::new(),
            mutation_rate: 0.1,
            natural_selection_factor: 1.0,
        }
    }
}

// ===== BIOLOGICAL INTERACTIONS =====
#[derive(Resource)]
pub struct BiologicalInteractions {
    pub symbiotic_relationships: Vec<SymbioticPair>,
    pub predator_prey_chains: Vec<PredatorPreyRelation>,
    pub chemical_communications: Vec<ChemicalSignal>,
    pub competitive_exclusions: Vec<CompetitiveRelation>,
}

#[derive(Clone)]
pub struct SymbioticPair {
    pub organism_a: String,
    pub organism_b: String,
    pub benefit_type: String,
    pub strength: f32,
}

#[derive(Clone)]
pub struct PredatorPreyRelation {
    pub predator: String,
    pub prey: String,
    pub consumption_rate: f32,
    pub chase_behavior: String,
}

#[derive(Clone)]
pub struct ChemicalSignal {
    pub sender: String,
    pub receiver: String,
    pub signal_type: String,
    pub effect_strength: f32,
    pub transmission_range: f32,
}

#[derive(Clone)]
pub struct CompetitiveRelation {
    pub competitor_a: String,
    pub competitor_b: String,
    pub resource_competed: String,
    pub interference_strength: f32,
}

impl Default for BiologicalInteractions {
    fn default() -> Self {
        Self {
            symbiotic_relationships: vec![
                SymbioticPair {
                    organism_a: "Player".to_string(),
                    organism_b: "BeneficialBacteria".to_string(),
                    benefit_type: "MetabolicBoost".to_string(),
                    strength: 1.2,
                },
            ],
            predator_prey_chains: vec![
                PredatorPreyRelation {
                    predator: "ParasiticProtozoa".to_string(),
                    prey: "ViralParticle".to_string(),
                    consumption_rate: 0.8,
                    chase_behavior: "ActiveHunting".to_string(),
                },
            ],
            chemical_communications: vec![
                ChemicalSignal {
                    sender: "SwarmCell".to_string(),
                    receiver: "SwarmCell".to_string(),
                    signal_type: "Pheromone".to_string(),
                    effect_strength: 1.5,
                    transmission_range: 100.0,
                },
            ],
            competitive_exclusions: vec![
                CompetitiveRelation {
                    competitor_a: "AggressiveBacteria".to_string(),
                    competitor_b: "BeneficialBacteria".to_string(),
                    resource_competed: "Nutrients".to_string(),
                    interference_strength: 0.7,
                },
            ],
        }
    }
}

// ===== ADDITIONAL HELPER TYPES =====
#[derive(Clone)]
pub struct ChemicalAttractant {
    pub position: Vec2,
    pub attractant_type: String,
    pub strength: f32,
    pub decay_rate: f32,
}

#[derive(Clone)]
pub struct ColonySpawnPattern {
    pub pattern_name: String,
    pub spawn_positions: Vec<Vec2>,
    pub spawn_delays: Vec<f32>,
    pub organism_types: Vec<String>,
}

// ===== WEAPON SYSTEM RESOURCES =====
#[derive(Resource, Default)]
pub struct WeaponSystemConfig {
    pub main_cannon_enabled: bool,
    pub wing_cannons_enabled: bool,
    pub missile_system_enabled: bool,
    pub support_drones_enabled: bool,
    
    // Firing patterns
    pub main_cannon_pattern: MainCannonPattern,
    pub wing_cannon_sync: bool,
    pub missile_auto_target: bool,
}

#[derive(Default, Clone)]
pub enum MainCannonPattern {
    #[default]
    Single,     // Single projectile
    Double,     // Two projectiles side by side
    Triple,     // Three projectiles in line
    Spread,     // Fan pattern
    Focused,    // Tight cluster
}

// ===== CARD SYSTEM INTEGRATION =====
#[derive(Resource, Default)]
pub struct CardSystemState {
    // Card collection tracking
    pub permanent_cards_owned: HashSet<String>,
    pub temporal_cards_active: Vec<(String, f32)>, // (card_id, remaining_time)
    
    // Drop rate modifiers
    pub base_drop_rate: f32,
    pub drop_rate_multiplier: f32,
    
    // Card effects tracking
    pub active_effects: HashMap<String, f32>, // effect_name -> strength/remaining_time
    
    // Statistics
    pub total_cards_found: u32,
    pub cards_found_this_run: u32,
    pub green_boxes_collected: u32,
}

// ===== STAGE PROGRESS TRACKING =====
#[derive(Resource, Default)]
pub struct StageProgressTracker {
    pub current_stage: u32,
    pub current_wave_in_stage: u32,
    pub waves_per_stage: u32,
    
    // Objectives tracking
    pub enemies_killed_this_stage: u32,
    pub total_enemies_this_stage: u32,
    pub infrastructure_destroyed_this_stage: u32,
    pub total_infrastructure_this_stage: u32,
    pub damage_taken_this_stage: bool,
    pub perfect_atp_collection: bool,
    
    // Performance metrics
    pub stage_start_time: f32,
    pub stage_completion_time: f32,
    pub accuracy_this_stage: f32,
    
    // Bonus tracking
    pub combo_multiplier: f32,
    pub max_combo_this_stage: u32,
    pub current_combo: u32,
}

// ===== PERFORMANCE METRICS =====
#[derive(Resource, Default)]
pub struct PerformanceMetrics {
    pub entity_count: usize,
    pub particle_count: usize,
    pub projectile_count: usize,
    pub enemy_count: usize,
    
    pub frame_time_ms: f32,
    pub update_time_ms: f32,
    pub render_time_ms: f32,
    
    pub memory_usage_mb: f32,
    pub peak_entities_this_session: usize,
    
    // Optimization flags
    pub particle_culling_active: bool,
    pub entity_pooling_active: bool,
    pub reduced_effects: bool,
}

// ===== AUDIO SYSTEM ENHANCEMENTS =====
#[derive(Clone, Resource, Default)]
pub struct EnhancedAudioSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub ambient_volume: f32,
    
    // Dynamic audio
    pub adaptive_audio: bool,
    pub combat_intensity_scaling: bool,
    pub environmental_audio: bool,
    
    // Audio pools for performance
    pub max_concurrent_sfx: usize,
    pub audio_fade_time: f32,
}

// ===== DIFFICULTY SCALING =====
#[derive(Resource)]
pub struct DifficultyScaling {
    pub base_difficulty: f32,
    pub current_difficulty: f32,
    pub difficulty_curve: DifficultyProfile,
    
    // Scaling factors
    pub enemy_health_multiplier: f32,
    pub enemy_speed_multiplier: f32,
    pub enemy_damage_multiplier: f32,
    pub spawn_rate_multiplier: f32,
    
    // Player compensation
    pub atp_reward_multiplier: f32,
    pub card_drop_bonus: f32,
    pub score_multiplier: f32,
    
    // Adaptive difficulty
    pub player_skill_rating: f32,
    pub recent_performance: Vec<f32>,
    pub auto_adjust: bool,
}

#[derive(Clone)]
pub enum DifficultyProfile {
    Linear { rate: f32 },
    Exponential { base: f32, exponent: f32 },
    Stepped { thresholds: Vec<(u32, f32)> },
    Adaptive { target_success_rate: f32 },
}

impl Default for DifficultyScaling {
    fn default() -> Self {
        Self {
            base_difficulty: 1.0,
            current_difficulty: 1.0,
            difficulty_curve: DifficultyProfile::Linear { rate: 0.1 },
            enemy_health_multiplier: 1.0,
            enemy_speed_multiplier: 1.0,
            enemy_damage_multiplier: 1.0,
            spawn_rate_multiplier: 1.0,
            atp_reward_multiplier: 1.0,
            card_drop_bonus: 1.0,
            score_multiplier: 1.0,
            player_skill_rating: 0.5,
            recent_performance: Vec::new(),
            auto_adjust: true,
        }
    }
}

// ===== SAVE/LOAD SYSTEM =====
#[derive(Resource, Default)]
pub struct SaveGameData {
    pub player_stats: PlayerPersistentStats,
    pub unlocked_content: UnlockedContent,
    pub settings: GameSettings,
    pub achievements: Vec<String>,
    pub high_scores: Vec<HighScoreEntry>,
}

#[derive(Default, Clone)]
pub struct PlayerPersistentStats {
    pub total_games_played: u32,
    pub total_play_time_hours: f32,
    pub total_score: u64,
    pub total_enemies_defeated: u32,
    pub total_atp_collected: u64,
    pub total_cards_found: u32,
    pub highest_stage_reached: u32,
    pub perfect_stages_completed: u32,
    pub favorite_evolution_type: String,
    pub preferred_difficulty: f32,
}

#[derive(Default, Clone)]
pub struct UnlockedContent {
    pub evolution_types: Vec<String>,
    pub card_types: Vec<String>,
    pub difficulty_levels: Vec<String>,
    pub achievements_unlocked: Vec<String>,
    pub cosmetic_unlocks: Vec<String>,
}


#[derive(Default, Clone)]
pub struct GameSettings {
    pub audio: EnhancedAudioSettings,
    pub graphics: GraphicsSettings,
    pub controls: ControlSettings,
    pub gameplay: GameplaySettings,
}

#[derive(Default, Clone)]
pub struct GraphicsSettings {
    pub fullscreen: bool,
    pub vsync: bool,
    pub particle_density: f32,
    pub lighting_quality: LightingQuality,
    pub post_processing: bool,
    pub screen_shake_intensity: f32,
}

#[derive(Default, Clone)]
pub enum LightingQuality {
    #[default]
    Medium,
    Low,
    High,
    Ultra,
}

#[derive(Clone)]
pub struct ControlSettings {
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub shoot: KeyCode,
    pub emergency_spore: KeyCode,
    pub pause: KeyCode,
    
    pub mouse_sensitivity: f32,
    pub controller_deadzone: f32,
    pub invert_y_axis: bool,
}

impl Default for ControlSettings {
    fn default() -> Self {
        Self {
            move_up: KeyCode::ArrowUp,
            move_down: KeyCode::ArrowDown,
            move_left: KeyCode::ArrowLeft,
            move_right: KeyCode::ArrowRight,
            shoot: KeyCode::Space,
            emergency_spore: KeyCode::KeyX,
            pause: KeyCode::Escape,
            mouse_sensitivity: 1.0,
            controller_deadzone: 0.1,
            invert_y_axis: false,
        }
    }
}

#[derive(Default, Clone)]
pub struct GameplaySettings {
    pub auto_fire: bool,
    pub show_damage_numbers: bool,
    pub show_fps: bool,
    pub colorblind_support: bool,
    pub tutorial_hints: bool,
    pub pause_on_focus_loss: bool,
}

// ===== RESOURCE INITIALIZATION =====
pub fn setup_enhanced_resources(mut commands: Commands) {
    commands.insert_resource(CardSystemState::default());
    commands.insert_resource(StageProgressTracker::default());
    commands.insert_resource(WeaponSystemConfig::default());
    commands.insert_resource(DifficultyScaling::default());
    commands.insert_resource(PerformanceMetrics::default());
    commands.insert_resource(SaveGameData::default());
    commands.insert_resource(PauseMenuState::default());
    
    // Initialize enhanced scoring
    commands.insert_resource(GameScore::default());
    
    // Initialize stage summary
    commands.insert_resource(StageSummaryData::default());
}


impl StageProgressTracker {
    pub fn reset_for_new_stage(&mut self, stage_number: u32) {
        self.current_stage = stage_number;
        self.current_wave_in_stage = 0;
        self.enemies_killed_this_stage = 0;
        self.total_enemies_this_stage = 0;
        self.infrastructure_destroyed_this_stage = 0;
        self.total_infrastructure_this_stage = 0;
        self.damage_taken_this_stage = false;
        self.perfect_atp_collection = true;
        self.stage_start_time = 0.0; // Set by time system
        self.accuracy_this_stage = 1.0;
        self.combo_multiplier = 1.0;
        self.max_combo_this_stage = 0;
        self.current_combo = 0;
    }
    
    pub fn get_completion_percentage(&self) -> f32 {
        if self.total_enemies_this_stage > 0 {
            self.enemies_killed_this_stage as f32 / self.total_enemies_this_stage as f32
        } else {
            0.0
        }
    }
    
    pub fn get_objectives_completed(&self) -> u32 {
        let mut completed = 0;
        
        // 70% enemies destroyed
        if self.get_completion_percentage() >= 0.7 { completed += 1; }
        
        // 100% enemies destroyed
        if self.get_completion_percentage() >= 1.0 { completed += 1; }
        
        // All infrastructure destroyed
        if self.infrastructure_destroyed_this_stage >= self.total_infrastructure_this_stage && self.total_infrastructure_this_stage > 0 {
            completed += 1;
        }
        
        // No damage taken
        if !self.damage_taken_this_stage { completed += 1; }
        
        completed
    }
}

