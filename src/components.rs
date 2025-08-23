use bevy::prelude::*;

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

#[derive(Component)]
pub struct Explosion {
    pub timer: f32,
    pub max_time: f32,
    pub intensity: f32,
    pub organic: bool, // New: organic explosions have different effects
}

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

#[derive(Component)]
pub struct Light2D {
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
}

// UI Components
#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarFill;

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
}

// better damage
#[derive(Component)]
pub struct FlashEffect {
    pub timer: f32,
    pub duration: f32,
    pub original_color: Color,
}

#[derive(Component)]
pub struct MiniExplosion {
    pub timer: f32,
    pub max_time: f32,
    pub size: f32,
}

#[derive(Component)]
pub struct ScreenShake {
    pub trauma: f32,
    pub offset: Vec2,
}

// powerup
#[derive(Component)]
pub struct ExtraLifePowerUp;

#[derive(Component)]
pub struct CellWallTimerText;

// lighting test
#[derive(Component)]
pub struct ExplosionLight {
    pub timer: f32,
    pub max_time: f32,
}
