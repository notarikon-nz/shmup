use bevy::prelude::*;

#[derive(Resource)]
pub struct GameAssets {
    pub player_texture: Handle<Image>,
    pub enemy_texture: Handle<Image>,
    pub projectile_texture: Handle<Image>,
    pub explosion_texture: Handle<Image>,
    pub health_powerup_texture: Handle<Image>,
    pub background_layers: Vec<Handle<Image>>,
    pub sfx_shoot: Handle<AudioSource>,
    pub sfx_explosion: Handle<AudioSource>,
    pub sfx_powerup: Handle<AudioSource>,
    pub music: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct ProjectilePool {
    pub entities: Vec<Entity>,
    pub index: usize,
}

#[derive(Resource, Default)]
pub struct InputState {
    pub movement: Vec2,
    pub shooting: bool,
    pub shoot_timer: f32,
    pub gamepad: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct EnemySpawner {
    pub spawn_timer: f32,
    pub wave_timer: f32,
    pub enemies_spawned: u32,
    pub powerup_timer: f32,
}

#[derive(Resource, Default)]
pub struct GameScore {
    pub current: u32,
    pub high_scores: Vec<u32>,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct GameStarted(pub bool);