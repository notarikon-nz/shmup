use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Enemy {
    pub ai_type: EnemyAI,
    pub health: i32,
    pub speed: f32,
    pub enemy_type: EnemyType,
    pub formation_id: Option<u32>,
}

#[derive(Clone)]
pub enum EnemyType {
    Basic,
    Fast,
    Heavy,
    Boss,
    Kamikaze,
    Turret,
    FormationFighter,
    Spawner,
    SpawnerMinion,
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
}

#[derive(Component)]
pub struct FormationLeader {
    pub formation_id: u32,
    pub members: Vec<Entity>,
    pub pattern_timer: f32,
    pub pattern_type: FormationPattern,
}

#[derive(Clone)]
pub enum FormationPattern {
    VFormation,
    LineFormation,
    CircleFormation,
    DiamondFormation,
}

#[derive(Component)]
pub struct TurretCannon {
    pub parent_entity: Entity,
    pub offset: Vec2,
    pub rotation_speed: f32,
}

// Enemy stats configuration
impl EnemyType {
    pub fn get_stats(&self) -> (i32, f32, f32, Color) {
        match self {
            EnemyType::Basic => (20, 15.0, 150.0, Color::WHITE),
            EnemyType::Fast => (15, 12.0, 250.0, Color::srgb(0.3, 0.8, 1.0)),
            EnemyType::Heavy => (50, 20.0, 100.0, Color::srgb(0.8, 0.8, 0.3)),
            EnemyType::Boss => (100, 30.0, 120.0, Color::srgb(1.0, 0.3, 0.3)),
            EnemyType::Kamikaze => (10, 12.0, 200.0, Color::srgb(1.0, 0.5, 0.0)),
            EnemyType::Turret => (40, 25.0, 0.0, Color::srgb(0.6, 0.6, 0.6)),
            EnemyType::FormationFighter => (25, 14.0, 180.0, Color::srgb(0.5, 1.0, 0.5)),
            EnemyType::Spawner => (80, 22.0, 80.0, Color::srgb(0.8, 0.3, 0.8)),
            EnemyType::SpawnerMinion => (8, 8.0, 300.0, Color::srgb(0.6, 0.2, 0.6)),
        }
    }

    pub fn get_points(&self) -> u32 {
        match self {
            EnemyType::Basic => 100,
            EnemyType::Fast => 150,
            EnemyType::Heavy => 200,
            EnemyType::Boss => 1000,
            EnemyType::Kamikaze => 120,
            EnemyType::Turret => 250,
            EnemyType::FormationFighter => 180,
            EnemyType::Spawner => 500,
            EnemyType::SpawnerMinion => 50,
        }
    }
}

impl FormationPattern {
    pub fn get_position(&self, index: usize, total: usize, timer: f32) -> Vec2 {
        match self {
            FormationPattern::VFormation => {
                let angle = if index == 0 { 0.0 } else {
                    let side = if index % 2 == 1 { 1.0 } else { -1.0 };
                    let row = (index + 1) / 2;
                    side * 0.5 * row as f32
                };
                Vec2::new(angle * 60.0, -(index as f32) * 40.0)
            }
            FormationPattern::LineFormation => {
                let offset = (index as f32 - total as f32 / 2.0) * 50.0;
                Vec2::new(offset, 0.0)
            }
            FormationPattern::CircleFormation => {
                let angle = (index as f32 / total as f32) * std::f32::consts::TAU + timer * 0.5;
                Vec2::new(angle.cos() * 80.0, angle.sin() * 80.0)
            }
            FormationPattern::DiamondFormation => {
                match index {
                    0 => Vec2::new(0.0, 40.0),   // Top
                    1 => Vec2::new(-40.0, 0.0),  // Left
                    2 => Vec2::new(40.0, 0.0),   // Right
                    3 => Vec2::new(0.0, -40.0),  // Bottom
                    _ => Vec2::new(0.0, 0.0),
                }
            }
        }
    }
}