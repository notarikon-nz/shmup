use bevy::prelude::*;
use crate::enemy_types::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub roll_factor: f32,
    pub lives: i32,
    pub invincible_timer: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub damage: i32,
    pub friendly: bool,
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
}

// Particle System Components
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
    pub fade_rate: f32,
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
}

#[derive(Component)]
pub struct EngineTrail;

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

#[derive(Component)]
pub struct PowerUp {
    pub power_type: PowerUpType,
    pub bob_timer: f32,
}

#[derive(Clone)]
pub enum PowerUpType {
    Health { amount: i32 },
    Shield { duration: f32 },
    Speed { multiplier: f32, duration: f32 },
    Multiplier { multiplier: f32, duration: f32 },
    RapidFire { rate_multiplier: f32, duration: f32 },
}

// Active Power-up Components
#[derive(Component)]
pub struct Shield {
    pub timer: f32,
    pub alpha_timer: f32,
}

#[derive(Component)]
pub struct ShieldVisual;

#[derive(Component)]
pub struct SpeedBoost {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct ScoreMultiplier {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct RapidFire {
    pub timer: f32,
    pub rate_multiplier: f32,
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
            color_start: Color::WHITE,
            color_end: Color::srgba(1.0, 1.0, 1.0, 0.0),
            velocity_range: (Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0)),
            lifetime_range: (0.5, 1.5),
            size_range: (2.0, 6.0),
            gravity: Vec2::new(0.0, -100.0),
        }
    }
}


// Weapon System Components
#[derive(Component, Clone)]
pub struct WeaponSystem {
    pub primary_weapon: WeaponType,
    pub secondary_weapon: Option<WeaponType>,
    pub weapon_upgrades: WeaponUpgrades,
    pub smart_bombs: u32,
}

#[derive(Clone)]
pub enum WeaponType {
    Basic { damage: i32, fire_rate: f32 },
    SpreadShot { damage: i32, fire_rate: f32, spread_count: u32, spread_angle: f32 },
    Laser { damage: i32, charge_time: f32, duration: f32, width: f32 },
    Missile { damage: i32, fire_rate: f32, homing_strength: f32, blast_radius: f32 },
    RapidFire { damage: i32, fire_rate: f32, heat_buildup: f32 },
}

#[derive(Clone)]
pub struct WeaponUpgrades {
    pub damage_multiplier: f32,
    pub fire_rate_multiplier: f32,
    pub armor_piercing: bool,
    pub explosive_rounds: bool,
    pub homing_enhancement: bool,
}

#[derive(Component)]
pub struct LaserBeam {
    pub timer: f32,
    pub max_duration: f32,
    pub damage_per_second: i32,
    pub width: f32,
    pub length: f32,
}

#[derive(Component)]
pub struct MissileProjectile {
    pub target: Option<Entity>,
    pub homing_strength: f32,
    pub blast_radius: f32,
    pub seek_timer: f32,
}

#[derive(Component)]
pub struct ExplosiveProjectile {
    pub blast_radius: f32,
    pub blast_damage: i32,
}

#[derive(Component)]
pub struct ArmorPiercing {
    pub pierce_count: u32,
    pub max_pierce: u32,
}

// Currency and Upgrade Components
#[derive(Component)]
pub struct Currency {
    pub amount: u32,
}

#[derive(Component)]
pub struct UpgradeStation;

#[derive(Component)]
pub struct PermanentUpgrades {
    pub max_health: i32,
    pub speed_boost: f32,
    pub damage_boost: f32,
    pub fire_rate_boost: f32,
    pub smart_bomb_capacity: u32,
}

// Smart Bomb Components
#[derive(Component)]
pub struct SmartBomb {
    pub blast_timer: f32,
    pub max_time: f32,
    pub damage: i32,
    pub radius: f32,
}

#[derive(Component)]
pub struct SmartBombWave {
    pub timer: f32,
    pub max_time: f32,
    pub current_radius: f32,
    pub max_radius: f32,
    pub damage: i32,
}

// Formation AI Enhancement
#[derive(Component)]
pub struct FormationCommander {
    pub formation_id: u32,
    pub members: Vec<Entity>,
    pub attack_pattern: AttackPattern,
    pub coordination_timer: f32,
}

#[derive(Clone)]
pub enum AttackPattern {
    SynchronizedShoot { interval: f32 },
    WaveAttack { wave_size: u32, wave_delay: f32 },
    CircularBarrage { projectile_count: u32, rotation_speed: f32 },
    FocusedAssault { target_focus: bool },
}

#[derive(Component)]
pub struct FormationMember {
    pub formation_id: u32,
    pub role: FormationRole,
    pub last_command_time: f32,
}

#[derive(Clone)]
pub enum FormationRole {
    Leader,
    Attacker,
    Defender,
    Support,
}

// Weapon Power-up Components
#[derive(Component)]
pub struct WeaponPowerUp {
    pub weapon_type: WeaponType,
    pub upgrade_type: WeaponUpgradeType,
    pub temporary: bool,
    pub duration: Option<f32>,
}

#[derive(Clone)]
pub enum WeaponUpgradeType {
    DamageBoost(f32),
    FireRateBoost(f32),
    ArmorPiercing,
    ExplosiveRounds,
    HomingUpgrade,
    WeaponSwap(WeaponType),
}

impl Default for WeaponSystem {
    fn default() -> Self {
        Self {
            primary_weapon: WeaponType::Basic { damage: 10, fire_rate: 0.1 },
            secondary_weapon: None,
            weapon_upgrades: WeaponUpgrades::default(),
            smart_bombs: 3,
        }
    }
}

impl Default for WeaponUpgrades {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            fire_rate_multiplier: 1.0,
            armor_piercing: false,
            explosive_rounds: false,
            homing_enhancement: false,
        }
    }
}

impl Default for PermanentUpgrades {
    fn default() -> Self {
        Self {
            max_health: 100,
            speed_boost: 1.0,
            damage_boost: 1.0,
            fire_rate_boost: 1.0,
            smart_bomb_capacity: 3,
        }
    }
}

impl WeaponType {
    pub fn get_base_damage(&self) -> i32 {
        match self {
            WeaponType::Basic { damage, .. } => *damage,
            WeaponType::SpreadShot { damage, .. } => *damage,
            WeaponType::Laser { damage, .. } => *damage,
            WeaponType::Missile { damage, .. } => *damage,
            WeaponType::RapidFire { damage, .. } => *damage,
        }
    }
    
    pub fn get_fire_rate(&self) -> f32 {
        match self {
            WeaponType::Basic { fire_rate, .. } => *fire_rate,
            WeaponType::SpreadShot { fire_rate, .. } => *fire_rate,
            WeaponType::Laser { charge_time, .. } => *charge_time,
            WeaponType::Missile { fire_rate, .. } => *fire_rate,
            WeaponType::RapidFire { fire_rate, .. } => *fire_rate,
        }
    }
}

impl AttackPattern {
    pub fn execute(&self, formation_timer: f32) -> bool {
        match self {
            AttackPattern::SynchronizedShoot { interval } => {
                formation_timer % interval < 0.1
            }
            AttackPattern::WaveAttack { wave_delay, .. } => {
                formation_timer % wave_delay < 0.1
            }
            AttackPattern::CircularBarrage { .. } => {
                formation_timer % 2.0 < 0.1
            }
            AttackPattern::FocusedAssault { .. } => {
                formation_timer % 1.5 < 0.1
            }
        }
    }
}

// Additional UI Components for enhanced features
#[derive(Component)]
pub struct UpgradeUI;

#[derive(Component)]
pub struct CurrencyText;

#[derive(Component)]
pub struct WeaponText;

#[derive(Component)]
pub struct SmartBombText;

#[derive(Component)]
pub struct ControlsText;

// Temporary weapon effect components for power-ups
#[derive(Component)]
pub struct TemporaryDamageBoost {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct TemporaryFireRateBoost {
    pub timer: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct TemporaryArmorPiercing {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryExplosiveRounds {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryHomingUpgrade {
    pub timer: f32,
}

#[derive(Component)]
pub struct TemporaryWeaponSwap {
    pub timer: f32,
    pub original_weapon: WeaponType,
}