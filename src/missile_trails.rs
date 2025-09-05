// src/missile_trails.rs - Efficient missile trail system using line segments
use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::despawn::*;

// ===== CONSTANTS =====
const MAX_TRAIL_SEGMENTS: usize = 8;           // Maximum segments per trail
const TRAIL_SEGMENT_DISTANCE: f32 = 15.0;      // Distance between segments
const TRAIL_FADE_TIME: f32 = 0.8;              // Time for trail to fade completely
const TRAIL_WIDTH: f32 = 3.0;                  // Base width of trail segments
const TRAIL_COLOR_START: Color = Color::srgb(0.9, 0.5, 0.3); // Orange missile color
const TRAIL_COLOR_END: Color = Color::srgba(0.9, 0.3, 0.1, 0.0); // Faded red-orange

// ===== COMPONENTS =====
#[derive(Clone, Component)]
pub struct MissileTrail {
    pub segments: Vec<TrailSegment>,
    pub last_position: Vec3,
    pub trail_type: MissileTrailType,
}

#[derive(Clone, Component)]
pub struct TrailSegment {
    pub position: Vec3,
    pub age: f32,
    pub width_factor: f32,
    pub creation_time: f32,
}

#[derive(Component)]
pub struct TrailRenderer {
    pub trail_entity: Entity,
    pub segment_index: usize,
}

#[derive(Clone, Copy)]
pub enum MissileTrailType {
    Standard,      // Orange trail for regular missiles
    Symbiotic,     // Pulsing bio-luminescent trail
    AutoMissile,   // Enhanced homing missile trail
}

impl MissileTrailType {
    pub fn get_colors(&self) -> (Color, Color) {
        match self {
            MissileTrailType::Standard => (TRAIL_COLOR_START, TRAIL_COLOR_END),
            MissileTrailType::Symbiotic => (
                Color::srgb(1.0, 0.7, 0.3), 
                Color::srgba(0.8, 0.4, 0.1, 0.0)
            ),
            MissileTrailType::AutoMissile => (
                Color::srgb(0.9, 0.6, 0.4), 
                Color::srgba(0.7, 0.3, 0.2, 0.0)
            ),
        }
    }
    
    pub fn get_width_modifier(&self) -> f32 {
        match self {
            MissileTrailType::Standard => 1.0,
            MissileTrailType::Symbiotic => 1.3,  // Slightly thicker for bio effect
            MissileTrailType::AutoMissile => 1.1,
        }
    }
}

// ===== MISSILE TRAIL SPAWNING =====
pub fn setup_missile_trails(
    mut commands: Commands,
    // Target all missile types without conflicting queries
    missile_query: Query<Entity, (
        Or<(With<MissileProjectile>, With<AutoMissile>)>, 
        Without<MissileTrail>
    )>,
    symbiotic_query: Query<&MissileProjectile>,
    auto_query: Query<&AutoMissile>,
) {
    for missile_entity in missile_query.iter() {
        // Determine trail type based on missile components
        let trail_type = if symbiotic_query.get(missile_entity).map(|m| m.symbiotic).unwrap_or(false) {
            MissileTrailType::Symbiotic
        } else if auto_query.contains(missile_entity) {
            MissileTrailType::AutoMissile
        } else {
            MissileTrailType::Standard
        };
        
        commands.entity(missile_entity).insert(MissileTrail {
            segments: Vec::with_capacity(MAX_TRAIL_SEGMENTS),
            last_position: Vec3::ZERO,
            trail_type,
        });
    }
}

// ===== TRAIL UPDATE SYSTEM =====
pub fn update_missile_trails(
    mut commands: Commands,
    mut trail_query: Query<(Entity, &Transform, &mut MissileTrail)>,
    mut trail_renderer_query: Query<(Entity, &mut Transform, &mut Sprite, &TrailRenderer), Without<MissileTrail>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    let Some(assets) = assets else { return };
    let dt = time.delta_secs();
    
    // Update missile trails
    for (missile_entity, missile_transform, mut trail) in trail_query.iter_mut() {
        let current_pos = missile_transform.translation;
        let distance_moved = current_pos.distance(trail.last_position);
        
        let trail_last_position = trail.last_position;
        // Add new segment if missile moved enough distance
        if distance_moved >= TRAIL_SEGMENT_DISTANCE {
            trail.segments.push(TrailSegment {
                position: trail_last_position,
                age: 0.0,
                width_factor: 1.0,
                creation_time: time.elapsed_secs(),
            });
            trail.last_position = current_pos;
            
            // Remove old segments if we exceed max
            if trail.segments.len() > MAX_TRAIL_SEGMENTS {
                trail.segments.remove(0);
            }
        }
        
        // Age existing segments and remove expired ones
        trail.segments.retain_mut(|segment| {
            segment.age += dt;
            segment.width_factor = (1.0 - segment.age / TRAIL_FADE_TIME).max(0.0);
            segment.age < TRAIL_FADE_TIME
        });

        let trail_clone = trail.clone();

        // Spawn/update trail renderer entities
        for (i, segment) in trail.segments.iter_mut().enumerate() {
            segment.age += dt;
            let segment_lifetime = time.elapsed_secs() - segment.creation_time;
            
            // Start fading after 1 second, fully transparent after 2 seconds
            let alpha = if segment_lifetime > 1.0 {
                (2.0 - segment_lifetime).max(0.0) // Fade from 1.0 to 0.0 over 1 second
            } else {
                1.0 // Full opacity for first second
            };
            
            if alpha > 0.0 {
                let segment_entity = commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: interpolate_trail_color(trail_clone.trail_type, 1.0 - alpha).with_alpha(alpha),
                        custom_size: Some(Vec2::splat(
                            TRAIL_WIDTH * trail_clone.trail_type.get_width_modifier() * alpha
                        )),
                        ..default()
                    },
                    Transform::from_translation(segment.position + Vec3::new(0.0, 0.0, -0.1)),
                    PendingDespawn { delay: (2.0 - segment_lifetime).max(0.0) },
                )).id();
            }
        }

        trail.segments.retain(|segment| {
            time.elapsed_secs() - segment.creation_time < 2.0
        });        
    }
    
    // Clean up orphaned trail renderers
    for (renderer_entity, _transform, _sprite, trail_renderer) in trail_renderer_query.iter() {
        if trail_query.get(trail_renderer.trail_entity).is_err() {
            commands.entity(renderer_entity).insert(PendingDespawn { delay: 0.1 });
            // commands.entity(renderer_entity).safe_despawn();
        }
    }
}

// ===== TRAIL CLEANUP =====
pub fn cleanup_missile_trails(
    mut commands: Commands,
    trail_renderer_query: Query<(Entity, &TrailRenderer)>,
    trail_query: Query<&MissileTrail, (Without<PendingDespawn>)>,
) {
    // Clean up trail renderers for despawned missiles
    for (renderer_entity, trail_renderer) in trail_renderer_query.iter() {
        if trail_query.get(trail_renderer.trail_entity).is_err() {
            commands.entity(renderer_entity)
                .safe_despawn();
        }
    }
}

// ===== HELPER FUNCTIONS =====
fn interpolate_trail_color(trail_type: MissileTrailType, fade_progress: f32) -> Color {
    let (start_color, end_color) = trail_type.get_colors();
    
    // For symbiotic trails, add pulsing effect
    let pulse_factor = if matches!(trail_type, MissileTrailType::Symbiotic) {
        1.0 + (fade_progress * 10.0).sin() * 0.2
    } else {
        1.0
    };
    
    let r = start_color.to_srgba().red * (1.0 - fade_progress) + end_color.to_srgba().red * fade_progress;
    let g = start_color.to_srgba().green * (1.0 - fade_progress) + end_color.to_srgba().green * fade_progress;
    let b = start_color.to_srgba().blue * (1.0 - fade_progress) + end_color.to_srgba().blue * fade_progress;
    let a = (start_color.to_srgba().alpha * (1.0 - fade_progress) + end_color.to_srgba().alpha * fade_progress) * pulse_factor;
    
    Color::srgba(r, g, b, a.clamp(0.0, 1.0))
}

// ===== PERFORMANCE OPTIMIZATION =====
pub fn optimize_trail_performance(
    trail_query: Query<&MissileTrail>,
    mut trail_count: Local<usize>,
    time: Res<Time>,
) {
    *trail_count = trail_query.iter().count();
    
    // If we have too many trails, we could implement culling here
    if *trail_count > 50 {
        // Could implement distance-based culling or reduce trail length
        // For now, just log a performance warning
        if (time.elapsed_secs() % 5.0) < 0.1 {
            info!("Performance: {} missile trails active", trail_count.to_string());
        }
    }
}