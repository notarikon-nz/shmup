use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// FONT
#[derive(Resource)]
pub struct GameFonts {
    pub default_font: Handle<Font>,
}


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
    pub viral_particle_texture: Handle<Image>,        // Small, spiky virus
    pub aggressive_bacteria_texture: Handle<Image>,   // Rod-shaped with flagella
    pub parasitic_protozoa_texture: Handle<Image>,    // Amoeba-like with pseudopods
    pub infected_macrophage_texture: Handle<Image>,   // Large corrupted cell
    pub suicidal_spore_texture: Handle<Image>,        // Explosive spore with warning patterns
    pub biofilm_colony_texture: Handle<Image>,        // Clustered bacterial mat
    pub swarm_cell_texture: Handle<Image>,            // Coordinated cell with communication tendrils
    pub reproductive_vesicle_texture: Handle<Image>,  // Egg-like spawning structure
    pub offspring_texture: Handle<Image>,             // Small, newly spawned cell

    // PowerUp Textirse
    pub health_powerup_texture: Handle<Image>,
    pub shield_powerup_texture: Handle<Image>,
    pub speed_powerup_texture: Handle<Image>,
    pub multiplier_powerup_texture: Handle<Image>,
    pub rapidfire_powerup_texture: Handle<Image>,

    pub background_layers: Vec<Handle<Image>>,
    pub sfx_shoot: Handle<AudioSource>,
    pub sfx_explosion: Handle<AudioSource>,
    pub sfx_powerup: Handle<AudioSource>,
    pub music: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct ParticlePool {
    pub entities: Vec<Entity>,
    pub index: usize,
}

#[derive(Resource, Default)]
pub struct InputState {
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

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
}

#[derive(Resource, Default)]
pub struct GameStarted(pub bool);

#[derive(Resource, Default)]
pub struct ShootingState {
    pub rate_multiplier: f32,
    pub base_rate: f32,
}

// New: Fluid Environment for biological movement
#[derive(Resource)]
pub struct FluidEnvironment {
    pub current_field: Vec<Vec2>, // Grid-based flow field
    pub grid_size: usize,
    pub cell_size: f32,
    pub tidal_phase: f32,
    pub turbulence_intensity: f32,
}

impl Default for FluidEnvironment {
    fn default() -> Self {
        Self {
            current_field: vec![Vec2::ZERO; 64 * 64],
            grid_size: 64,
            cell_size: 20.0,
            tidal_phase: 0.0,
            turbulence_intensity: 0.3,
        }
    }
}

// New: Chemical Environment for pH and oxygen simulation
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

// New: Scale Management for seamless transitions
#[derive(Resource)]
pub struct ScaleManager {
    pub current_scale: f32, // 1.0 = normal, 0.1 = zoomed in 10x, 10.0 = zoomed out 10x
    pub target_scale: f32,
    pub transition_speed: f32,
    pub scale_levels: Vec<ScaleLevel>,
    pub physics_scale_factor: f32,
}

#[derive(Clone)]
pub struct ScaleLevel {
    pub scale: f32,
    pub name: String,
    pub physics_multiplier: f32, // Brownian motion more intense at smaller scales
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

// New: Current Field Generator for procedural flow
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

// New: Bioluminescence Manager for organic lighting
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


// New: Ecosystem State Manager
#[derive(Resource)]
pub struct EcosystemState {
    pub health: f32, // Overall ecosystem health (0.0-1.0)
    pub infection_level: f32, // How corrupted the environment is
    pub symbiotic_activity: f32, // Cooperative organism activity
    pub nutrient_density: f32, // Available resources
    pub ph_stability: f32, // How stable the chemical environment is
    pub oxygen_circulation: f32, // Oxygen flow health
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

// New: Tidal Pool Physics
#[derive(Resource)]
pub struct TidalPoolPhysics {
    pub tide_level: f32, // -1.0 (low tide) to 1.0 (high tide)
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


// New: Microscopic Scale Physics
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

// New: Biological Progression System
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

// New: Biological Interaction Matrix
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

// Screen Shake Effect
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

// (Persistent) High Scores
#[derive(Resource, Default)]
pub struct GameScore {
    pub current: u32,
    pub high_scores: Vec<u32>,
    pub score_multiplier: f32,
    pub multiplier_timer: f32,
    pub high_score_data: Option<HighScoreData>,
    pub total_atp_collected: u64,
    pub enemies_defeated: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScoreData {
    pub scores: Vec<HighScoreEntry>,
    pub total_games_played: u32,
    pub best_evolution_reached: String,
    pub longest_survival_time: f32,
    pub total_atp_collected: u64,
    pub enemies_defeated: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScoreEntry {
    pub score: u32,
    pub date: String,
    pub evolution_type: String,
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
                    waves_survived: 10,
                    time_played: 300.0,
                },
                HighScoreEntry {
                    score: 7500,
                    date: "2025-01-01".to_string(),
                    evolution_type: "Pseudopod Network".to_string(),
                    waves_survived: 8,
                    time_played: 240.0,
                },
                HighScoreEntry {
                    score: 5000,
                    date: "2025-01-01".to_string(),
                    evolution_type: "Bioluminescent Beam".to_string(),
                    waves_survived: 6,
                    time_played: 180.0,
                },
            ],
            total_games_played: 3,
            best_evolution_reached: "Bioluminescent Beam".to_string(),
            longest_survival_time: 300.0,
            total_atp_collected: 2500,
            enemies_defeated: 150,
        }
    }
}

use crate::components::{TidePhase};

// Tidal Mechanics
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