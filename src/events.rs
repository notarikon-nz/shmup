use bevy::prelude::*;
use crate::components::{EnemyAI, PowerUpType};

#[derive(Event)]
pub struct SpawnExplosion {
    pub position: Vec3,
    pub intensity: f32,
}

#[derive(Event)]
pub struct SpawnEnemy {
    pub position: Vec3,
    pub ai_type: EnemyAI,
}

#[derive(Event)]
pub struct SpawnPowerUp {
    pub position: Vec3,
    pub power_type: PowerUpType,
}