use bevy::prelude::*;
use crate::enemy_types::*;

#[derive(Component)]
pub struct AlreadyDespawned;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub roll_factor: f32,
    pub lives: i32,
    pub invincible_timer: f32,
    pub cell_membrane_thickness: f32, // New biological property
}

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub damage: i32,
    pub friendly: bool,
    pub organic_trail: bool, // New: leaves bioluminescent trail
}

#[derive(Component)]
pub struct Collider {
    pub radius: f32,
}

#[derive(Component)]
pub struct Health(pub i32);



// Enhanced Particle System with Biological Properties
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
    pub fade_rate: f32,
    pub bioluminescent: bool, // New: glowing particles
    pub drift_pattern: DriftPattern, // New: organic motion
}

#[derive(Component)]
pub struct ParticleEmitter {
    pub spawn_rate: f32,
    pub spawn_timer: f32,
    pub particle_config: ParticleConfig,
    pub active: bool,
}

#[derive(Clone)]
pub struct ParticleConfig {
    pub color_start: Color,
    pub color_end: Color,
    pub velocity_range: (Vec2, Vec2),
    pub lifetime_range: (f32, f32),
    pub size_range: (f32, f32),
    pub gravity: Vec2,
    pub organic_motion: bool, // New: enables biological motion patterns
    pub bioluminescence: f32, // New: glow intensity
}

// New: Organic motion patterns
#[derive(Clone, Copy)]
pub enum DriftPattern {
    Floating,   // Gentle up-down motion like plankton
    Pulsing,    // Jellyfish-like contraction
    Spiraling,  // DNA-like helix motion
    Brownian,   // Random molecular motion
}

#[derive(Component)]
pub struct EngineTrail;

// New: Fluid Dynamics System
#[derive(Component)]
pub struct FluidDynamics {
    pub velocity: Vec2,
    pub viscosity_resistance: f32,
    pub buoyancy: f32,
    pub current_influence: f32, // How much currents affect this entity
}

// New: Chemical Sensitivity
#[derive(Component)]
pub struct ChemicalSensitivity {
    pub ph_tolerance_min: f32,
    pub ph_tolerance_max: f32,
    pub oxygen_requirement: f32,
    pub damage_per_second_outside_range: i32,
}

// New: Environmental Zones
#[derive(Component)]
pub struct EnvironmentalZone {
    pub zone_type: ZoneType,
    pub intensity: f32,
    pub effect_radius: f32,
}

#[derive(Clone)]
pub enum ZoneType {
    Acidic,      // High damage to calcium-based enemies, damages player over time
    Alkaline,    // Boosts photosynthetic abilities, neutralizes toxins
    Hypoxic,     // Slower movement, reduced energy regeneration
    Thermal,     // Increased metabolic rate, faster but more vulnerable
    Nutrient,    // Healing zone, spawns more power-ups
    Toxic,       // Constant damage, but weakens enemies too
    Current,     // Directional force affecting movement
}

#[derive(Component)]
pub struct ParallaxLayer {
    pub speed: f32,
    pub depth: f32,
}

// UI Components
#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct HealthNumericText;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HighScoreText;

#[derive(Component)]
pub struct MultiplierText;

#[derive(Component)]
pub struct PauseOverlay;

// Updated Power-ups for Biological Theme
#[derive(Component)]
pub struct PowerUp {
    pub power_type: PowerUpType,
    pub bob_timer: f32,
    pub bioluminescent_pulse: f32, // New: organic glow effect
}

#[derive(Clone)]
pub enum PowerUpType {
    // Updated names for biological theme
    CellularRegeneration { amount: i32 },
    CellWall { duration: f32 },
    Flagella { multiplier: f32, duration: f32 },
    SymbioticBoost { multiplier: f32, duration: f32 },
    MitochondriaOvercharge { rate_multiplier: f32, duration: f32 },

    // New biological power-ups
    Photosynthesis { energy_regen: f32, duration: f32 },
    Chemotaxis { homing_strength: f32, duration: f32 },
    Osmoregulation { immunity_duration: f32 },
    BinaryFission { clone_duration: f32 },
}

// Active Power-up Components (updated names)
#[derive(Component)]
pub struct CellWallReinforcement {
    pub timer: f32,
    pub alpha_timer: f32,
}

#[derive(Component)]
pub struct CellWallVisual;

#[derive(Component)]
pub struct FlagellaBoost {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct SymbioticMultiplier {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct MitochondriaOvercharge {
    pub timer: f32,
    pub rate_multiplier: f32,
}

// New biological effects
#[derive(Component)]
pub struct PhotosynthesisActive {
    pub timer: f32,
    pub energy_per_second: f32,
}

#[derive(Component)]
pub struct ChemotaxisActive {
    pub timer: f32,
    pub homing_strength: f32,
}

#[derive(Component)]
pub struct OsmoregulationActive {
    pub timer: f32,
}

#[derive(Component)]
pub struct BinaryFissionActive {
    pub timer: f32,
    pub clone_timer: f32,
}

#[derive(Component)]
pub struct GameOverUI;

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct FinalScoreText;

#[derive(Component)]
pub struct GameOverText;

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            color_start: Color::srgb(0.4, 0.8, 1.0), // Default to bioluminescent blue-green
            color_end: Color::srgba(0.2, 1.0, 0.8, 0.0),
            velocity_range: (Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0)),
            lifetime_range: (0.5, 1.5),
            size_range: (0.2, 0.6),
            gravity: Vec2::new(0.0, -20.0), // Reduced gravity for underwater feel
            organic_motion: true,
            bioluminescence: 0.8,
        }
    }
}

// Updated Weapon System for Biological Theme
#[derive(Component, Clone)]
pub struct EvolutionSystem {
    pub primary_evolution: EvolutionType,
    pub secondary_evolution: Option<EvolutionType>,
    pub cellular_adaptations: CellularAdaptations,
    pub emergency_spores: u32,
}

#[derive(Clone)]
pub enum EvolutionType {
    CytoplasmicSpray { damage: i32, fire_rate: f32 },
    PseudopodNetwork { damage: i32, fire_rate: f32, tendril_count: u32, spread_angle: f32 },
    BioluminescentBeam { damage: i32, charge_time: f32, duration: f32, width: f32 },
    SymbioticHunters { damage: i32, fire_rate: f32, homing_strength: f32, blast_radius: f32 },
    EnzymeBurst { damage: i32, fire_rate: f32, acid_damage: f32 },
    ToxinCloud { damage_per_second: i32, cloud_radius: f32, duration: f32 },
    ElectricDischarge { damage: i32, chain_count: u32, range: f32 },
}

#[derive(Clone)]
pub struct CellularAdaptations {
    pub membrane_permeability: f32,
    pub metabolic_efficiency: f32,
    pub chemoreceptor_sensitivity: f32,
    pub biofilm_formation: bool,
    pub extremophile_traits: bool,
}

#[derive(Component)]
pub struct LaserBeam {
    pub timer: f32,
    pub max_duration: f32,
    pub damage_per_second: i32,
    pub width: f32,
    pub length: f32,
    pub bioluminescent: bool, // New: organic beam effects
}

#[derive(Component)]
pub struct MissileProjectile {
    pub target: Option<Entity>,
    pub homing_strength: f32,
    pub blast_radius: f32,
    pub seek_timer: f32,
    pub symbiotic: bool, // New: biological homing behavior
}

#[derive(Component)]
pub struct ExplosiveProjectile {
    pub blast_radius: f32,
    pub blast_damage: i32,
    pub organic_explosion: bool, // New: biological explosion effects
}

#[derive(Component)]
pub struct ArmorPiercing {
    pub pierce_count: u32,
    pub max_pierce: u32,
    pub enzyme_based: bool, // New: dissolves through enemies rather than piercing
}

// Currency renamed to ATP (biological energy)
#[derive(Component)]
pub struct ATP {
    pub amount: u32,
}

// Evolution Chamber (renamed from UpgradeStation)
#[derive(Component)]
pub struct EvolutionChamber;

#[derive(Component)]
pub struct CellularUpgrades {
    pub max_health: i32,
    pub movement_efficiency: f32,
    pub damage_amplification: f32,
    pub metabolic_rate: f32,
    pub spore_capacity: u32,
}

// Emergency Spore System (renamed from Smart Bomb)
#[derive(Component)]
pub struct EmergencySpore {
    pub blast_timer: f32,
    pub max_time: f32,
    pub damage: i32,
    pub radius: f32,
}

#[derive(Component)]
pub struct SporeWave {
    pub timer: f32,
    pub max_time: f32,
    pub current_radius: f32,
    pub max_radius: f32,
    pub damage: i32,
}

// Formation AI Enhancement (with biological terminology)
#[derive(Component)]
pub struct ColonyCommander {
    pub colony_id: u32,
    pub members: Vec<Entity>,
    pub coordination_pattern: CoordinationPattern,
    pub chemical_timer: f32, // Changed from coordination_timer
}

#[derive(Clone)]
pub enum CoordinationPattern {
    ChemicalSignaling { interval: f32 },
    SwarmBehavior { swarm_size: u32, swarm_delay: f32 },
    BiofilmFormation { member_count: u32, rotation_speed: f32 },
    PheromoneTrail { target_focus: bool },
}

#[derive(Component)]
pub struct ColonyMember {
    pub colony_id: u32,
    pub role: ColonyRole,
    pub last_signal_time: f32,
}

#[derive(Clone)]
pub enum ColonyRole {
    Queen,      // Leader
    Worker,     // Attacker
    Guardian,   // Defender
    Symbiont,   // Support
}

// Biological Power-up Components
#[derive(Component)]
pub struct EvolutionPowerUp {
    pub evolution_type: EvolutionType,
    pub adaptation_type: AdaptationType,
    pub temporary: bool,
    pub duration: Option<f32>,
}

#[derive(Clone)]
pub enum AdaptationType {
    MetabolicBoost(f32),
    CellularDivisionRate(f32),
    EnzymeProduction,
    Bioluminescence,
    ChemicalResistance,
    EvolutionSwap(EvolutionType),
}

impl Default for EvolutionSystem {
    fn default() -> Self {
        Self {
            primary_evolution: EvolutionType::CytoplasmicSpray { damage: 10, fire_rate: 0.1 },
            secondary_evolution: None,
            cellular_adaptations: CellularAdaptations::default(),
            emergency_spores: 3,
        }
    }
}

impl Default for CellularAdaptations {
    fn default() -> Self {
        Self {
            membrane_permeability: 1.0,
            metabolic_efficiency: 1.0,
            chemoreceptor_sensitivity: 1.0,
            biofilm_formation: false,
            extremophile_traits: false,
        }
    }
}

impl Default for CellularUpgrades {
    fn default() -> Self {
        Self {
            max_health: 100,
            movement_efficiency: 1.0,
            damage_amplification: 1.0,
            metabolic_rate: 1.0,
            spore_capacity: 3,
        }
    }
}

impl EvolutionType {
    pub fn get_base_damage(&self) -> i32 {
        match self {
            EvolutionType::CytoplasmicSpray { damage, .. } => *damage,
            EvolutionType::PseudopodNetwork { damage, .. } => *damage,
            EvolutionType::BioluminescentBeam { damage, .. } => *damage,
            EvolutionType::SymbioticHunters { damage, .. } => *damage,
            EvolutionType::EnzymeBurst { damage, .. } => *damage,
            EvolutionType::ToxinCloud { damage_per_second, .. } => *damage_per_second,
            EvolutionType::ElectricDischarge { damage, .. } => *damage,
        }
    }

    pub fn get_fire_rate(&self) -> f32 {
        match self {
            EvolutionType::CytoplasmicSpray { fire_rate, .. } => *fire_rate,
            EvolutionType::PseudopodNetwork { fire_rate, .. } => *fire_rate,
            EvolutionType::BioluminescentBeam { charge_time, .. } => *charge_time,
            EvolutionType::SymbioticHunters { fire_rate, .. } => *fire_rate,
            EvolutionType::EnzymeBurst { fire_rate, .. } => *fire_rate,
            EvolutionType::ToxinCloud { duration, .. } => *duration,
            EvolutionType::ElectricDischarge { .. } => 1.5,
        }
    }

    pub fn get_display_name(&self) -> &'static str {
        match self {
            EvolutionType::CytoplasmicSpray { .. } => "Cytoplasmic Spray",
            EvolutionType::PseudopodNetwork { .. } => "Pseudopod Network",
            EvolutionType::BioluminescentBeam { .. } => "Bioluminescent Beam",
            EvolutionType::SymbioticHunters { .. } => "Symbiotic Hunters",
            EvolutionType::EnzymeBurst { .. } => "Enzyme Burst",
            EvolutionType::ToxinCloud { .. } => "Toxin Cloud",
            EvolutionType::ElectricDischarge { .. } => "Electric Discharge",
        }
    }
}

impl CoordinationPattern {
    pub fn execute(&self, chemical_timer: f32) -> bool {
        match self {
            CoordinationPattern::ChemicalSignaling { interval } => {
                chemical_timer % interval < 0.1
            }
            CoordinationPattern::SwarmBehavior { swarm_delay, .. } => {
                chemical_timer % swarm_delay < 0.1
            }
            CoordinationPattern::BiofilmFormation { .. } => {
                chemical_timer % 2.0 < 0.1
            }
            CoordinationPattern::PheromoneTrail { .. } => {
                chemical_timer % 1.5 < 0.1
            }
        }
    }
}

// Additional UI Components for biological theme
#[derive(Component)]
pub struct EvolutionUI;

#[derive(Component)]
pub struct ATPText;

#[derive(Component)]
pub struct EvolutionText;

#[derive(Component)]
pub struct SporeText;

#[derive(Component)]
pub struct ControlsText;

// Temporary evolution effect components
#[derive(Component)]
pub struct TemporaryMetabolicBoost {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct TemporaryCellularDivision {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct TemporaryEnzymeProduction {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryBioluminescence {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryChemicalResistance {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryEvolutionSwap {
    pub timer: f32,
    pub original_evolution: EvolutionType,
}

// New: Bioluminescent particles for organic effects
#[derive(Component)]
pub struct BioluminescentParticle {
    pub base_color: Color,
    pub pulse_frequency: f32,
    pub pulse_intensity: f32,
    pub organic_motion: OrganicMotion,
}

#[derive(Component)]
pub struct OrganicMotion {
    pub undulation_speed: f32,
    pub response_to_current: f32,
}


// New system: Chemical environment effects
#[derive(Component)]
pub struct ChemicalZone {
    pub ph_level: f32,
    pub oxygen_level: f32,
    pub toxicity: f32,
    pub temperature: f32,
}

// New: Current field for fluid dynamics
#[derive(Component)]
pub struct CurrentField {
    pub direction: Vec2,
    pub strength: f32,
    pub turbulence: f32,
}

// floating combat text
#[derive(Component)]
pub struct DamageText {
    pub timer: f32,
    pub velocity: Vec2,
}

// critical hits
#[derive(Component)]
pub struct CriticalHitStats {
    pub chance: f32,
    pub damage_multiplier: f32,
}

impl Default for CriticalHitStats {
    fn default() -> Self {
        Self {
            chance: 0.1, // 10% base crit chance
            damage_multiplier: 2.0, // 100% bonus damage
        }
    }
}

// Advanced Particle Systems
#[derive(Component)]
pub struct PheromoneParticle {
    pub signal_type: PheromoneType,
    pub strength: f32,
    pub decay_rate: f32,
}

#[derive(Clone)]
pub enum PheromoneType {
    Coordination,
    Alarm,
    Trail,
    Aggregation,
}

#[derive(Component)]
pub struct MembranePhysics {
    pub tension: f32,
    pub permeability: f32,
    pub elasticity: f32,
}

// see thermal_vent_effects_system
#[derive(Component)]
pub struct ThermalParticle {
    pub heat_intensity: f32,
    pub rise_speed: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

// better damage


#[derive(Component)]
pub struct ScreenShake {
    pub trauma: f32,
    pub offset: Vec2,
}

// powerup
// Extra Life powerup with enhanced effects
#[derive(Component)]
pub struct ExtraLifePowerUp {
    pub collected: bool,
    pub pulse_timer: f32,
}


#[derive(Component)]
pub struct CellWallTimerText;

// Add this component to link lights to explosions
#[derive(Component)]
pub struct LinkedExplosionLight(pub Entity);


// lighting test
#[derive(Component)]
pub struct ExplosionLight {
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
    pub timer: f32,
    pub max_time: f32,
    pub falloff: f32,
}

#[derive(Component, Clone)]
pub struct Explosion {
    pub timer: f32,
    pub max_time: f32,
    pub intensity: f32,
    pub explosion_type: ExplosionType,
    pub layers: Vec<ExplosionLayer>,
    pub current_layer_index: usize,
}

// Keep ExplosionLayer and ExplosionPhase as they provide the variety
#[derive(Component, Clone)]
pub struct ExplosionLayer {
    pub phase: ExplosionPhase,
    pub delay: f32,
    pub duration: f32,
    pub particle_count: u32,
    pub color_start: Color,
    pub color_end: Color,
    pub size_range: (f32, f32),
    pub velocity_range: (Vec2, Vec2),
    pub completed: bool,
}

#[derive(Clone)]
pub enum ExplosionPhase {
    Shockwave,
    CoreBlast,
    Debris,
    Afterglow,
    Membrane,
    MiniBlast,  // Incorporates old MiniExplosion functionality
}

#[derive(Clone, Copy)]
pub enum ExplosionType {
    Standard,
    Biological { toxin_release: bool, membrane_rupture: bool },
    Chemical { ph_change: f32, oxygen_release: f32 },
    Electrical { arc_count: u32, chain_range: f32 },
    Thermal { heat_wave: bool, temperature: f32 },
}

impl From<&Enemy> for ExplosionType {
    fn from(enemy: &Enemy) -> Self {
        match enemy.enemy_type {
            EnemyType::InfectedMacrophage => ExplosionType::Biological {
                toxin_release: true,
                membrane_rupture: true
            },
            EnemyType::BiofilmColony => ExplosionType::Chemical {
                ph_change: -1.2,
                oxygen_release: 0.4
            },
            EnemyType::AggressiveBacteria => ExplosionType::Biological {
                toxin_release: enemy.chemical_signature.releases_toxins,
                membrane_rupture: false
            },
            EnemyType::ParasiticProtozoa => ExplosionType::Biological {
                toxin_release: false,
                membrane_rupture: true
            },
            _ => ExplosionType::Standard,
        }
    }
}


#[derive(Component)]
pub struct FlashEffect {
    pub timer: f32,
    pub duration: f32,
    pub original_color: Color,
    pub flash_color: Color,
}



#[derive(Component)]
pub struct LightGlowSprite;

// TIDAL SYSTEM

#[derive(Component)]
pub struct TidalDebris {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub spin_speed: f32,
}

#[derive(Component)]
pub struct ThermalVentActivation {
    pub timer: f32,
    pub max_time: f32,
    pub pulse_frequency: f32,
}

#[derive(Component)]
pub struct CurrentIndicator {
    pub timer: f32,
    pub max_time: f32,
    pub direction: Vec2,
}

#[derive(Clone, Copy, Default)]
pub enum TidePhase {
    #[default]
    Rising,
    HighTide,
    Receding,
    LowTide,
}

// Tidal UI
#[derive(Component)]
pub struct TidalStatusText;

// Animation Components (Unique Enemies)

#[derive(Component)]
pub struct PulsingAnimation {
    pub frequency: f32,
    pub intensity: f32,
}

#[derive(Component)]
pub struct FlagellaAnimation {
    pub undulation_speed: f32,
    pub amplitude: f32,
}

#[derive(Component)]
pub struct PseudopodAnimation {
    pub extension_speed: f32,
    pub max_extension: f32,
}

#[derive(Component)]
pub struct CorruptionEffect {
    pub intensity: f32,
    pub color_shift_speed: f32,
}

#[derive(Component)]
pub struct WarningFlash {
    pub flash_frequency: f32,
    pub warning_color: Color,
}

#[derive(Component)]
pub struct ToxicAura {
    pub radius: f32,
    pub pulse_speed: f32,
}

#[derive(Component)]
pub struct CoordinationIndicator {
    pub signal_strength: f32,
    pub communication_range: f32,
}

#[derive(Component)]
pub struct GestationAnimation {
    pub pulse_frequency: f32,
    pub growth_factor: f32,
}

#[derive(Component)]
pub struct JuvenileWiggle {
    pub wiggle_speed: f32,
    pub amplitude: f32,
}


// Improved Enemy AI
#[derive(Component, Clone)]
pub struct PredatorPreyBehavior {
    pub predator_types: Vec<EnemyType>,
    pub prey_types: Vec<EnemyType>,
    pub hunt_range: f32,
    pub flee_range: f32,
    pub hunting_speed_bonus: f32,
    pub fear_intensity: f32,
}

#[derive(Component)]
pub struct ChemicalTrail {
    pub trail_type: ChemicalTrailType,
    pub strength: f32,
    pub decay_rate: f32,
    pub creation_timer: f32,
}

#[derive(Clone)]
pub enum ChemicalTrailType {
    PlayerPheromone,
    BloodTrail,
    ToxinTrail,
    FoodTrail,
    AlarmPheromone,
}

#[derive(Component)]
pub struct EcosystemRole {
    pub role: EcosystemRoleType,
    pub influence_radius: f32,
    pub balance_factor: f32,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum EcosystemRoleType {
    Apex,          // Top predator, low spawn rate
    Primary,       // Main threat, normal spawn rate
    Secondary,     // Support enemy, higher spawn rate
    Decomposer,    // Cleanup crew, spawn after deaths
    Symbiont,      // Beneficial when not hostile
}

#[derive(Component)]
pub struct AdaptiveDifficulty {
    pub threat_level: f32,
    pub adaptation_rate: f32,
    pub player_evolution_response: f32,
}


// Environmental storytelling components
#[derive(Component)]
pub struct EnhancedCoral {
    pub coral_type: CoralType,
    pub health: f32,
    pub corruption_level: f32,
    pub spread_rate: f32,
    pub bioluminescent_warning: bool,
    pub original_color: Color,
    pub size: Vec2,
    pub gameplay_effect: CoralEffect,
    pub influence_radius: f32,
    pub last_spawn_time: f32,
}

#[derive(Clone)]
pub enum CoralType {
    // Beneficial corals
    FilterFeeder {        // Cleans chemical contamination
        purification_rate: f32,
        ph_stabilization: f32,
    },
    OxygenProducer {      // Increases local oxygen levels
        oxygen_output: f32,
        photosynthesis_rate: f32,
    },
    SymbioticReef {       // Provides healing and ATP
        healing_rate: f32,
        atp_generation: f32,
    },
    
    // Neutral/environmental corals
    BarrierReef {         // Physical obstacles that provide cover
        structural_integrity: f32,
        provides_cover: bool,
    },
    BioluminescentBeacon { // Navigation aids and early warning
        pulse_frequency: f32,
        detection_range: f32,
    },
    
    // Corrupted/hostile corals
    CorruptedColony {     // Spawns toxins and hostile microbes
        toxin_production: f32,
        spawn_hostiles: bool,
    },
    AcidicFormation {     // Lowers local pH, damages nearby entities
        acid_strength: f32,
        corrosion_rate: f32,
    },
    ParasiticGrowth {     // Drains health and ATP from nearby player
        drain_rate: f32,
        infection_chance: f32,
    },
}

#[derive(Clone)]
pub enum CoralEffect {
    Beneficial {
        healing_per_second: f32,
        atp_per_second: f32,
        ph_stabilization: f32,
        oxygen_boost: f32,
    },
    Neutral {
        provides_cover: bool,
        navigation_aid: bool,
    },
    Harmful {
        damage_per_second: f32,
        ph_reduction: f32,
        spawns_enemies: f32, // Spawn rate
        corruption_spread: f32,
    },
}



#[derive(Component)]
pub struct ContaminationCloud {
    pub toxicity_level: f32,
    pub expansion_rate: f32,
    pub source_type: ContaminationType,
    pub warning_intensity: f32,
}

#[derive(Clone)]
pub enum ContaminationType {
    IndustrialWaste,
    BiologicalToxin,
    RadioactiveSeepage,
    ChemicalSpill,
    PlasticPollution,
}

#[derive(Component)]
pub struct MicroscopicDebris {
    pub debris_type: DebrisType,
    pub story_fragment: String,
    pub age: f32,
    pub reveal_distance: f32,
}

#[derive(Clone)]
pub enum DebrisType {
    PlasticFragment { size: f32, color: Color },
    MetalParticle { oxidation_level: f32 },
    ChemicalResidue { compound_type: String },
    BiologicalRemains { species: String, decay_level: f32 },
    SyntheticFiber { material: String, weathering: f32 },
}

#[derive(Component)]
pub struct BioluminescentWarning {
    pub pattern_type: WarningPattern,
    pub intensity: f32,
    pub pulse_frequency: f32,
    pub danger_level: f32,
}

#[derive(Clone)]
pub enum WarningPattern {
    RadialPulse,      // Danger spreading outward
    DirectionalStrobe, // Directional hazard warning
    ColorShift,       // Chemical change indicator
    FlashingGrid,     // Systematic contamination
    ChaotticFlicker,  // Ecosystem breakdown
}


// USER INTERFACE

#[derive(Component)]
pub struct EnvironmentText;

#[derive(Component)]
pub struct EcosystemStatusText;

#[derive(Component)]
pub struct ContaminationWarningText;

#[derive(Component)]
pub struct PerfHudText;


// menu system
#[derive(Component)]
pub struct LoadingBar;

#[derive(Component)]
pub struct LoadingBarFill;

#[derive(Component)]
pub struct TitleScreen;

#[derive(Component)]
pub struct MenuButton { pub action: MenuAction }

#[derive(Component)]
pub struct BackgroundParticle { pub velocity: Vec2 }

#[derive(Component)]
pub struct ControlsScreen;

#[derive(Component)]
pub struct CopyrightText;

#[derive(Component)]
pub struct HighScoreDisplay;

#[derive(Clone)]
pub enum MenuAction { 
    Play, 
    Options, 
    Quit,
    Settings,
    HighScores,
    Back,
    ToggleFullscreen,
    ResetControls,
}

// ===== MENU SYSTEM COMPONENTS =====
#[derive(Component)]
pub struct PulsingText;

#[derive(Component)]
pub struct AnimatedParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub pulse_phase: f32,
}

#[derive(Component, Clone)]
pub enum SliderType {
    Master,
    SFX,
    Music,
}

#[derive(Component)]
pub struct AudioSlider {
    pub slider_type: SliderType,
}

#[derive(Component)]
pub struct SliderFill {
    pub slider_type: SliderType,
}

#[derive(Component)]
pub enum VolumeText {
    Master,
    SFX,
    Music,
}

#[derive(Component)]
pub struct SettingsMenu;

#[derive(Component)]
pub struct HighScoreMenu;