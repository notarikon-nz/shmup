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
