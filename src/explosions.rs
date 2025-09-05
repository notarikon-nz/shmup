use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;
use crate::despawn::*;
use crate::hanabi_particles::*;

#[derive(Resource)]
pub struct EntityPools {
    pub available_particles: Vec<Entity>,
    pub active_particles: Vec<Entity>,
    pub available_explosions: Vec<Entity>,
    pub active_explosions: Vec<Entity>,
}

impl Default for EntityPools {
    fn default() -> Self {
        Self {
            available_particles: Vec::with_capacity(500),
            active_particles: Vec::with_capacity(500),
            available_explosions: Vec::with_capacity(20),
            active_explosions: Vec::with_capacity(20),
        }
    }
}

#[derive(Component)]
pub struct PooledEntity {
    pub pool_type: PoolType,
    pub in_use: bool,
}

#[derive(Clone)]
pub enum PoolType {
    Particle,
    Explosion,
    Projectile,
    DamageText,
}

// ===== BATCH PARTICLE SPAWNING =====
struct ParticleData {
    position: Vec3,
    velocity: Vec2,
    color: Color,
    size: f32,
    lifetime: f32,
    drift_pattern: DriftPattern,
}

// ===== POOL INITIALIZATION =====
pub fn init_entity_pools(mut commands: Commands) {
    // Start with empty pools - entities will be created as needed
    let pools = EntityPools::default();
    commands.insert_resource(pools);
}


// ===== OPTIMIZED EXPLOSION SYSTEM =====
pub fn optimized_explosion_system(
    mut commands: Commands,
    mut pools: ResMut<EntityPools>,
    mut explosion_query: Query<(Entity, &mut Explosion, &mut Transform, &mut Sprite), 
        (With<Explosion>, Without<PendingDespawn>)>,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut shake_events: EventWriter<AddScreenShake>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        // Handle new explosions
        for event in explosion_events.read() {
            let explosion_entity = if let Some(pooled_entity) = pools.available_explosions.pop() {
                // Move from available to active
                pools.active_explosions.push(pooled_entity);
                pooled_entity
            } else {
                // Create new entity and add to active
                let entity = commands.spawn_empty().id();
                pools.active_explosions.push(entity);
                entity
            };

            let explosion_type = get_explosion_type(&event.enemy_type);
            let layers = create_optimized_layers(&explosion_type, event.intensity);
            
            shake_events.write(AddScreenShake { amount: event.intensity * 0.3 });
            
            // Insert all components fresh
            commands.entity(explosion_entity).insert((
                Sprite {
                    image: assets.explosion_texture.clone(),
                    color: get_explosion_color(&explosion_type),
                    custom_size: Some(Vec2::splat(32.0 * event.intensity)),
                    ..default()
                },
                Transform::from_translation(event.position),
                Explosion {
                    timer: 0.0,
                    max_time: 1.0,
                    intensity: event.intensity,
                    explosion_type,
                    layers,
                    current_layer_index: 0,
                },
                PooledEntity { pool_type: PoolType::Explosion, in_use: true },
            ));
        }
        
        // Update explosions with batch particle spawning
        let mut particles_to_spawn = Vec::new();
        let mut completed_explosions = Vec::new();
        
        for (entity, mut explosion, mut transform, mut sprite) in explosion_query.iter_mut() {

            let explosion_clone = explosion.clone();

            explosion.timer += time.delta_secs();
            
            if explosion.timer >= explosion.max_time {
                completed_explosions.push(entity);
                continue;
            }
            
            // Batch particle spawning - collect all particles to spawn
            for layer in &mut explosion.layers {
                if !layer.completed && explosion_clone.timer >= layer.delay {
                    let layer_progress = (explosion_clone.timer - layer.delay) / layer.duration;
                    
                    if layer_progress >= 1.0 {
                        layer.completed = true;
                        continue;
                    }
                    
                    // Collect particle data instead of spawning immediately
                    collect_layer_particles(
                        &mut particles_to_spawn, 
                        &transform, 
                        layer, 
                        layer_progress,
                        &explosion_clone.explosion_type
                    );
                }
            }
            
            // Update main explosion
            let progress = explosion_clone.timer / explosion_clone.max_time;
            let scale = explosion_clone.intensity * (1.0 + progress * 0.8);
            transform.scale = Vec3::splat(scale);
            sprite.color.set_alpha((1.0 - progress).powi(2));
        }
        
        // Clean up completed explosions
        for entity in completed_explosions {
            if let Some(index) = pools.active_explosions.iter().position(|&e| e == entity) {
                pools.active_explosions.remove(index);
                pools.available_explosions.push(entity);
            }
            
            commands.entity(entity).remove::<(Explosion, Sprite, Transform)>();
        }
        
        // Batch spawn all collected particles
        batch_spawn_particles(&mut commands, &mut pools, &assets, particles_to_spawn);
    }
}


// ===== HELPER FUNCTIONS =====
fn get_explosion_type(enemy_type: &Option<EnemyType>) -> ExplosionType {
    match enemy_type {
        Some(EnemyType::InfectedMacrophage) => ExplosionType::Biological { 
            toxin_release: true, 
            membrane_rupture: true 
        },
        Some(EnemyType::BiofilmColony) => ExplosionType::Chemical { 
            ph_change: -1.2, 
            oxygen_release: 0.3 
        },
        Some(EnemyType::AggressiveBacteria) => ExplosionType::Biological { 
            toxin_release: true, 
            membrane_rupture: false 
        },
        _ => ExplosionType::Standard,
    }
}


// ===== REDUCED EXPLOSION LAYERS =====
fn create_optimized_layers(explosion_type: &ExplosionType, intensity: f32) -> Vec<ExplosionLayer> {
    // Significantly fewer layers for performance
    match explosion_type {
        ExplosionType::Biological { .. } => vec![
            ExplosionLayer {
                phase: ExplosionPhase::Membrane,
                delay: 0.0,
                duration: 0.15, // Shorter duration
                particle_count: (15.0 * intensity) as u32, // Reduced from 25
                color_start: Color::srgb(0.8, 1.0, 0.7),
                color_end: Color::srgba(0.4, 0.8, 0.6, 0.0),
                size_range: (2.0, 6.0),
                velocity_range: (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0)),
                completed: false,
            },
        ],
        _ => vec![
            ExplosionLayer {
                phase: if intensity < 1.0 { ExplosionPhase::MiniBlast } else { ExplosionPhase::CoreBlast },
                delay: 0.0,
                duration: 0.25,
                particle_count: (8.0 * intensity) as u32, // Reduced from 20
                color_start: Color::srgb(1.0, 0.8, 0.4),
                color_end: Color::srgba(1.0, 0.4, 0.2, 0.0),
                size_range: (1.0, 4.0),
                velocity_range: (Vec2::new(-120.0, -120.0), Vec2::new(120.0, 120.0)),
                completed: false,
            },
        ]
    }
}

fn collect_layer_particles(
    particles: &mut Vec<ParticleData>,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
    explosion_type: &ExplosionType,
) {
    if progress > 0.3 { return; } // Only spawn early in layer
    
    // Reduced particle counts
    let count = match layer.phase {
        ExplosionPhase::Shockwave => 8,     // Was 12
        ExplosionPhase::CoreBlast => 12,    // Was 40
        ExplosionPhase::Membrane => 4,      // Was 6
        ExplosionPhase::MiniBlast => 6,     // Was 10
        _ => 3,
    };
    
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let speed = match layer.phase {
            ExplosionPhase::Shockwave => 200.0,
            ExplosionPhase::CoreBlast => 120.0,
            _ => 80.0,
        };
        
        particles.push(ParticleData {
            position: transform.translation,
            velocity: Vec2::from_angle(angle) * speed,
            color: layer.color_start,
            size: layer.size_range.0,
            lifetime: 0.6, // Reduced from various values
            drift_pattern: match explosion_type {
                ExplosionType::Biological { .. } => DriftPattern::Floating,
                _ => DriftPattern::Pulsing,
            },
        });
    }
}

fn batch_spawn_particles(
    commands: &mut Commands,
    pools: &mut EntityPools,
    assets: &GameAssets,
    particle_data: Vec<ParticleData>,
) {
    const MAX_SPAWN_PER_FRAME: usize = 50; // Reduced limit
    
    for (i, data) in particle_data.iter().enumerate() {
        if i >= MAX_SPAWN_PER_FRAME { break; }
        
        let particle_entity = if let Some(pooled_entity) = pools.available_particles.pop() {
            // Move from available to active
            pools.active_particles.push(pooled_entity);
            pooled_entity
        } else {
            // Create new entity and add to active
            let entity = commands.spawn_empty().id();
            pools.active_particles.push(entity);
            entity
        };

        commands.entity(particle_entity).insert((
            Sprite {
                image: assets.particle_texture.clone(),
                color: data.color,
                custom_size: Some(Vec2::splat(data.size)),
                ..default()
            },
            Transform::from_translation(data.position).with_scale(Vec3::splat(data.size)),
            Particle {
                velocity: data.velocity,
                lifetime: 0.0,
                max_lifetime: data.lifetime,
                size: data.size,
                fade_rate: 1.0,
                bioluminescent: false,
                drift_pattern: data.drift_pattern,
            },
            PooledEntity { pool_type: PoolType::Particle, in_use: true },
        ));
    }
}

// ===== ADDITIONAL PERFORMANCE IMPROVEMENTS =====

// Simplified particle defaults
impl Default for Particle {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            lifetime: 0.0,
            max_lifetime: 1.0,
            size: 2.0,
            fade_rate: 1.0,
            bioluminescent: false,
            drift_pattern: DriftPattern::Pulsing,
        }
    }
}



























// ===== 


pub fn consolidated_explosion_system(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Explosion, &mut Transform, &mut Sprite), Without<PendingDespawn>>,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut shake_events: EventWriter<AddScreenShake>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        // Handle new explosion events
        for event in explosion_events.read() {
            let explosion_type = match &event.enemy_type {
                Some(EnemyType::InfectedMacrophage) => ExplosionType::Biological { 
                    toxin_release: true, 
                    membrane_rupture: true 
                },
                Some(EnemyType::BiofilmColony) => ExplosionType::Chemical { 
                    ph_change: -1.5, 
                    oxygen_release: 0.3 
                },
                Some(EnemyType::AggressiveBacteria) => ExplosionType::Biological { 
                    toxin_release: true, 
                    membrane_rupture: false 
                },
                _ => ExplosionType::Standard,
            };
            
            let layers = create_explosion_layers(&explosion_type, event.intensity);
            let shake_amount = calculate_shake_amount(&explosion_type, event.intensity);
            
            shake_events.write(AddScreenShake { amount: shake_amount });
            
            // Get explosion color for lighting
            let light_color = get_explosion_light_color(&explosion_type);
            let light_intensity = 2000.0 * event.intensity;
            let light_range = 80.0 * event.intensity;
            
            // Spawn unified explosion entity with enhanced lighting
            let explosion_entity = commands.spawn((
                Sprite {
                    image: assets.explosion_texture.clone(),
                    color: get_explosion_color(&explosion_type),
                    custom_size: Some(Vec2::splat(32.0 * event.intensity)),
                    ..default()
                },
                Transform::from_translation(event.position),
                Explosion {
                    timer: 0.0,
                    max_time: 2.0,
                    intensity: event.intensity,
                    explosion_type: explosion_type.clone(),
                    layers,
                    current_layer_index: 0,
                },
                // FIXED: Add PointLight for enhanced visual effect
                PointLight {
                    color: light_color,
                    intensity: light_intensity,
                    range: light_range,
                    radius: light_range * 0.8,
                    shadows_enabled: false,
                    affects_lightmapped_mesh_diffuse: false,
                    shadow_depth_bias: 0.0,
                    shadow_map_near_z: 0.0,
                    shadow_normal_bias: 0.0,

                },
            )).id();
            
            // Link light to explosion for cleanup
            commands.entity(explosion_entity).insert(LinkedExplosionLight(explosion_entity));
        }
        
        // Update existing explosions with dynamic lighting
        for (entity, mut explosion, mut transform, mut sprite) in explosion_query.iter_mut() {
            explosion.timer += time.delta_secs();
            
            if explosion.timer >= explosion.max_time {
                commands.entity(entity)
                    .safe_despawn();
                continue;
            }
            
            let explosion_clone = explosion.clone();
            
            // Process explosion layers in sequence
            for (i, layer) in explosion.layers.iter_mut().enumerate() {
                if !layer.completed && explosion_clone.timer >= layer.delay {
                    let layer_progress = (explosion_clone.timer - layer.delay) / layer.duration;
                    
                    if layer_progress >= 1.0 {
                        layer.completed = true;
                        continue;
                    }
                    
                    // Execute layer effects based on phase
                    match layer.phase {
                        ExplosionPhase::Shockwave => {
                            update_shockwave_layer(&mut commands, &assets, &transform, layer, layer_progress, &explosion_clone.explosion_type);
                        }
                        ExplosionPhase::CoreBlast => {
                            update_core_blast_layer(&mut commands, &assets, &transform, layer, layer_progress, explosion_clone.intensity);
                        }
                        ExplosionPhase::Debris => {
                            update_debris_layer(&mut commands, &assets, &transform, layer, layer_progress, &explosion_clone.explosion_type);
                        }
                        ExplosionPhase::Afterglow => {
                            update_afterglow_layer(&mut commands, &assets, &transform, layer, layer_progress);
                        }
                        ExplosionPhase::Membrane => {
                            update_membrane_layer(&mut commands, &assets, &transform, layer, layer_progress);
                        }
                        ExplosionPhase::MiniBlast => {
                            update_mini_blast_layer(&mut commands, &assets, &transform, layer, layer_progress);
                        }
                    }
                }
            }
            
            // Update main explosion sprite
            let global_progress = explosion.timer / explosion.max_time;
            let scale = explosion.intensity * (1.0 + global_progress * 1.5);
            transform.scale = Vec3::splat(scale);
            sprite.color.set_alpha(0.8 * (1.0 - global_progress).powi(2));
            
            // Organic rotation
            transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.5);
        }
    }
}

// Helper function for explosion light colors
fn get_explosion_light_color(explosion_type: &ExplosionType) -> Color {
    match explosion_type {
        ExplosionType::Biological { .. } => Color::srgb(0.4, 1.0, 0.6),
        ExplosionType::Chemical { .. } => Color::srgb(0.8, 0.8, 0.2),
        ExplosionType::Electrical { .. } => Color::srgb(0.2, 0.6, 1.0),
        ExplosionType::Thermal { .. } => Color::srgb(1.0, 0.4, 0.1),
        _ => Color::srgb(1.0, 0.6, 0.2),
    }
}

// New mini blast layer to replace MiniExplosion
fn update_mini_blast_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
) {
    if progress < 0.2 {
        let ring_particles = 6;
        for i in 0..ring_particles {
            let angle = (i as f32 / ring_particles as f32) * std::f32::consts::TAU;
            let radius = 15.0 + progress * 40.0;
            let velocity = Vec2::from_angle(angle) * (100.0 + progress * 80.0);
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: layer.color_start,
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 0.4,
                    size: 3.0,
                    fade_rate: 2.5,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Pulsing,
                },
            ));
        }
    }
}

fn get_explosion_color(explosion_type: &ExplosionType) -> Color {
    match explosion_type {
        ExplosionType::Biological { .. } => Color::srgb(0.6, 1.0, 0.8),
        ExplosionType::Chemical { .. } => Color::srgb(0.9, 0.9, 0.4),
        ExplosionType::Electrical { .. } => Color::srgb(0.4, 0.8, 1.0),
        ExplosionType::Thermal { .. } => Color::srgb(1.0, 0.6, 0.3),
        _ => Color::srgb(1.0, 0.8, 0.4),
    }
}

// Update create_explosion_layers to include MiniBlast for small explosions
fn create_explosion_layers(explosion_type: &ExplosionType, intensity: f32) -> Vec<ExplosionLayer> {
    let layers = match explosion_type {
        ExplosionType::Biological { toxin_release, membrane_rupture } => {
            let mut bio_layers = vec![
                ExplosionLayer {
                    phase: ExplosionPhase::Membrane,
                    delay: 0.0,
                    duration: 0.2,
                    particle_count: (25.0 * intensity) as u32,
                    color_start: Color::srgb(0.9, 1.0, 0.8),
                    color_end: Color::srgba(0.4, 0.8, 0.6, 0.0),
                    size_range: (2.0, 8.0),
                    velocity_range: (Vec2::new(-150.0, -150.0), Vec2::new(150.0, 150.0)),
                    completed: false,
                },
                ExplosionLayer {
                    phase: ExplosionPhase::CoreBlast,
                    delay: 0.05,
                    duration: 0.4,
                    particle_count: (40.0 * intensity) as u32,
                    color_start: Color::srgb(0.8, 0.9, 0.4),
                    color_end: Color::srgba(0.2, 0.6, 0.3, 0.0),
                    size_range: (1.0, 6.0),
                    velocity_range: (Vec2::new(-200.0, -200.0), Vec2::new(200.0, 200.0)),
                    completed: false,
                },
            ];
            
            if *toxin_release {
                bio_layers.push(ExplosionLayer {
                    phase: ExplosionPhase::Afterglow,
                    delay: 0.3,
                    duration: 1.2,
                    particle_count: (8.0 * intensity) as u32,
                    color_start: Color::srgb(0.9, 0.4, 0.6),
                    color_end: Color::srgba(0.7, 0.3, 0.4, 0.0),
                    size_range: (3.0, 12.0),
                    velocity_range: (Vec2::new(-50.0, -25.0), Vec2::new(50.0, 25.0)),
                    completed: false,
                });
            }
            
            bio_layers
        }
        _ => {
            // For small explosions, use MiniBlast phase
            if intensity < 1.0 {
                vec![
                    ExplosionLayer {
                        phase: ExplosionPhase::MiniBlast,
                        delay: 0.0,
                        duration: 0.3,
                        particle_count: (10.0 * intensity) as u32,
                        color_start: Color::srgb(1.0, 0.8, 0.4),
                        color_end: Color::srgba(1.0, 0.4, 0.2, 0.0),
                        size_range: (1.0, 4.0),
                        velocity_range: (Vec2::new(-120.0, -120.0), Vec2::new(120.0, 120.0)),
                        completed: false,
                    }
                ]
            } else {
                vec![
                    ExplosionLayer {
                        phase: ExplosionPhase::CoreBlast,
                        delay: 0.0,
                        duration: 0.5,
                        particle_count: (20.0 * intensity) as u32,
                        color_start: Color::srgb(1.0, 0.8, 0.4),
                        color_end: Color::srgba(1.0, 0.4, 0.2, 0.0),
                        size_range: (1.0, 5.0),
                        velocity_range: (Vec2::new(-180.0, -180.0), Vec2::new(180.0, 180.0)),
                        completed: false,
                    }
                ]
            }
        }
    };
    
    layers
}

// Layer update functions
fn update_shockwave_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
    explosion_type: &ExplosionType,
) {
    if progress < 0.1 { // Only spawn particles early in shockwave
        let ring_particles = 12;
        for i in 0..ring_particles {
            let angle = (i as f32 / ring_particles as f32) * std::f32::consts::TAU;
            let radius = 20.0 + progress * 100.0;
            let position = transform.translation + Vec3::new(
                angle.cos() * radius,
                angle.sin() * radius,
                0.1,
            );
            
            let velocity = Vec2::from_angle(angle) * 250.0;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: layer.color_start,
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                Transform::from_translation(position),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 0.3,
                    size: 3.0,
                    fade_rate: 3.0,
                    bioluminescent: matches!(explosion_type, ExplosionType::Biological { .. }),
                    drift_pattern: DriftPattern::Pulsing,
                },
            ));
        }
    }
}

fn update_core_blast_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
    intensity: f32,
) {
    if progress < 0.2 { // Spawn core particles early
        let count = (layer.particle_count as f32 * (1.0 - progress * 5.0)).max(0.0) as u32;
        
        for i in 0..count.min(8) { // Limit per frame
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU + progress * 10.0;
            let speed = 80.0 + progress * 120.0;
            let velocity = Vec2::from_angle(angle) * speed;
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: layer.color_start,
                    custom_size: Some(Vec2::splat(4.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 0.6,
                    size: 4.0 * intensity,
                    fade_rate: 1.5,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Spiraling,
                },
                BioluminescentParticle {
                    base_color: layer.color_start,
                    pulse_frequency: 6.0,
                    pulse_intensity: 0.8,
                    organic_motion: OrganicMotion {
                        undulation_speed: 3.0,
                        response_to_current: 0.2,
                    },
                },
            ));
        }
    }
}

fn update_debris_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
    explosion_type: &ExplosionType,
) {
    if progress < 0.3 {
        let debris_color = match explosion_type {
            ExplosionType::Biological { .. } => Color::srgb(0.7, 0.8, 0.6),
            ExplosionType::Chemical { .. } => Color::srgb(0.8, 0.9, 0.5),
            _ => Color::srgb(0.6, 0.6, 0.6),
        };
        
        for i in 0..(layer.particle_count / 8).min(4) {
            let angle = (i as f32 * 1.7) + progress * 8.0;
            let distance = 25.0 + progress * 40.0;
            let velocity = Vec2::from_angle(angle) * (60.0 + progress * 80.0);
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: debris_color,
                    custom_size: Some(Vec2::splat(2.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation + Vec3::new(
                    angle.cos() * distance,
                    angle.sin() * distance,
                    0.0,
                )),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 1.5,
                    size: 2.0,
                    fade_rate: 0.8,
                    bioluminescent: false,
                    drift_pattern: DriftPattern::Brownian,
                },
            ));
        }
    }
}

fn update_afterglow_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
) {
    if progress < 0.4 {
        for i in 0..(layer.particle_count / 10).min(3) {
            let velocity = Vec2::new(
                (progress * 50.0 + i as f32 * 20.0).sin() * 30.0,
                (progress * 40.0 + i as f32 * 15.0).cos() * 25.0,
            );
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: layer.color_start,
                    custom_size: Some(Vec2::splat(6.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 3.0,
                    size: 6.0,
                    fade_rate: 0.4,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Floating,
                },
                BioluminescentParticle {
                    base_color: layer.color_start,
                    pulse_frequency: 1.0,
                    pulse_intensity: 0.6,
                    organic_motion: OrganicMotion {
                        undulation_speed: 1.5,
                        response_to_current: 0.9,
                    },
                },
            ));
        }
    }
}

fn update_membrane_layer(
    commands: &mut Commands,
    assets: &GameAssets,
    transform: &Transform,
    layer: &ExplosionLayer,
    progress: f32,
) {
    if progress < 0.15 {
        // Membrane fragments with organic shapes
        for i in 0..6 {
            let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
            let fragment_size = 4.0 + (i % 3) as f32 * 2.0;
            let velocity = Vec2::from_angle(angle) * (120.0 + progress * 100.0);
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgb(0.8, 1.0, 0.9),
                    custom_size: Some(Vec2::splat(fragment_size)),
                    ..default()
                },
                Transform::from_translation(transform.translation + Vec3::new(
                    angle.cos() * 15.0,
                    angle.sin() * 15.0,
                    0.1,
                )),
                Particle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: 1.0,
                    size: fragment_size,
                    fade_rate: 1.2,
                    bioluminescent: true,
                    drift_pattern: DriftPattern::Floating,
                },
                BioluminescentParticle {
                    base_color: Color::srgb(0.8, 1.0, 0.9),
                    pulse_frequency: 4.0,
                    pulse_intensity: 0.4,
                    organic_motion: OrganicMotion {
                        undulation_speed: 2.5,
                        response_to_current: 0.8,
                    },
                },
            ));
        }
    }
}

// Calculate screen shake based on explosion properties
fn calculate_shake_amount(explosion_type: &ExplosionType, intensity: f32) -> f32 {
    let base_shake = intensity * 0.3;
    
    match explosion_type {
        ExplosionType::Biological { membrane_rupture: true, .. } => base_shake * 1.5,
        ExplosionType::Chemical { .. } => base_shake * 1.2,
        ExplosionType::Electrical { .. } => base_shake * 0.8,
        ExplosionType::Thermal { .. } => base_shake * 1.3,
        _ => base_shake,
    }
}


// Event Systems with biological enhancements
pub fn spawn_explosion_system(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosion>,
    mut shake_events: EventWriter<AddScreenShake>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = assets {
        for event in explosion_events.read() {
            let explosion_type = match &event.enemy_type {
                Some(EnemyType::InfectedMacrophage) => ExplosionType::Biological { 
                    toxin_release: true, 
                    membrane_rupture: true 
                },
                Some(EnemyType::BiofilmColony) => ExplosionType::Chemical { 
                    ph_change: -1.5, 
                    oxygen_release: 0.3 
                },
                Some(EnemyType::AggressiveBacteria) => ExplosionType::Biological { 
                    toxin_release: true, 
                    membrane_rupture: false 
                },
                _ => ExplosionType::Standard,
            };
            
            let layers = create_explosion_layers(&explosion_type, event.intensity);
            let shake_amount = calculate_shake_amount(&explosion_type, event.intensity);
            
            shake_events.write(AddScreenShake { amount: shake_amount });
            
            // Spawn main explosion entity
            let explosion_entity = commands.spawn((
                Sprite {
                    image: assets.explosion_texture.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0), // Start transparent
                    ..default()
                },
                Transform::from_translation(event.position),
                Explosion {
                    timer: 0.0,
                    max_time: 1.5,
                    intensity: event.intensity,
                    explosion_type: explosion_type.clone(),
                    layers,
                    current_layer_index: 0,
                },
            )).id();
            
            // Create initial shockwave
            spawn_shockwave(&mut commands, &assets, event.position, event.intensity, &explosion_type);
        }
    }
}


// Spawn shockwave effect
fn spawn_shockwave(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    intensity: f32,
    explosion_type: &ExplosionType,
) {
    let ring_color = match explosion_type {
        ExplosionType::Biological { .. } => Color::srgb(0.4, 1.0, 0.8),
        ExplosionType::Chemical { .. } => Color::srgb(0.9, 0.9, 0.3),
        ExplosionType::Electrical { .. } => Color::srgb(0.3, 0.8, 1.0),
        _ => Color::srgb(1.0, 0.8, 0.4),
    };
    
    let layers = create_explosion_layers(&explosion_type, intensity);

    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgba(ring_color.to_srgba().red, ring_color.to_srgba().green, ring_color.to_srgba().blue, 0.6),
            custom_size: Some(Vec2::splat(20.0)),
            ..default()
        },
        Transform::from_translation(position),
        Explosion {
            timer: 0.0,
            max_time: 0.4,
            intensity,
            explosion_type: explosion_type.clone(),
            layers,
            current_layer_index: 0,
        },
    ));
}
