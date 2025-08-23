use bevy::prelude::*;
use crate::components::{PowerUpType, ParticleConfig, ExplosionType};
use crate::enemy_types::{EnemyAI, EnemyType};
use crate::resources::{TidalPoolPhysics};

#[derive(Event)]
pub struct SpawnExplosion {
    pub position: Vec3,
    pub intensity: f32,
    pub enemy_type: Option<EnemyType>,
}

#[derive(Event)]
pub struct SpawnEnemy {
    pub position: Vec3,
    pub ai_type: EnemyAI,
    pub enemy_type: EnemyType,
}

#[derive(Event)]
pub struct SpawnPowerUp {
    pub position: Vec3,
    pub power_type: PowerUpType,
}

#[derive(Event)]
pub struct SpawnParticles {
    pub position: Vec3,
    pub count: u32,
    pub config: ParticleConfig,
}

#[derive(Event)]
pub struct PlayerHit {
    pub position: Vec3,
    pub damage: i32,
}

#[derive(Event)]
pub struct AddScreenShake {
    pub amount: f32,
}

#[derive(Event)]
pub struct EnemyHit {
    pub entity: Entity,
    pub position: Vec3,
}

#[derive(Event)]
pub struct SpawnEnhancedExplosion {
    pub position: Vec3,
    pub intensity: f32,
    pub explosion_type: ExplosionType,
}

// Tidal Events

#[derive(Event)]
pub enum TidalEvent {
    KingTideBegin { intensity: f32, duration: f32 },
    KingTideEnd,
    HighTideReached,
    LowTideReached,
    CurrentReversal { new_direction: Vec2 },
}

// Add to TidalPoolPhysics in resources.rs:
impl Default for TidalPoolPhysics {
    fn default() -> Self {
        Self {
            tide_level: 0.0,
            tide_cycle_speed: 0.02, // Slower for more dramatic effect
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