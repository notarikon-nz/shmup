use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::despawn::*;

pub fn spawn_extra_life_powerup(
    mut commands: Commands,
    enemy_spawner: ResMut<EnemySpawner>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut life_spawn_timer: Local<f32>,
) {
    if let Some(assets) = assets {
        *life_spawn_timer += time.delta_secs();

        // Spawn extra life every 120 seconds
        if *life_spawn_timer >= 120.0 {
            *life_spawn_timer = 0.0;

            let x_position = (time.elapsed_secs() * 30.0).sin() * 300.0;

            commands.spawn((
                Sprite {
                    image: assets.health_powerup_texture.clone(),
                    color: Color::srgb(1.0, 0.3, 0.8), // Pink heart color
                    custom_size: Some(Vec2::splat(20.0)),
                    ..default()
                },
                Transform::from_xyz(x_position, 420.0, 0.0),
                Collider { radius: 10.0 },
                PowerUp {
                    power_type: PowerUpType::CellularRegeneration { amount: 0 }, // Dummy type
                    bob_timer: 0.0,
                    bioluminescent_pulse: 0.0,
                },
                BioluminescentParticle {
                    base_color: Color::srgb(1.0, 0.3, 0.8),
                    pulse_frequency: 1.5,
                    pulse_intensity: 0.8,
                    organic_motion: OrganicMotion {
                        undulation_speed: 2.0,
                        response_to_current: 0.5,
                    },
                },
            ));
        }
    }
}

// 7. Extra Life Collection
pub fn collect_extra_life(
    mut commands: Commands,
    extra_life_query: Query<(Entity, &Transform, &Collider), (With<ExtraLifePowerUp>, Without<PendingDespawn>)>,
    mut player_query: Query<(&Transform, &Collider, &mut Player)>,
    mut particle_events: EventWriter<SpawnParticles>,
) {
    if let Ok((player_transform, player_collider, mut player)) = player_query.single_mut() {
        for (life_entity, life_transform, life_collider) in extra_life_query.iter() {
            let distance = player_transform.translation.distance(life_transform.translation);
            if distance < player_collider.radius + life_collider.radius {
                player.lives += 1;

                // Spawn celebration particles
                particle_events.write(SpawnParticles {
                    position: life_transform.translation,
                    count: 20,
                    config: ParticleConfig {
                        color_start: Color::srgb(1.0, 0.3, 0.8),
                        color_end: Color::srgba(1.0, 0.8, 0.9, 0.0),
                        velocity_range: (Vec2::new(-100.0, -50.0), Vec2::new(100.0, 100.0)),
                        lifetime_range: (1.0, 2.0),
                        size_range: (0.4, 1.0),
                        gravity: Vec2::new(0.0, -20.0),
                        organic_motion: true,
                        bioluminescence: 1.0,
                    },
                });

                commands.entity(life_entity)
                    .safe_despawn();
            }
        }
    }
}

pub fn spawn_powerup_system(
    mut commands: Commands,
    mut powerup_events: EventReader<SpawnPowerUp>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in powerup_events.read() {
            let (texture, color) = match &event.power_type {
                PowerUpType::CellularRegeneration { .. } => (assets.health_powerup_texture.clone(), Color::srgb(0.4, 1.0, 0.6)),
                PowerUpType::CellWall { .. } => (assets.shield_powerup_texture.clone(), Color::srgb(0.4, 1.0, 0.8)),
                PowerUpType::Flagella { .. } => (assets.speed_powerup_texture.clone(), Color::srgb(0.6, 0.9, 1.0)),
                PowerUpType::SymbioticBoost { .. } => (assets.multiplier_powerup_texture.clone(), Color::srgb(1.0, 0.8, 0.4)),
                PowerUpType::MitochondriaOvercharge { .. } => (assets.rapidfire_powerup_texture.clone(), Color::srgb(1.0, 0.6, 0.8)),
                PowerUpType::Photosynthesis { .. } => (assets.health_powerup_texture.clone(), Color::srgb(0.6, 1.0, 0.3)),
                PowerUpType::Chemotaxis { .. } => (assets.speed_powerup_texture.clone(), Color::srgb(0.8, 0.6, 1.0)),
                PowerUpType::Osmoregulation { .. } => (assets.shield_powerup_texture.clone(), Color::srgb(0.3, 0.8, 0.9)),
                PowerUpType::BinaryFission { .. } => (assets.rapidfire_powerup_texture.clone(), Color::srgb(1.0, 0.9, 0.3)),
                PowerUpType::MagneticField { .. } => (assets.multiplier_powerup_texture.clone(), Color::srgb(0.4, 0.9, 0.4)),
            };
            
            commands.spawn((
                Sprite {
                    image: texture,
                    color,
                    ..default()
                },
                Transform::from_translation(event.position),
                PowerUp {
                    power_type: event.power_type.clone(),
                    bob_timer: 0.0,
                    bioluminescent_pulse: 0.0,
                },
                Collider { radius: 12.0 },
                BioluminescentParticle {
                    base_color: color,
                    pulse_frequency: 2.5,
                    pulse_intensity: 0.5,
                    organic_motion: OrganicMotion {
                        undulation_speed: 1.8,
                        response_to_current: 0.7,
                    },
                },
            ));
        }
    }
}

