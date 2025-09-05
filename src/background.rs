use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::despawn::*;
use std::f32::consts::TAU;

// New background system components
#[derive(Component)]
pub struct ProceduralBackgroundTile {
    pub tile_type: BackgroundTileType,
    pub depth_layer: f32,
    pub parallax_speed: f32,
    pub generation_seed: u32,
    pub tile_size: Vec2,
    pub world_position: Vec2,
    pub next_spawn_y: f32,
}

#[derive(Component)]
pub struct BackgroundParticle {
    pub particle_type: BackgroundParticleType,
    pub depth: f32,
    pub drift_velocity: Vec2,
    pub rotation_speed: f32,
    pub scale_variation: f32,
    pub lifecycle_timer: f32,
    pub max_lifetime: f32,
}

#[derive(Clone)]
pub enum BackgroundTileType {
    // Deep layer (furthest back, slowest parallax)
    RockySeafloor { 
        coral_density: f32,
        thermal_vents: Vec<Vec2>,
        rock_formations: Vec<SeafloorRock>,
    },
    CoralGarden {
        coral_types: Vec<CoralFormation>,
        biodiversity_level: f32,
        health_status: f32,
    },
    
    // Mid layer (medium parallax)
    OpenWater {
        plankton_density: f32,
        debris_scattered: Vec<DebrisType>,
        chemical_gradients: Vec<ChemicalGradient>,
    },
    KelpForest {
        kelp_strands: Vec<KelpStrand>,
        current_sway: f32,
    },
    
    // Surface layer (fastest parallax)
    SurfaceWater {
        light_caustics: CausticPattern,
        bubble_streams: Vec<BubbleStream>,
        surface_tension_effects: f32,
    },
    ContaminatedZone {
        pollution_type: ContaminationType,
        visibility_reduction: f32,
        toxic_particles: u32,
    },
}

#[derive(Clone)]
pub enum BackgroundParticleType {
    // Deep water particles
    Microorganism { species: String, bioluminescent: bool },
    Sediment { size: f32, density: f32 },
    NutrientParticle { concentration: f32 },
    
    // Mid water particles
    Plankton { bloom_type: PlanktonType, cluster_size: u32 },
    DeepSeaSnow { organic_content: f32 },
    ChemicalGradient { ph_level: f32, opacity: f32 },
    
    // Surface particles
    Bubble { size: f32, rise_speed: f32 },
    LightRay { caustic_intensity: f32, wave_frequency: f32 },
    FloatingDebris { material: String, buoyancy: f32 },
}

#[derive(Resource)]
pub struct ProceduralBackgroundManager {
    pub active_tiles: Vec<Entity>,
    pub tile_pool: Vec<Entity>,
    pub generation_distance: f32,
    pub cleanup_distance: f32,
    pub current_depth_focus: f32, // For depth of field
    pub environmental_state: EnvironmentalConditions,
    pub tile_generation_seed: u32,
}

#[derive(Clone)]
pub struct EnvironmentalConditions {
    pub water_clarity: f32,
    pub current_strength: f32,
    pub ecosystem_health: f32,
    pub contamination_level: f32,
    pub tidal_phase: TidePhase,
    pub light_penetration: f32,
}

// Detailed sub-structures
#[derive(Clone)]
pub struct SeafloorRock {
    pub position: Vec2,
    pub size: Vec2,
    pub rock_type: RockType,
    pub covered_in_life: bool,
}

#[derive(Clone)]
pub enum RockType {
    Basalt,
    Limestone,
    Coral,
    Volcanic,
}

#[derive(Clone)]
pub struct CoralFormation {
    pub position: Vec2,
    pub coral_type: CoralSpecies,
    pub health: f32,
    pub size: f32,
    pub bioluminescence: f32,
}

#[derive(Clone)]
pub enum CoralSpecies {
    BranchingCoral,
    TableCoral,
    BrainCoral,
    SoftCoral,
    DeepSeaCoral,
}

#[derive(Clone)]
pub struct KelpStrand {
    pub base_position: Vec2,
    pub segments: Vec<Vec2>,
    pub sway_amplitude: f32,
    pub length: f32,
}

#[derive(Clone)]
pub struct CausticPattern {
    pub wave_frequency: f32,
    pub intensity: f32,
    pub direction: Vec2,
    pub color_shift: f32,
}

#[derive(Clone)]
pub struct BubbleStream {
    pub source_position: Vec2,
    pub bubble_rate: f32,
    pub stream_angle: f32,
    pub dispersion: f32,
}

#[derive(Clone)]
pub struct ChemicalGradient {
    pub center: Vec2,
    pub radius: f32,
    pub chemical_type: String,
    pub concentration: f32,
    pub color: Color,
}

#[derive(Clone)]
pub enum PlanktonType {
    Diatoms,
    Dinoflagellates,
    Copepods,
    Krill,
    Bioluminescent,
}

impl Default for ProceduralBackgroundManager {
    fn default() -> Self {
        Self {
            active_tiles: Vec::new(),
            tile_pool: Vec::new(),
            generation_distance: 800.0,
            cleanup_distance: 1000.0,
            current_depth_focus: 1.0,
            environmental_state: EnvironmentalConditions {
                water_clarity: 0.8,
                current_strength: 1.0,
                ecosystem_health: 0.7,
                contamination_level: 0.2,
                tidal_phase: TidePhase::Rising,
                light_penetration: 0.6,
            },
            tile_generation_seed: 12345,
        }
    }
}

// Main procedural generation system
pub fn procedural_background_generation(
    mut commands: Commands,
    mut bg_manager: ResMut<ProceduralBackgroundManager>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<ProceduralBackgroundTile>)>,
    tile_query: Query<(Entity, &Transform, &ProceduralBackgroundTile), Without<Camera2d>>,
    ecosystem: Res<EcosystemState>,
    tidal_physics: Res<TidalPoolPhysics>,
    chemical_environment: Res<ChemicalEnvironment>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut initial_generation: Local<bool>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        if let Some(assets) = assets {
            // Update environmental conditions
            bg_manager.environmental_state.ecosystem_health = ecosystem.health;
            bg_manager.environmental_state.contamination_level = 1.0 - ecosystem.health;
            bg_manager.environmental_state.current_strength = tidal_physics.current_strength;
            bg_manager.environmental_state.water_clarity = 0.9 - ecosystem.infection_level * 0.4;
            
            let camera_y = camera_transform.translation.y;
            
            // Initial generation - populate entire view on first frame
            if !*initial_generation {
                *initial_generation = true;
                
                // Generate tiles across the entire visible area plus buffer
                for y in -2..=3 {
                    let tile_y = camera_y + (y as f32 * 200.0);
                    for depth_layer in [0.1, 0.3, 0.5, 0.7, 0.8] { // More layers for richness
                        generate_background_tile(
                            &mut commands,
                            &mut bg_manager,
                            &assets,
                            depth_layer,
                            tile_y,
                            time.elapsed_secs() + y as f32,
                        );
                    }
                }
            }
            
            // Normal generation - only ahead of camera
            let generation_threshold = camera_y + bg_manager.generation_distance;
            
            // Check each depth layer for tile generation needs
            for depth_layer in [0.1, 0.5, 0.8] { // Deep, Mid, Surface
                if should_generate_tile_at_depth(depth_layer, generation_threshold, &tile_query) {
                    generate_background_tile(
                        &mut commands,
                        &mut bg_manager,
                        &assets,
                        depth_layer,
                        generation_threshold,
                        time.elapsed_secs(),
                    );
                }
            }
            
            // Cleanup distant tiles
            cleanup_distant_tiles(&mut commands, &mut bg_manager, camera_y);
        }
    }
}

fn should_generate_tile_at_depth(
    depth: f32,
    y_threshold: f32,
    tile_query: &Query<(Entity, &Transform, &ProceduralBackgroundTile), Without<Camera2d>>,
) -> bool {
    let depth_tolerance = 0.1;
    
    for (_, transform, tile) in tile_query.iter() {
        if (tile.depth_layer - depth).abs() < depth_tolerance 
            && transform.translation.y > y_threshold - 100.0 {
            return false; // Already have a tile at this depth near the threshold
        }
    }
    
    true
}

fn generate_background_tile(
    commands: &mut Commands,
    bg_manager: &mut ProceduralBackgroundManager,
    assets: &GameAssets,
    depth_layer: f32,
    y_position: f32,
    time_seed: f32,
) {
    let seed = (bg_manager.tile_generation_seed as f32 + time_seed * 1000.0) as u32;
    bg_manager.tile_generation_seed = bg_manager.tile_generation_seed.wrapping_add(1);
    
    let tile_type = generate_tile_type_for_depth(depth_layer, &bg_manager.environmental_state, seed);
    let parallax_speed = calculate_parallax_speed(depth_layer);
    
    let tile_entity = spawn_background_tile(commands, assets, &tile_type, depth_layer, y_position, parallax_speed);
    bg_manager.active_tiles.push(tile_entity);
    
    // Generate associated particles
    spawn_tile_particles(commands, assets, &tile_type, Vec2::new(0.0, y_position), depth_layer);
}

fn generate_tile_type_for_depth(
    depth: f32,
    conditions: &EnvironmentalConditions,
    seed: u32,
) -> BackgroundTileType {
    let random = (seed as f32 * 0.001).sin().abs();
    
    match depth {
        d if d < 0.3 => {
            // Deep layer
            if random < 0.6 {
                BackgroundTileType::RockySeafloor {
                    coral_density: conditions.ecosystem_health * 0.8,
                    thermal_vents: generate_thermal_vents(seed, 2),
                    rock_formations: generate_rock_formations(seed, 5),
                }
            } else {
                BackgroundTileType::CoralGarden {
                    coral_types: generate_coral_formations(seed, conditions.ecosystem_health),
                    biodiversity_level: conditions.ecosystem_health,
                    health_status: conditions.ecosystem_health,
                }
            }
        }
        d if d < 0.7 => {
            // Mid layer
            if random < 0.7 {
                BackgroundTileType::OpenWater {
                    plankton_density: conditions.ecosystem_health * 0.6,
                    debris_scattered: generate_debris_for_conditions(conditions),
                    chemical_gradients: generate_chemical_gradients(seed),
                }
            } else {
                BackgroundTileType::KelpForest {
                    kelp_strands: generate_kelp_strands(seed, conditions.current_strength),
                    current_sway: conditions.current_strength,
                }
            }
        }
        _ => {
            // Surface layer
            if conditions.contamination_level > 0.5 {
                BackgroundTileType::ContaminatedZone {
                    pollution_type: ContaminationType::PlasticPollution,
                    visibility_reduction: conditions.contamination_level * 0.3,
                    toxic_particles: (conditions.contamination_level * 20.0) as u32,
                }
            } else {
                BackgroundTileType::SurfaceWater {
                    light_caustics: CausticPattern {
                        wave_frequency: 2.0,
                        intensity: conditions.light_penetration,
                        direction: Vec2::new(1.0, 0.0),
                        color_shift: 0.0,
                    },
                    bubble_streams: generate_bubble_streams(seed),
                    surface_tension_effects: 1.0,
                }
            }
        }
    }
}

fn spawn_background_tile(
    commands: &mut Commands,
    assets: &GameAssets,
    tile_type: &BackgroundTileType,
    depth: f32,
    y_pos: f32,
    parallax_speed: f32,
) -> Entity {
    let (texture, color, size) = match tile_type {
        BackgroundTileType::RockySeafloor { coral_density, .. } => {
            (assets.enemy_texture.clone(), 
             Color::srgb(0.3 + coral_density * 0.2, 0.2 + coral_density * 0.3, 0.2), 
             Vec2::new(1280.0, 400.0))
        }
        BackgroundTileType::CoralGarden { health_status, .. } => {
            (assets.enemy_texture.clone(),
             Color::srgb(0.4, 0.3 + health_status * 0.4, 0.3 + health_status * 0.5),
             Vec2::new(800.0, 300.0))
        }
        BackgroundTileType::OpenWater { plankton_density, .. } => {
            (assets.particle_texture.clone(),
             Color::srgba(0.1, 0.3, 0.5 + plankton_density * 0.2, 0.3),
             Vec2::new(1280.0, 600.0))
        }
        BackgroundTileType::KelpForest { .. } => {
            (assets.projectile_texture.clone(),
             Color::srgb(0.2, 0.4, 0.2),
             Vec2::new(400.0, 800.0))
        }
        BackgroundTileType::SurfaceWater { light_caustics, .. } => {
            (assets.particle_texture.clone(),
             Color::srgba(0.4, 0.6, 0.8, 0.2 + light_caustics.intensity * 0.3),
             Vec2::new(1280.0, 200.0))
        }
        BackgroundTileType::ContaminatedZone { visibility_reduction, .. } => {
            (assets.explosion_texture.clone(),
             Color::srgba(0.6, 0.4, 0.3, 0.4 + visibility_reduction),
             Vec2::new(1280.0, 400.0))
        }
    };
    
    commands.spawn((
        Sprite {
            image: texture,
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(0.0, y_pos, -depth),
        ProceduralBackgroundTile {
            tile_type: tile_type.clone(),
            depth_layer: depth,
            parallax_speed,
            generation_seed: 0, // Set by generator
            tile_size: size,
            world_position: Vec2::new(0.0, y_pos),
            next_spawn_y: y_pos + size.y,
        },
        ParallaxLayer { speed: parallax_speed, depth: -depth },
    )).id()
}

fn calculate_parallax_speed(depth: f32) -> f32 {
    match depth {
        d if d < 0.3 => 0.05, // Deep layer moves very slowly
        d if d < 0.7 => 0.25, // Mid layer moves at medium speed  
        _ => 0.6, // Surface layer moves faster
    }
}

// Particle generation functions
fn spawn_tile_particles(
    commands: &mut Commands,
    assets: &GameAssets,
    tile_type: &BackgroundTileType,
    position: Vec2,
    depth: f32,
) {
    match tile_type {
        BackgroundTileType::OpenWater { plankton_density, .. } => {
            spawn_plankton_particles(commands, assets, position, *plankton_density, depth);
        }
        BackgroundTileType::SurfaceWater { bubble_streams, .. } => {
            spawn_bubble_particles(commands, assets, position, bubble_streams, depth);
        }
        BackgroundTileType::ContaminatedZone { toxic_particles, .. } => {
            spawn_contamination_particles(commands, assets, position, *toxic_particles, depth);
        }
        _ => {} // Other tile types don't need immediate particle spawning
    }
}

fn spawn_plankton_particles(
    commands: &mut Commands,
    assets: &GameAssets,
    base_position: Vec2,
    density: f32,
    depth: f32,
) {
    let particle_count = (density * 15.0) as u32;
    
    for i in 0..particle_count {
        let offset = Vec2::new(
            (i as f32 * 123.45).sin() * 400.0,
            (i as f32 * 67.89).cos() * 200.0,
        );
        
        commands.spawn((
            Sprite {
                image: assets.particle_texture.clone(),
                color: Color::srgba(0.8, 1.0, 0.9, 0.6),
                custom_size: Some(Vec2::splat(2.0)),
                ..default()
            },
            Transform::from_translation((base_position + offset).extend(-depth + 0.01)),
            BackgroundParticle {
                particle_type: BackgroundParticleType::Plankton {
                    bloom_type: PlanktonType::Diatoms,
                    cluster_size: 1,
                },
                depth,
                drift_velocity: Vec2::new(
                    (i as f32 * 45.67).sin() * 10.0,
                    -20.0,
                ),
                rotation_speed: 0.5,
                scale_variation: 0.2,
                lifecycle_timer: 0.0,
                max_lifetime: 30.0,
            },
            BioluminescentParticle {
                base_color: Color::srgb(0.8, 1.0, 0.9),
                pulse_frequency: 1.0 + (i as f32 * 0.1),
                pulse_intensity: 0.3,
                organic_motion: OrganicMotion {
                    undulation_speed: 1.0,
                    response_to_current: 0.8,
                },
            },
        ));
    }
}

fn spawn_bubble_particles(
    commands: &mut Commands,
    assets: &GameAssets,
    base_position: Vec2,
    bubble_streams: &[BubbleStream],
    depth: f32,
) {
    for stream in bubble_streams {
        for i in 0..5 {
            let bubble_pos = stream.source_position + Vec2::new(
                (i as f32 * stream.dispersion).sin() * 20.0,
                i as f32 * 15.0,
            );
            
            commands.spawn((
                Sprite {
                    image: assets.particle_texture.clone(),
                    color: Color::srgba(0.9, 0.95, 1.0, 0.4),
                    custom_size: Some(Vec2::splat(3.0 + i as f32 * 0.5)),
                    ..default()
                },
                Transform::from_translation((base_position + bubble_pos).extend(-depth + 0.02)),
                BackgroundParticle {
                    particle_type: BackgroundParticleType::Bubble {
                        size: 3.0 + i as f32 * 0.5,
                        rise_speed: 80.0 + i as f32 * 10.0,
                    },
                    depth,
                    drift_velocity: Vec2::new(0.0, 80.0 + i as f32 * 10.0),
                    rotation_speed: 0.0,
                    scale_variation: 0.1,
                    lifecycle_timer: 0.0,
                    max_lifetime: 8.0,
                },
            ));
        }
    }
}

fn spawn_contamination_particles(
    commands: &mut Commands,
    assets: &GameAssets,
    base_position: Vec2,
    particle_count: u32,
    depth: f32,
) {
    for i in 0..particle_count {
        let offset = Vec2::new(
            (i as f32 * 234.56).sin() * 300.0,
            (i as f32 * 78.90).cos() * 150.0,
        );
        
        commands.spawn((
            Sprite {
                image: assets.particle_texture.clone(),
                color: Color::srgba(0.7, 0.5, 0.3, 0.8),
                custom_size: Some(Vec2::splat(4.0)),
                ..default()
            },
            Transform::from_translation((base_position + offset).extend(-depth + 0.01)),
            BackgroundParticle {
                particle_type: BackgroundParticleType::FloatingDebris {
                    material: "Plastic".to_string(),
                    buoyancy: 0.3,
                },
                depth,
                drift_velocity: Vec2::new(
                    (i as f32 * 12.34).sin() * 15.0,
                    -25.0,
                ),
                rotation_speed: 1.0,
                scale_variation: 0.3,
                lifecycle_timer: 0.0,
                max_lifetime: 45.0,
            },
        ));
    }
}

// Update system for background particles
pub fn update_background_particles(
    mut commands: Commands,
    mut particle_query: Query<(
        Entity,
        &mut Transform,
        &mut BackgroundParticle,
        &mut Sprite,
        Option<&mut BioluminescentParticle>
    ), Without<PendingDespawn>>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite, bio_particle) in particle_query.iter_mut() {
        particle.lifecycle_timer += time.delta_secs();
        
        if particle.lifecycle_timer >= particle.max_lifetime {
            commands.entity(entity)
                .safe_despawn();
            continue;
        }
        
        // Update position based on particle type
        match &particle.particle_type {
            BackgroundParticleType::Bubble { rise_speed, .. } => {
                transform.translation.y += rise_speed * time.delta_secs();
                // Bubbles fade as they rise
                let fade = 1.0 - (particle.lifecycle_timer / particle.max_lifetime);
                sprite.color.set_alpha(fade * 0.4);
            }
            
            BackgroundParticleType::Plankton { .. } => {
                // Plankton drift with current + organic motion
                let organic_drift = Vec2::new(
                    (time.elapsed_secs() * 2.0 + transform.translation.x * 0.01).sin() * 5.0,
                    (time.elapsed_secs() * 1.5 + transform.translation.y * 0.01).cos() * 3.0,
                );
                transform.translation += (particle.drift_velocity + organic_drift).extend(0.0) * time.delta_secs();
            }
            
            BackgroundParticleType::FloatingDebris { .. } => {
                transform.translation += particle.drift_velocity.extend(0.0) * time.delta_secs();
                transform.rotation *= Quat::from_rotation_z(particle.rotation_speed * time.delta_secs());
            }
            
            _ => {
                transform.translation += particle.drift_velocity.extend(0.0) * time.delta_secs();
            }
        }
        
        // Scale variation for organic feel
        let scale_pulse = (time.elapsed_secs() * 3.0 + particle.lifecycle_timer).sin();
        let scale = 1.0 + scale_pulse * particle.scale_variation;
        transform.scale = Vec3::splat(scale);
    }
}

fn cleanup_distant_tiles(
    commands: &mut Commands,
    bg_manager: &mut ProceduralBackgroundManager,
    camera_y: f32,
) {
    let cleanup_threshold = camera_y - bg_manager.cleanup_distance;
    
    bg_manager.active_tiles.retain(|&entity| {
        // In a real implementation, you'd check the tile's position
        // For now, we'll assume tiles below the threshold should be cleaned up
        true // Placeholder - implement actual position checking
    });
}

// Helper functions for generating tile content
fn generate_thermal_vents(_seed: u32, count: u32) -> Vec<Vec2> {
    let mut vents = Vec::new();
    for i in 0..count {
        vents.push(Vec2::new(
            (i as f32 * 200.0) - 400.0,
            (i as f32 * 50.0).sin() * 100.0,
        ));
    }
    vents
}

fn generate_rock_formations(_seed: u32, count: u32) -> Vec<SeafloorRock> {
    let mut rocks = Vec::new();
    for i in 0..count {
        rocks.push(SeafloorRock {
            position: Vec2::new(
                (i as f32 * 150.0) - 300.0,
                (i as f32 * 30.0).cos() * 50.0,
            ),
            size: Vec2::new(40.0 + i as f32 * 10.0, 30.0 + i as f32 * 5.0),
            rock_type: match i % 4 {
                0 => RockType::Basalt,
                1 => RockType::Limestone,
                2 => RockType::Coral,
                _ => RockType::Volcanic,
            },
            covered_in_life: i % 3 == 0,
        });
    }
    rocks
}

fn generate_coral_formations(_seed: u32, health: f32) -> Vec<CoralFormation> {
    let mut corals = Vec::new();
    let count = (health * 8.0) as u32;
    
    for i in 0..count {
        corals.push(CoralFormation {
            position: Vec2::new(
                (i as f32 * 100.0) - 200.0,
                (i as f32 * 20.0).sin() * 40.0,
            ),
            coral_type: match i % 5 {
                0 => CoralSpecies::BranchingCoral,
                1 => CoralSpecies::TableCoral,
                2 => CoralSpecies::BrainCoral,
                3 => CoralSpecies::SoftCoral,
                _ => CoralSpecies::DeepSeaCoral,
            },
            health,
            size: 20.0 + health * 30.0,
            bioluminescence: if health > 0.7 { 0.8 } else { 0.2 },
        });
    }
    corals
}

fn generate_kelp_strands(_seed: u32, current_strength: f32) -> Vec<KelpStrand> {
    let mut kelp = Vec::new();
    let count = 6;
    
    for i in 0..count {
        let mut segments = Vec::new();
        let base_pos = Vec2::new((i as f32 * 80.0) - 240.0, -100.0);
        
        // Generate kelp segments
        for j in 0..12 {
            let segment_y = base_pos.y + j as f32 * 60.0;
            let sway = (j as f32 * 0.2).sin() * current_strength * 15.0;
            segments.push(Vec2::new(base_pos.x + sway, segment_y));
        }
        
        kelp.push(KelpStrand {
            base_position: base_pos,
            segments,
            sway_amplitude: current_strength * 20.0,
            length: 720.0,
        });
    }
    kelp
}

fn generate_bubble_streams(_seed: u32) -> Vec<BubbleStream> {
    let mut streams = Vec::new();
    for i in 0..4 {
        streams.push(BubbleStream {
            source_position: Vec2::new(
                (i as f32 * 160.0) - 240.0,
                -80.0,
            ),
            bubble_rate: 2.0 + i as f32 * 0.5,
            stream_angle: (i as f32 * 0.2).sin() * 0.3,
            dispersion: 15.0 + i as f32 * 5.0,
        });
    }
    streams
}

fn generate_debris_for_conditions(conditions: &EnvironmentalConditions) -> Vec<DebrisType> {
    let mut debris = Vec::new();
    
    if conditions.contamination_level > 0.3 {
        debris.push(DebrisType::PlasticFragment { 
            size: 3.0, 
            color: Color::srgb(0.8, 0.8, 0.9) 
        });
        debris.push(DebrisType::ChemicalResidue { 
            compound_type: "Industrial".to_string() 
        });
    }
    
    if conditions.ecosystem_health < 0.5 {
        debris.push(DebrisType::BiologicalRemains { 
            species: "Dead Plankton".to_string(), 
            decay_level: 0.8 
        });
    }
    
    debris
}

fn generate_chemical_gradients(_seed: u32) -> Vec<ChemicalGradient> {
    let mut gradients = Vec::new();
    
    for i in 0..3 {
        gradients.push(ChemicalGradient {
            center: Vec2::new(
                (i as f32 * 200.0) - 200.0,
                (i as f32 * 60.0).sin() * 80.0,
            ),
            radius: 60.0 + i as f32 * 20.0,
            chemical_type: match i {
                0 => "Nutrients".to_string(),
                1 => "pH Gradient".to_string(),
                _ => "Oxygen Zone".to_string(),
            },
            concentration: 0.5 + i as f32 * 0.2,
            color: match i {
                0 => Color::srgba(0.3, 0.8, 0.3, 0.3),
                1 => Color::srgba(0.8, 0.8, 0.3, 0.3),
                _ => Color::srgba(0.3, 0.3, 0.8, 0.3),
            },
        });
    }
    gradients
}

// Depth of Field System for future camera enhancement
#[derive(Component)]
pub struct DepthOfFieldTarget {
    pub focus_depth: f32,
    pub aperture: f32,
    pub focal_length: f32,
}

pub fn update_depth_of_field_focus(
    mut bg_manager: ResMut<ProceduralBackgroundManager>,
    camera_query: Query<&DepthOfFieldTarget>,
    mut tile_query: Query<(&ProceduralBackgroundTile, &mut Sprite)>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(dof_target) = camera_query.single() {
        bg_manager.current_depth_focus = dof_target.focus_depth;
        
        // Adjust tile opacity/blur based on depth of field
        for (tile, mut sprite) in tile_query.iter_mut() {
            let depth_difference = (tile.depth_layer - dof_target.focus_depth).abs();
            let blur_factor = (depth_difference / dof_target.aperture).clamp(0.0, 1.0);
            
            // Simulate depth of field by adjusting alpha and saturation
            let focus_alpha = 1.0 - blur_factor * 0.6;
            let current_color = sprite.color;
            
            sprite.color = Color::srgba(
                current_color.to_srgba().red * (1.0 - blur_factor * 0.3),
                current_color.to_srgba().green * (1.0 - blur_factor * 0.3),
                current_color.to_srgba().blue * (1.0 - blur_factor * 0.3),
                current_color.to_srgba().alpha * focus_alpha,
            );
        }
    }
}

// Enhanced parallax system that responds to movement
pub fn enhanced_parallax_system(
    mut tile_query: Query<(&mut Transform, &ProceduralBackgroundTile), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<Player>, Without<ProceduralBackgroundTile>)>,
    time: Res<Time>,
) {
    if let (Ok(player_transform), Ok(camera_transform)) = (player_query.single(), camera_query.single()) {
        let player_velocity = Vec2::ZERO; // Would track velocity in real implementation
        
        for (mut tile_transform, tile) in tile_query.iter_mut() {
            // Base parallax movement
            tile_transform.translation.y -= tile.parallax_speed * 100.0 * time.delta_secs();
            
            // Respond to player movement with layered parallax
            let movement_response = player_velocity * tile.parallax_speed * 0.1;
            tile_transform.translation.x += movement_response.x * time.delta_secs();
            
            // Organic tile movement based on type
            match &tile.tile_type {
                BackgroundTileType::KelpForest { current_sway, .. } => {
                    let sway = (time.elapsed_secs() * 0.5 + tile_transform.translation.x * 0.001).sin();
                    tile_transform.translation.x += sway * current_sway * 5.0 * time.delta_secs();
                }
                BackgroundTileType::SurfaceWater { light_caustics, .. } => {
                    let caustic_wave = (time.elapsed_secs() * light_caustics.wave_frequency).sin();
                    tile_transform.translation.y += caustic_wave * light_caustics.intensity * 2.0 * time.delta_secs();
                }
                _ => {}
            }
            
            // Reset tiles that moved too far down
            if tile_transform.translation.y < -800.0 {
                tile_transform.translation.y = 800.0;
            }
        }
    }
}

// System to provide biological feedback
pub fn biological_feedback_system(
    mut commands: Commands,
    ecosystem: Res<EcosystemState>,
    tidal_physics: Res<TidalPoolPhysics>,
    chemical_environment: Res<ChemicalEnvironment>,
    player_query: Query<&Transform, With<Player>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut feedback_timer: Local<f32>,
) {
    *feedback_timer += time.delta_secs();
    
    if *feedback_timer >= 2.0 {
        *feedback_timer = 0.0;
        
        if let (Ok(player_transform), Some(assets)) = (player_query.single(), assets) {
            // Ecosystem health feedback
            if ecosystem.health < 0.3 {
                spawn_warning_indicator(&mut commands, &assets, 
                    player_transform.translation + Vec3::new(-50.0, 30.0, 1.0),
                    "ECOSYSTEM CRITICAL", Color::srgb(1.0, 0.3, 0.3));
            }
            
            // King tide warning
            if tidal_physics.king_tide_active {
                spawn_environmental_effect(&mut commands, &assets,
                    player_transform.translation + Vec3::new(0.0, 50.0, 1.0),
                    "KING TIDE ACTIVE", Color::srgb(0.3, 0.6, 1.0));
            }
            
            // Chemical contamination feedback
            let avg_ph = chemical_environment.ph_zones.iter()
                .map(|z| z.ph_level)
                .sum::<f32>() / chemical_environment.ph_zones.len().max(1) as f32;
                
            if avg_ph < 5.5 || avg_ph > 8.5 {
                spawn_chemical_indicator(&mut commands, &assets,
                    player_transform.translation + Vec3::new(50.0, 30.0, 1.0),
                    &format!("pH: {:.1}", avg_ph),
                    if avg_ph < 6.0 { Color::srgb(1.0, 0.4, 0.4) } else { Color::srgb(0.4, 0.4, 1.0) });
            }
        }
    }
}

fn spawn_warning_indicator(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    text: &str,
    color: Color,
) {
    commands.spawn((
        Sprite {
            image: assets.particle_texture.clone(),
            color,
            custom_size: Some(Vec2::splat(8.0)),
            ..default()
        },
        Transform::from_translation(position),
        Particle {
            velocity: Vec2::new(0.0, 20.0),
            lifetime: 0.0,
            max_lifetime: 3.0,
            size: 8.0,
            fade_rate: 0.8,
            bioluminescent: true,
            drift_pattern: DriftPattern::Pulsing,
        },
    ));
}

fn spawn_environmental_effect(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    text: &str,
    color: Color,
) {
    // Create ripple effect for tidal feedback
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * TAU;
        let radius = 20.0 + i as f32 * 10.0;
        let offset = Vec2::from_angle(angle) * radius;
        
        commands.spawn((
            Sprite {
                image: assets.particle_texture.clone(),
                color: Color::srgba(color.to_srgba().red, color.to_srgba().green, color.to_srgba().blue, 0.6),
                custom_size: Some(Vec2::splat(4.0)),
                ..default()
            },
            Transform::from_translation(position + offset.extend(0.0)),
            Particle {
                velocity: offset.normalize() * 30.0,
                lifetime: 0.0,
                max_lifetime: 2.0,
                size: 4.0,
                fade_rate: 1.0,
                bioluminescent: true,
                drift_pattern: DriftPattern::Floating,
            },
        ));
    }
}

fn spawn_chemical_indicator(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    text: &str,
    color: Color,
) {
    commands.spawn((
        Sprite {
            image: assets.explosion_texture.clone(),
            color: Color::srgba(color.to_srgba().red, color.to_srgba().green, color.to_srgba().blue, 0.4),
            custom_size: Some(Vec2::splat(15.0)),
            ..default()
        },
        Transform::from_translation(position),
        Particle {
            velocity: Vec2::ZERO,
            lifetime: 0.0,
            max_lifetime: 4.0,
            size: 15.0,
            fade_rate: 0.6,
            bioluminescent: false,
            drift_pattern: DriftPattern::Pulsing,
        },
    ));
}

pub fn init_procedural_background(mut commands: Commands) {
    commands.insert_resource(ProceduralBackgroundManager::default());
}