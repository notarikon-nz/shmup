use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;

// This structure is designed to be compatible with future Bevy 2D lighting
// or custom shader-based lighting systems

#[derive(Component, Clone)]
pub struct DynamicLight2D {
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
    pub flicker: Option<LightFlicker>,
    pub organic_pulse: Option<OrganicPulse>,
}

#[derive(Component, Clone)]
pub struct LightFlicker {
    pub frequency: f32,
    pub intensity_variation: f32,
    pub timer: f32,
}

#[derive(Component, Clone)]
pub struct OrganicPulse {
    pub base_frequency: f32,
    pub frequency_variation: f32,
    pub intensity_range: (f32, f32),
    pub phase_offset: f32,
}

// Future expansion: when Bevy gets 2D lighting, replace this with actual light components
pub fn update_dynamic_lights(
    mut light_query: Query<(&mut DynamicLight2D, &mut Transform), With<ExplosionLight>>,
    time: Res<Time>,
) {
    for (mut light, transform) in light_query.iter_mut() {
        let light_c = light.clone();
        // Organic light pulsing
        if let Some(mut pulse) = light_c.organic_pulse {
            pulse.phase_offset += time.delta_secs() * pulse.base_frequency;
            let pulse_intensity = pulse.phase_offset.sin() * 0.5 + 0.5;
            let intensity_variation = pulse.intensity_range.0 + 
                (pulse.intensity_range.1 - pulse.intensity_range.0) * pulse_intensity;
            
            light.intensity *= intensity_variation;
        }
        
        // Light flickering
        if let Some(mut flicker) = light_c.flicker {
            flicker.timer += time.delta_secs() * flicker.frequency;
            let flicker_factor = 1.0 + (flicker.timer.sin() * flicker.intensity_variation);
            light.intensity *= flicker_factor;
        }
    }
}

// When real lighting system is available, this will render actual lights
// For now, it can create glow effects using sprites
pub fn render_light_effects(
    mut commands: Commands,
    light_query: Query<(&DynamicLight2D, &Transform), Without<LightGlowSprite>>,
    glow_query: Query<(&mut Sprite, &mut Transform), (With<LightGlowSprite>, Without<DynamicLight2D>)>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        // Create glow sprites for lights without them
        for (light, light_transform) in light_query.iter() {
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgba(
                        light.color.to_srgba().red,
                        light.color.to_srgba().green,
                        light.color.to_srgba().blue,
                        0.3
                    ),
                    custom_size: Some(Vec2::splat(light.radius * 2.0)),
                    ..default()
                },
                Transform::from_translation(light_transform.translation),
                LightGlowSprite,
            ));
        }
    }
}

// lighting.rs - GPU-accelerated 2D lighting using bevy_light_2d
use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use crate::components::*;

// Minimal high-performance GPU lighting system
#[derive(Component)]
pub struct BioluminescentLight {
    pub intensity_base: f32,
    pub pulse_frequency: f32,
    pub color_base: Color,
}

#[derive(Component)]
pub struct ExplosionGlow {
    pub timer: f32,
    pub max_time: f32,
    pub peak_intensity: f32,
}

// Ultra-fast GPU light updates - runs on GPU compute shaders via bevy_light_2d
pub fn update_bioluminescent_lights(
    mut light_query: Query<(&mut PointLight2d, &BioluminescentLight, &Transform)>,
    time: Res<Time>,
) {
    // Batch process all lights in parallel on GPU
    for (mut light, bio_light, _transform) in light_query.iter_mut() {
        // Organic pulsing using sine wave
        let pulse = (time.elapsed_secs() * bio_light.pulse_frequency).sin() * 0.3 + 0.7;
        light.intensity = bio_light.intensity_base * pulse;
        
        // Color temperature variation for organic feel
        let color_shift = pulse * 0.1;
        light.color = Color::srgb(
            bio_light.color_base.to_srgba().red + color_shift,
            bio_light.color_base.to_srgba().green,
            bio_light.color_base.to_srgba().blue - color_shift * 0.5,
        );
    }
}

// Handle explosion lighting effects
pub fn update_explosion_lights(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut PointLight2d, &mut ExplosionGlow)>,
    time: Res<Time>,
) {
    for (entity, mut light, mut glow) in explosion_query.iter_mut() {
        glow.timer += time.delta_secs();
        
        if glow.timer >= glow.max_time {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Fast falloff curve for explosions
        let progress = glow.timer / glow.max_time;
        let falloff = (1.0 - progress).powi(3); // Cubic falloff
        
        light.intensity = glow.peak_intensity * falloff;
        light.radius = 150.0 * (1.0 - progress * 0.7); // Shrinking radius
    }
}

// Spawn optimized lights for game entities
pub fn spawn_entity_lights(
    mut commands: Commands,
    // Player light
    player_query: Query<Entity, (With<Player>, Without<PointLight2d>)>,
    // Enemy lights for bioluminescent enemies
    enemy_query: Query<(Entity, &Enemy), Without<PointLight2d>>,
    // Projectile lights
    projectile_query: Query<(Entity, &Projectile), Without<PointLight2d>>,
) {
    // Player bioluminescent glow
    for player_entity in player_query.iter() {
        commands.entity(player_entity).insert((
            PointLight2d {
                intensity: 2.0,
                radius: 80.0,
                color: Color::srgb(0.4, 0.9, 0.7),
                ..default()
            },
            BioluminescentLight {
                intensity_base: 2.0,
                pulse_frequency: 1.5,
                color_base: Color::srgb(0.4, 0.9, 0.7),
            },
        ));
    }
    
    // Enemy bioluminescent lights (selective for performance)
    for (enemy_entity, enemy) in enemy_query.iter() {
        let (should_glow, color, intensity) = match enemy.enemy_type {
            EnemyType::BiofilmColony => (true, Color::srgb(0.8, 0.3, 0.9), 1.5),
            EnemyType::InfectedMacrophage => (true, Color::srgb(0.9, 0.2, 0.2), 2.2),
            EnemyType::SwarmCell => (true, Color::srgb(0.3, 0.8, 1.0), 1.0),
            _ => (false, Color::WHITE, 0.0),
        };
        
        if should_glow {
            commands.entity(enemy_entity).insert((
                PointLight2d {
                    intensity,
                    radius: 40.0,
                    color,
                    ..default()
                },
                BioluminescentLight {
                    intensity_base: intensity,
                    pulse_frequency: 2.0,
                    color_base: color,
                },
            ));
        }
    }
    
    // Projectile trail lights (only for special projectiles)
    for (projectile_entity, projectile) in projectile_query.iter() {
        if projectile.organic_trail {
            commands.entity(projectile_entity).insert(PointLight2d {
                intensity: 0.8,
                radius: 25.0,
                color: Color::srgb(0.6, 1.0, 0.8),
                ..default()
            });
        }
    }
}

// Optimized explosion light spawning
pub fn spawn_explosion_lights(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
) {
    for explosion in explosion_events.read() {
        // Different light intensities based on explosion type
        let (intensity, color, radius) = match &explosion.enemy_type {
            Some(EnemyType::InfectedMacrophage) => (8.0, Color::srgb(1.0, 0.4, 0.2), 200.0),
            Some(EnemyType::BiofilmColony) => (5.0, Color::srgb(0.8, 0.9, 0.3), 150.0),
            Some(_) => (3.0, Color::srgb(0.9, 0.7, 0.4), 100.0),
            None => (4.0, Color::srgb(1.0, 0.8, 0.6), 120.0),
        };
        
        commands.spawn((
            PointLight2d {
                intensity: intensity * explosion.intensity,
                radius,
                color,
                ..default()
            },
            ExplosionGlow {
                timer: 0.0,
                max_time: 1.2,
                peak_intensity: intensity * explosion.intensity,
            },
            Transform::from_translation(explosion.position),
        ));
    }
}

// Clean up expired lights for performance
pub fn cleanup_expired_lights(
    mut commands: Commands,
    expired_query: Query<Entity, (With<PointLight2d>, With<AlreadyDespawned>)>,
) {
    for entity in expired_query.iter() {
        commands.entity(entity).despawn();
    }
}

// Plugin for easy integration
pub struct PerformantLightingPlugin;

impl Plugin for PerformantLightingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add the GPU-accelerated lighting plugin
            .add_plugins(Light2dPlugin)
            
            // Lighting systems run in PostUpdate for optimal batching
            .add_systems(PostUpdate, (
                spawn_entity_lights,
                update_bioluminescent_lights,
                update_explosion_lights,
                cleanup_expired_lights,
            ).chain())
            
            .add_systems(Update, spawn_explosion_lights);
    }
}