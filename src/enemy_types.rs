use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Enemy {
    pub ai_type: EnemyAI,
    pub health: i32,
    pub speed: f32,
    pub enemy_type: EnemyType,
    pub colony_id: Option<u32>, // Renamed from formation_id
    pub chemical_signature: ChemicalSignature, // New: biological properties
}

// New: Chemical properties affecting AI behavior
#[derive(Clone)]
pub struct ChemicalSignature {
    pub ph_preference: f32,
    pub oxygen_tolerance: f32,
    pub responds_to_pheromones: bool,
    pub releases_toxins: bool,
}

#[derive(Clone)]
pub enum EnemyType {
    // Updated names for biological theme
    ViralParticle,          // Basic
    AggressiveBacteria,     // Fast
    ParasiticProtozoa,      // Heavy
    InfectedMacrophage,     // Boss
    SuicidalSpore,          // Kamikaze
    BiofilmColony,          // Turret
    SwarmCell,              // FormationFighter
    ReproductiveVesicle,    // Spawner
    Offspring,              // SpawnerMinion
}

#[derive(Component, Clone)]
pub enum EnemyAI {
    Static,
    Linear { direction: Vec2 },
    Sine { amplitude: f32, frequency: f32, phase: f32 },
    MiniBoss { pattern: usize, timer: f32 },
    Kamikaze { target_pos: Vec2, dive_speed: f32, acquired_target: bool },
    Turret { rotation: f32, shoot_timer: f32, detection_range: f32 },
    Formation { 
        formation_id: u32, 
        position_in_formation: Vec2, 
        leader_offset: Vec2,
        formation_timer: f32,
    },
    Spawner { 
        spawn_timer: f32, 
        spawn_rate: f32, 
        minions_spawned: u32, 
        max_minions: u32 
    },
    
    // New biological AI behaviors
    Chemotaxis { // Follows chemical gradients
        target_chemical: ChemicalType,
        sensitivity: f32,
        current_direction: Vec2,
    },
    CellDivision { // Splits when damaged
        division_threshold: f32,
        division_timer: f32,
        has_divided: bool,
    },
    SymbioticPair { // Moves in pairs, dies when partner dies
        partner_entity: Option<Entity>,
        bond_distance: f32,
        sync_timer: f32,
    },
    FluidFlow { // Follows water currents more strongly
        flow_sensitivity: f32,
        base_direction: Vec2,
    },
}

#[derive(Clone)]
pub enum ChemicalType {
    PlayerPheromones,
    NutrientGradient,
    ToxinAvoidance,
    OxygenSeeker,
}

#[derive(Component)]
pub struct ColonyLeader {
    pub colony_id: u32,
    pub members: Vec<Entity>,
    pub pattern_timer: f32,
    pub pattern_type: ColonyPattern,
    pub chemical_communication: bool, // New: uses chemical signals
}

#[derive(Clone)]
pub enum ColonyPattern {
    BiofilmFormation,    // VFormation -> organic cluster
    LinearChain,         // LineFormation -> cell chain
    CircularCluster,     // CircleFormation -> spherical colony
    SymbioticPair,       // DiamondFormation -> paired organisms
}

#[derive(Component)]
pub struct ToxinProducer {
    pub parent_entity: Entity,
    pub toxin_cloud_radius: f32,
    pub production_rate: f32,
}

// Enemy stats configuration with biological properties
impl EnemyType {
    pub fn get_stats(&self) -> (i32, f32, f32, Color) {
        match self {
            EnemyType::ViralParticle => (20, 15.0, 150.0, Color::srgb(0.8, 0.9, 1.0)),
            EnemyType::AggressiveBacteria => (15, 12.0, 250.0, Color::srgb(0.9, 0.3, 0.3)),
            EnemyType::ParasiticProtozoa => (50, 20.0, 100.0, Color::srgb(0.6, 0.8, 0.3)),
            EnemyType::InfectedMacrophage => (100, 30.0, 120.0, Color::srgb(1.0, 0.2, 0.8)),
            EnemyType::SuicidalSpore => (10, 12.0, 200.0, Color::srgb(1.0, 0.7, 0.2)),
            EnemyType::BiofilmColony => (40, 25.0, 0.0, Color::srgb(0.4, 0.6, 0.4)),
            EnemyType::SwarmCell => (25, 14.0, 180.0, Color::srgb(0.5, 1.0, 0.5)),
            EnemyType::ReproductiveVesicle => (80, 22.0, 80.0, Color::srgb(0.8, 0.3, 0.8)),
            EnemyType::Offspring => (8, 8.0, 300.0, Color::srgb(0.6, 0.2, 0.6)),
        }
    }

    pub fn get_points(&self) -> u32 {
        match self {
            EnemyType::ViralParticle => 100,
            EnemyType::AggressiveBacteria => 150,
            EnemyType::ParasiticProtozoa => 200,
            EnemyType::InfectedMacrophage => 1000,
            EnemyType::SuicidalSpore => 120,
            EnemyType::BiofilmColony => 250,
            EnemyType::SwarmCell => 180,
            EnemyType::ReproductiveVesicle => 500,
            EnemyType::Offspring => 50,
        }
    }
    
    pub fn get_chemical_signature(&self) -> ChemicalSignature {
        match self {
            EnemyType::ViralParticle => ChemicalSignature {
                ph_preference: 7.0,
                oxygen_tolerance: 0.5,
                responds_to_pheromones: false,
                releases_toxins: false,
            },
            EnemyType::AggressiveBacteria => ChemicalSignature {
                ph_preference: 6.5,
                oxygen_tolerance: 0.8,
                responds_to_pheromones: true,
                releases_toxins: true,
            },
            EnemyType::ParasiticProtozoa => ChemicalSignature {
                ph_preference: 7.5,
                oxygen_tolerance: 0.3,
                responds_to_pheromones: false,
                releases_toxins: false,
            },
            EnemyType::InfectedMacrophage => ChemicalSignature {
                ph_preference: 7.2,
                oxygen_tolerance: 0.9,
                responds_to_pheromones: true,
                releases_toxins: true,
            },
            EnemyType::SuicidalSpore => ChemicalSignature {
                ph_preference: 8.0,
                oxygen_tolerance: 0.1,
                responds_to_pheromones: true,
                releases_toxins: false,
            },
            EnemyType::BiofilmColony => ChemicalSignature {
                ph_preference: 6.8,
                oxygen_tolerance: 0.4,
                responds_to_pheromones: true,
                releases_toxins: true,
            },
            EnemyType::SwarmCell => ChemicalSignature {
                ph_preference: 7.1,
                oxygen_tolerance: 0.7,
                responds_to_pheromones: true,
                releases_toxins: false,
            },
            EnemyType::ReproductiveVesicle => ChemicalSignature {
                ph_preference: 7.3,
                oxygen_tolerance: 0.6,
                responds_to_pheromones: false,
                releases_toxins: false,
            },
            EnemyType::Offspring => ChemicalSignature {
                ph_preference: 7.0,
                oxygen_tolerance: 0.5,
                responds_to_pheromones: true,
                releases_toxins: false,
            },
        }
    }
    
    pub fn get_biological_description(&self) -> &'static str {
        match self {
            EnemyType::ViralParticle => "Basic viral particle - simple infection vector",
            EnemyType::AggressiveBacteria => "Fast-moving pathogenic bacteria",
            EnemyType::ParasiticProtozoa => "Heavily armored single-celled parasite",
            EnemyType::InfectedMacrophage => "Corrupted immune cell, highly dangerous",
            EnemyType::SuicidalSpore => "Explosive reproductive cell",
            EnemyType::BiofilmColony => "Stationary bacterial cluster with toxin production",
            EnemyType::SwarmCell => "Coordinated colony member with hive behavior",
            EnemyType::ReproductiveVesicle => "Spawning organism that produces offspring",
            EnemyType::Offspring => "Newly spawned cell with basic mobility",
        }
    }
}

impl ColonyPattern {
    pub fn get_position(&self, index: usize, total: usize, timer: f32) -> Vec2 {
        match self {
            ColonyPattern::BiofilmFormation => {
                // Organic clustering pattern
                let layer = (index as f32 / 3.0) as usize;
                let angle_in_layer = (index % 3) as f32 * (std::f32::consts::TAU / 3.0);
                let radius = 30.0 + layer as f32 * 25.0;
                let organic_offset = (timer * 0.3 + index as f32).sin() * 5.0;
                Vec2::new(
                    angle_in_layer.cos() * (radius + organic_offset), 
                    angle_in_layer.sin() * (radius + organic_offset)
                )
            }
            ColonyPattern::LinearChain => {
                // Cell chain like bacteria
                let spacing = 40.0;
                let chain_undulation = (timer * 2.0 + index as f32 * 0.5).sin() * 10.0;
                Vec2::new(
                    (index as f32 - total as f32 / 2.0) * spacing,
                    chain_undulation
                )
            }
            ColonyPattern::CircularCluster => {
                // Spherical colony cross-section
                let angle = (index as f32 / total as f32) * std::f32::consts::TAU + timer * 0.5;
                let radius = 80.0 + (timer * 1.5).sin() * 10.0; // Breathing motion
                Vec2::new(angle.cos() * radius, angle.sin() * radius)
            }
            ColonyPattern::SymbioticPair => {
                // Paired organisms
                let pair_index = index / 2;
                let is_secondary = index % 2 == 1;
                let base_angle = pair_index as f32 * (std::f32::consts::TAU / (total / 2) as f32);
                let offset = if is_secondary { 30.0 } else { 0.0 };
                let sync_motion = (timer * 3.0).sin() * 5.0;
                Vec2::new(
                    base_angle.cos() * (50.0 + offset) + sync_motion,
                    base_angle.sin() * (50.0 + offset)
                )
            }
        }
    }
}

impl Default for ChemicalSignature {
    fn default() -> Self {
        Self {
            ph_preference: 7.0,
            oxygen_tolerance: 0.5,
            responds_to_pheromones: false,
            releases_toxins: false,
        }
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            ai_type: EnemyAI::Linear { direction: Vec2::new(0.0, -1.0) },
            health: 20,
            speed: 150.0,
            enemy_type: EnemyType::ViralParticle,
            colony_id: None,
            chemical_signature: ChemicalSignature::default(),
        }
    }
}

