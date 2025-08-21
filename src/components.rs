use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub roll_factor: f32,
}

#[derive(Component, Clone)]
pub struct Enemy {
    pub ai_type: EnemyAI,
    pub health: i32,
    pub speed: f32,
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

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HighScoreText;

#[derive(Component)]
pub struct PowerUp {
    pub power_type: PowerUpType,
    pub bob_timer: f32,
}

#[derive(Clone)]
pub enum PowerUpType {
    Health { amount: i32 },
    // Future power-ups can be added here:
    // Shield { duration: f32 },
    // SpeedBoost { multiplier: f32, duration: f32 },
    // WeaponUpgrade { weapon_type: WeaponType },
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