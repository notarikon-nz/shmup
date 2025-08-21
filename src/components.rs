use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub roll_factor: f32,
    pub lives: i32,
    pub invincible_timer: f32,
}

#[derive(Component, Clone)]
pub struct Enemy {
    pub ai_type: EnemyAI,
    pub health: i32,
    pub speed: f32,
    pub enemy_type: EnemyType,
}

#[derive(Clone)]
pub enum EnemyType {
    Basic,
    Fast,
    Heavy,
    Boss,
}

#[derive(Component)]
pub enum EnemyAI {
    Static,
    Linear { direction: Vec2 },
    Sine { amplitude: f32, frequency: f32, phase: f32 },
    MiniBoss { pattern: usize, timer: f32 },
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

// Helper implementations
impl Clone for EnemyAI {
    fn clone(&self) -> Self {
        match self {
            EnemyAI::Static => EnemyAI::Static,
            EnemyAI::Linear { direction } => EnemyAI::Linear { direction: *direction },
            EnemyAI::Sine { amplitude, frequency, phase } => EnemyAI::Sine {
                amplitude: *amplitude,
                frequency: *frequency,
                phase: *phase,
            },
            EnemyAI::MiniBoss { pattern, timer } => EnemyAI::MiniBoss {
                pattern: *pattern,
                timer: *timer,
            },
        }
    }
}

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