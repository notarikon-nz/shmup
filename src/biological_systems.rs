use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::events::*;
use crate::enemy_types::*;
use crate::despawn::{SafeDespawn};
use std::f32::consts::{TAU};

// Constants to replace magic numbers
const FLUID_UPDATE_INTERVAL: f32 = 0.5;
const CURRENT_CELL_SIZE: f32 = 20.0;
const THERMAL_VENT_RANGE: f32 = 250.0;
const CHEMICAL_ZONE_SPAWN_INTERVAL: f32 = 8.0;
const PH_DAMAGE_THRESHOLD: f32 = 1.5;
const CORAL_SPAWN_INTERVAL: f32 = 25.0;
const DEBRIS_SPAWN_INTERVAL: f32 = 8.0;

// Fluid Dynamics System - Consolidated current generation
pub fn fluid_dynamics_system(
    mut fluid_environment: ResMut<FluidEnvironment>,
    mut current_generator: ResMut<CurrentGenerator>,
    tidal_physics: Res<TidalPoolPhysics>,
    time: Res<Time>,
) {
    update_tidal_cycle(&mut current_generator, &time, &tidal_physics);
    
    if should_update_current_field(&mut current_generator, &time) {
        generate_current_field(&mut fluid_environment, &current_generator, &time);
    }
}

// Chemical Environment System - Consolidated chemical effects
pub fn chemical_environment_system(
    mut chemical_env: ResMut<ChemicalEnvironment>,
    mut organism_query: Query<(&Transform, &mut Health, &ChemicalSensitivity, Option<&OsmoregulationActive>)>,
    enemy_query: Query<(&Transform, &Enemy), Without<Player>>,
    time: Res<Time>,
) {
    update_chemical_zones(&mut chemical_env, &time);
    apply_chemical_effects_to_organisms(organism_query, &chemical_env, &time);
    update_oxygen_depletion(&mut chemical_env, enemy_query, &time);
}

// Organic AI System - Streamlined biological behaviors
pub fn organic_ai_system(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    chemical_environment: Res<ChemicalEnvironment>,
    fluid_environment: Res<FluidEnvironment>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    
    for (mut transform, mut enemy) in enemy_query.iter_mut() {
        update_enemy_ai(&mut transform, &mut enemy, player_transform, &chemical_environment, &fluid_environment, &time);
    }
}

// Thermal Vent Effects - Optimized particle spawning
pub fn thermal_vent_effects_system(
    mut commands: Commands,
    current_generator: Res<CurrentGenerator>,
    mut queries: ParamSet<(
        Query<(&Transform, &mut Health), With<Player>>,
        Query<(Entity, &Transform, &mut Enemy, &mut Health), Without<Player>>,
    )>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut vent_timer: Local<f32>,
) {
    *vent_timer += time.delta_secs();
    
    for vent in current_generator.thermal_vents.iter().filter(|v| v.active) {
        if should_spawn_thermal_particles(*vent_timer) {
            spawn_thermal_particles(&mut commands, &assets, vent.position, vent.strength);
        }
        
        apply_thermal_effects_to_entities(&mut queries, vent, &time);
    }
}

// Enhanced Coral System - Consolidated coral management
pub fn enhanced_coral_system(
    mut commands: Commands,
    mut coral_query: Query<(Entity, &mut EnhancedCoral, &mut Sprite, &mut Transform)>,
    mut player_query: Query<(&Transform, &mut Health, &mut ATP), (With<Player>, Without<EnhancedCoral>)>,
    ecosystem: Res<EcosystemState>,
    mut chemical_environment: ResMut<ChemicalEnvironment>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
    mut spawn_events: EventWriter<SpawnEnemy>,
) {
    process_existing_corals(&mut commands, coral_query, &mut player_query, &mut chemical_environment, &assets, &time, &mut spawn_events, &ecosystem);
    
    *spawn_timer += time.delta_secs();
    if *spawn_timer >= CORAL_SPAWN_INTERVAL {
        *spawn_timer = 0.0;
        spawn_coral_formation(&mut commands, &assets, &ecosystem);
    }
}

// Separate Animation Systems - Each handles specific enemy types
pub fn virus_pulsing_animation(
    mut pulsing_query: Query<(&mut Transform, &mut Sprite, &PulsingAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, mut sprite, pulsing) in pulsing_query.iter_mut() {
        let pulse = (time.elapsed_secs() * pulsing.frequency).sin();
        let scale = 1.0 + pulse * pulsing.intensity;
        transform.scale = Vec3::splat(scale);
        
        let brightness = 0.8 + pulse * 0.2;
        sprite.color = Color::srgba(
            sprite.color.to_srgba().red * brightness,
            sprite.color.to_srgba().green * brightness,
            sprite.color.to_srgba().blue * brightness,
            sprite.color.to_srgba().alpha,
        );
    }
}

pub fn bacteria_flagella_animation(
    mut flagella_query: Query<(&mut Transform, &FlagellaAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, flagella) in flagella_query.iter_mut() {
        let undulation = (time.elapsed_secs() * flagella.undulation_speed).sin();
        transform.translation.x += undulation * flagella.amplitude * time.delta_secs();
        transform.rotation *= Quat::from_rotation_z(undulation * 0.1 * time.delta_secs());
    }
}

pub fn pseudopod_animation(
    mut pseudopod_query: Query<(&mut Transform, &PseudopodAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, pseudopod) in pseudopod_query.iter_mut() {
        let extension = (time.elapsed_secs() * pseudopod.extension_speed).sin();
        let stretch_x = 1.0 + extension * pseudopod.max_extension * 0.1;
        let stretch_y = 1.0 - extension * pseudopod.max_extension * 0.05;
        transform.scale = Vec3::new(stretch_x, stretch_y, 1.0);
    }
}

pub fn corruption_color_shift(
    mut corruption_query: Query<(&mut Sprite, &CorruptionEffect)>,
    time: Res<Time>,
) {
    for (mut sprite, corruption) in corruption_query.iter_mut() {
        let corruption_pulse = (time.elapsed_secs() * corruption.color_shift_speed).sin();
        let corruption_factor = corruption.intensity * (0.5 + corruption_pulse * 0.5);
        
        let base_color = sprite.color;
        sprite.color = Color::srgba(
            base_color.to_srgba().red * (1.0 - corruption_factor * 0.3),
            base_color.to_srgba().green * (1.0 + corruption_factor * 0.2),
            base_color.to_srgba().blue * (1.0 - corruption_factor * 0.5),
            base_color.to_srgba().alpha,
        );
    }
}

pub fn warning_flash_animation(
    mut warning_query: Query<(&mut Sprite, &WarningFlash)>,
    time: Res<Time>,
) {
    for (mut sprite, warning) in warning_query.iter_mut() {
        let flash = (time.elapsed_secs() * warning.flash_frequency).sin();
        sprite.color = if flash > 0.7 {
            warning.warning_color
        } else {
            Color::srgb(1.0, 0.7, 0.2)
        };
    }
}

pub fn offspring_wiggle_animation(
    mut transforms: Query<&mut Transform>,
    wiggle_query: Query<(Entity, &JuvenileWiggle)>,
    time: Res<Time>,
) {
    for (entity, wiggle) in wiggle_query.iter() {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            let wiggle_x = (time.elapsed_secs() * wiggle.wiggle_speed).sin();
            let wiggle_y = (time.elapsed_secs() * wiggle.wiggle_speed * 1.3).cos();
            transform.translation.x += wiggle_x * wiggle.amplitude * time.delta_secs();
            transform.translation.y += wiggle_y * wiggle.amplitude * 0.5 * time.delta_secs();
        }
    }
}

pub fn gestation_animation(
    mut transforms: Query<&mut Transform>,
    gestation_query: Query<(Entity, &GestationAnimation)>,
    time: Res<Time>,
) {
    for (entity, gestation) in gestation_query.iter() {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            let growth_pulse = (time.elapsed_secs() * gestation.pulse_frequency).sin();
            let growth = 1.0 + growth_pulse * gestation.growth_factor * 0.1;
            transform.scale = Vec3::splat(growth);
        }
    }
}

pub fn toxic_aura_animation(
    mut commands: Commands,
    transform_lookup: Query<&Transform, Without<Particle>>,
    aura_query: Query<(Entity, &ToxicAura)>,
    time: Res<Time>,
    assets: Option<Res<GameAssets>>,
) {
    if let Some(assets) = &assets {
        for (entity, aura) in aura_query.iter() {
            if let Ok(transform) = transform_lookup.get(entity) {
                let pulse = (time.elapsed_secs() * aura.pulse_speed).sin();
                if pulse > 0.8 {
                    for i in 0..3 {
                        let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
                        let offset = Vec2::from_angle(angle) * aura.radius;
                        
                        commands.spawn((
                            Sprite {
                                image: assets.particle_texture.clone(),
                                color: Color::srgba(0.6, 0.8, 0.3, 0.5),
                                custom_size: Some(Vec2::splat(4.0)),
                                ..default()
                            },
                            Transform::from_translation(transform.translation + offset.extend(0.0)),
                            Particle {
                                velocity: offset.normalize() * 20.0,
                                lifetime: 0.0,
                                max_lifetime: 2.0,
                                size: 4.0,
                                fade_rate: 0.5,
                                bioluminescent: false,
                                drift_pattern: DriftPattern::Floating,
                            },
                        ));
                    }
                }
            }
        }
    }
}

pub fn signal_particle_spawning(
    mut commands: Commands,
    coordination_query: Query<(Entity, &CoordinationIndicator)>,
    transform_lookup: Query<&Transform, Without<Particle>>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = &assets {
        for (entity, coordination) in coordination_query.iter() {
            if let Ok(transform) = transform_lookup.get(entity) {
                let signal_pulse = (time.elapsed_secs() * 4.0).sin();
                if signal_pulse > 0.9 {
                    for i in 0..2 {
                        let angle = (i as f32 / 2.0) * std::f32::consts::TAU + time.elapsed_secs();
                        let offset = Vec2::from_angle(angle) * coordination.communication_range;
                        
                        commands.spawn((
                            Sprite {
                                image: assets.particle_texture.clone(),
                                color: Color::srgba(0.4, 0.8, 1.0, 0.6),
                                custom_size: Some(Vec2::splat(3.0)),
                                ..default()
                            },
                            Transform::from_translation(transform.translation + offset.extend(0.0)),
                            Particle {
                                velocity: Vec2::ZERO,
                                lifetime: 0.0,
                                max_lifetime: 0.5,
                                size: 3.0,
                                fade_rate: 2.0,
                                bioluminescent: true,
                                drift_pattern: DriftPattern::Pulsing,
                            },
                        ));
                    }
                }
            }
        }
    }
}

// Ecosystem Monitoring - Streamlined state tracking
pub fn ecosystem_monitoring_system(
    mut ecosystem: ResMut<EcosystemState>,
    enemy_query: Query<&Enemy>,
    chemical_environment: Res<ChemicalEnvironment>,
    player_query: Query<&Health, With<Player>>,
) {
    update_population_counts(&mut ecosystem, enemy_query);
    update_ecosystem_health(&mut ecosystem, &chemical_environment, player_query);
}

// Environmental Systems - Consolidated contamination and debris
pub fn environmental_storytelling_system(
    mut commands: Commands,
    mut contamination_query: Query<(&mut ContaminationCloud, &mut Transform, &mut Sprite)>,
    mut debris_query: Query<(Entity, &mut MicroscopicDebris, &mut Transform, &mut Sprite)>,
    player_query: Query<&Transform, (With<Player>, Without<MicroscopicDebris>)>,
    ecosystem: Res<EcosystemState>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
    mut timers: Local<(f32, f32)>, // (contamination_timer, debris_timer)
) {
    update_contamination_clouds(contamination_query, &ecosystem, &time);
    update_microscopic_debris(&mut commands, debris_query, player_query, &time);
    let mut timers_clone = timers.clone();
    spawn_environmental_elements(&mut commands, &assets, &mut timers.0, &mut timers_clone.1, &time);
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn update_tidal_cycle(generator: &mut CurrentGenerator, time: &Res<Time>, physics: &TidalPoolPhysics) {
    generator.tidal_cycle += time.delta_secs() * physics.tide_cycle_speed;
    generator.update_timer += time.delta_secs();
}

fn should_update_current_field(generator: &mut CurrentGenerator, time: &Res<Time>) -> bool {
    if generator.update_timer >= FLUID_UPDATE_INTERVAL {
        generator.update_timer = 0.0;
        true
    } else {
        false
    }
}

fn generate_current_field(fluid_env: &mut FluidEnvironment, generator: &CurrentGenerator, time: &Res<Time>) {
    let tidal_strength = (generator.tidal_cycle * TAU).sin() * 0.4;
    
    for y in 0..fluid_env.grid_size {
        for x in 0..fluid_env.grid_size {
            let world_pos = grid_to_world_pos(x, y, fluid_env);
            let mut flow = Vec2::new(tidal_strength, 0.0);
            
            // Add turbulence
            flow += calculate_turbulence(world_pos, time) * fluid_env.turbulence_intensity * 30.0;
            
            // Apply thermal vent influences
            for vent in &generator.thermal_vents {
                if vent.active {
                    flow += calculate_vent_influence(world_pos, vent, time);
                }
            }
            
            fluid_env.current_field[y * fluid_env.grid_size + x] = flow;
        }
    }
}

fn calculate_turbulence(world_pos: Vec2, time: &Res<Time>) -> Vec2 {
    let noise_x = (world_pos.x * 0.005 + time.elapsed_secs() * 0.2).sin();
    let noise_y = (world_pos.y * 0.007 + time.elapsed_secs() * 0.15).cos();
    Vec2::new(noise_x, noise_y)
}

fn calculate_vent_influence(world_pos: Vec2, vent: &ThermalVent, time: &Res<Time>) -> Vec2 {
    let distance = world_pos.distance(vent.position);
    if distance >= THERMAL_VENT_RANGE { return Vec2::ZERO; }
    
    let direction = (world_pos - vent.position).normalize_or_zero();
    let strength = vent.strength * (1.0 - distance / THERMAL_VENT_RANGE).powi(2);
    let swirl_angle = (distance * 0.02) + (time.elapsed_secs() * 0.8);
    let swirl = Vec2::new(swirl_angle.cos(), swirl_angle.sin());
    
    direction * strength * 0.7 + Vec2::new(0.0, strength * 1.2) + swirl * strength * 0.4
}

fn update_chemical_zones(chemical_env: &mut ChemicalEnvironment, time: &Res<Time>) {
    for zone in &mut chemical_env.ph_zones {
        zone.intensity += (time.elapsed_secs() * 0.5 + zone.position.x * 0.001).sin() * 0.01;
        zone.intensity = zone.intensity.clamp(0.3, 1.0);
        
        zone.position.x += (time.elapsed_secs() * 0.3).sin() * 10.0 * time.delta_secs();
        zone.position.y += (time.elapsed_secs() * 0.2).cos() * 5.0 * time.delta_secs();
    }
}

fn apply_chemical_effects_to_organisms(
    mut organism_query: Query<(&Transform, &mut Health, &ChemicalSensitivity, Option<&OsmoregulationActive>)>,
    chemical_env: &ChemicalEnvironment,
    time: &Res<Time>,
) {
    for (transform, mut health, sensitivity, osmoregulation) in organism_query.iter_mut() {
        if osmoregulation.is_some() { continue; }
        
        let ph = sample_ph(chemical_env, transform.translation.truncate());
        let oxygen = sample_oxygen(chemical_env, transform.translation.truncate());
        
        apply_chemical_damage(&mut health, ph, oxygen, sensitivity, time);
    }
}

fn apply_chemical_damage(health: &mut Health, ph: f32, oxygen: f32, sensitivity: &ChemicalSensitivity, time: &Res<Time>) {
    if ph < sensitivity.ph_tolerance_min || ph > sensitivity.ph_tolerance_max {
        health.0 -= (sensitivity.damage_per_second_outside_range as f32 * time.delta_secs()) as i32;
    }
    
    if oxygen < sensitivity.oxygen_requirement {
        health.0 -= (3.0 * time.delta_secs()) as i32;
    }
}

fn update_oxygen_depletion(
    chemical_env: &mut ChemicalEnvironment,
    enemy_query: Query<(&Transform, &Enemy), Without<Player>>,
    time: &Res<Time>,
) {
    for oxygen_zone in &mut chemical_env.oxygen_zones {
        let nearby_count = enemy_query.iter()
            .filter(|(transform, _)| {
                transform.translation.distance(oxygen_zone.position.extend(0.0)) < oxygen_zone.radius
            })
            .count();
        
        oxygen_zone.oxygen_level -= nearby_count as f32 * oxygen_zone.depletion_rate * time.delta_secs();
        oxygen_zone.oxygen_level = (oxygen_zone.oxygen_level + 0.1 * time.delta_secs()).clamp(0.1, 1.0);
    }
}

fn update_enemy_ai(
    transform: &mut Transform,
    enemy: &mut Enemy,
    player_transform: &Transform,
    chemical_env: &ChemicalEnvironment,
    fluid_env: &FluidEnvironment,
    time: &Res<Time>,
) {
    match &mut enemy.ai_type {
        EnemyAI::Chemotaxis { sensitivity, current_direction, .. } => {
            let player_distance = transform.translation.distance(player_transform.translation);
            if player_distance < 300.0 {
                let direction = (player_transform.translation.truncate() - transform.translation.truncate()).normalize_or_zero();
                let influence = (1.0 / (player_distance * 0.01 + 1.0)) * *sensitivity;
                *current_direction = current_direction.lerp(direction, influence * time.delta_secs());
            }
            transform.translation += current_direction.extend(0.0) * enemy.speed * time.delta_secs();
        }
        
        EnemyAI::FluidFlow { flow_sensitivity, base_direction } => {
            let grid_pos = world_to_grid_pos(transform.translation.truncate(), fluid_env);
            let current = sample_current(fluid_env, grid_pos);
            let flow_influence = current * *flow_sensitivity * time.delta_secs();
            *base_direction = (*base_direction + flow_influence).normalize_or_zero();
            transform.translation += base_direction.extend(0.0) * enemy.speed * time.delta_secs();
        }
        
        EnemyAI::Linear { direction } => {
            let undulation = Vec2::new(
                (time.elapsed_secs() * 2.0 + transform.translation.y * 0.01).sin() * 10.0,
                0.0,
            );
            transform.translation += (direction.extend(0.0) + undulation.extend(0.0)) * enemy.speed * time.delta_secs();
        }
        
        _ => {} // Handle other AI types as needed
    }
    
    // Apply chemical environment effects
    if enemy.chemical_signature.responds_to_pheromones {
        apply_chemical_avoidance(transform, enemy, chemical_env, time);
    }
}

fn apply_chemical_avoidance(transform: &mut Transform, enemy: &Enemy, chemical_env: &ChemicalEnvironment, time: &Res<Time>) {
    let local_ph = sample_ph(chemical_env, transform.translation.truncate());
    let ph_difference = (local_ph - enemy.chemical_signature.ph_preference).abs();
    
    if ph_difference > 1.0 {
        let avoidance = if local_ph > enemy.chemical_signature.ph_preference {
            Vec2::new(-1.0, 0.0)
        } else {
            Vec2::new(1.0, 0.0)
        };
        transform.translation += avoidance.extend(0.0) * enemy.speed * 0.3 * time.delta_secs();
    }
}

fn should_spawn_thermal_particles(timer: f32) -> bool {
    timer % 0.3 < 0.1
}

fn spawn_thermal_particles(commands: &mut Commands, assets: &Option<Res<GameAssets>>, position: Vec2, strength: f32) {
    let Some(assets) = assets else { return };
    
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * TAU;
        let offset = Vec2::from_angle(angle) * 20.0;
        
        commands.spawn((
            Sprite {
                image: assets.particle_texture.clone(),
                color: Color::srgb(1.0, 0.6, 0.2),
                custom_size: Some(Vec2::splat(4.0)),
                ..default()
            },
            Transform::from_translation(position.extend(0.0) + offset.extend(0.0)),
            ThermalParticle {
                heat_intensity: strength / 200.0,
                rise_speed: 60.0,
                lifetime: 0.0,
                max_lifetime: 4.0,
            },
            Particle {
                velocity: Vec2::new(0.0, 80.0) + offset.normalize() * 20.0,
                lifetime: 0.0,
                max_lifetime: 4.0,
                size: 4.0,
                fade_rate: 0.8,
                bioluminescent: true,
                drift_pattern: DriftPattern::Floating,
            },
        ));
    }
}

fn apply_thermal_effects_to_entities(
    queries: &mut ParamSet<(
        Query<(&Transform, &mut Health), With<Player>>,
        Query<(Entity, &Transform, &mut Enemy, &mut Health), Without<Player>>,
    )>,
    vent: &ThermalVent,
    time: &Res<Time>,
) {
    // Player effects
    if let Ok((player_transform, mut player_health)) = queries.p0().single_mut() {
        let distance = player_transform.translation.distance(vent.position.extend(0.0));
        if distance < 120.0 {
            let heat_intensity = (120.0 - distance) / 120.0;
            let health_change = if heat_intensity > 0.7 {
                -(heat_intensity * 15.0 * time.delta_secs()) as i32
            } else if heat_intensity > 0.3 {
                (2.0 * time.delta_secs()) as i32
            } else {
                0
            };
            player_health.0 = (player_health.0 + health_change).clamp(0, 100);
        }
    }
    
    // Enemy effects
    for (_, enemy_transform, mut enemy, mut enemy_health) in queries.p1().iter_mut() {
        apply_thermal_effect_to_enemy(enemy_transform, &mut enemy, &mut enemy_health, vent, time);
    }
}

fn apply_thermal_effect_to_enemy(transform: &Transform, enemy: &mut Enemy, health: &mut Health, vent: &ThermalVent, time: &Res<Time>) {
    let distance = transform.translation.distance(vent.position.extend(0.0));
    if distance >= 150.0 { return; }
    
    let heat_factor = (150.0 - distance) / 150.0;
    
    match enemy.enemy_type {
        EnemyType::ViralParticle if heat_factor > 0.5 => {
            health.0 -= (heat_factor * 20.0 * time.delta_secs()) as i32;
        }
        EnemyType::AggressiveBacteria if heat_factor > 0.3 && heat_factor < 0.8 => {
            enemy.speed = 180.0 * (1.0 + heat_factor * 0.5);
        }
        EnemyType::ParasiticProtozoa if distance < 100.0 => {
            enemy.speed = 120.0;
        }
        _ => {}
    }
}

fn process_existing_corals(
    commands: &mut Commands,
    mut coral_query: Query<(Entity, &mut EnhancedCoral, &mut Sprite, &mut Transform)>,
    player_query: &mut Query<(&Transform, &mut Health, &mut ATP), (With<Player>, Without<EnhancedCoral>)>,
    chemical_environment: &mut ChemicalEnvironment,
    assets: &Option<Res<GameAssets>>,
    time: &Res<Time>,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    ecosystem: &EcosystemState,
) {
    let mut corals_to_remove = Vec::new();
    
    for (entity, mut coral, mut sprite, mut transform) in coral_query.iter_mut() {
        update_coral_health(&mut coral, ecosystem, time);
        
        if coral.health <= 0.0 {
            corals_to_remove.push(entity);
            continue;
        }
        
        update_coral_visuals(&coral, &mut sprite, &mut transform, time);
        apply_coral_effects(&coral, &transform, player_query, chemical_environment, commands, assets, spawn_events, time);
    }
    
    for entity in corals_to_remove {
        commands.entity(entity).safe_despawn();
    }
}

fn update_coral_health(coral: &mut EnhancedCoral, ecosystem: &EcosystemState, time: &Res<Time>) {
    let corruption_factor = 1.0 - ecosystem.health;
    coral.corruption_level += coral.spread_rate * corruption_factor * time.delta_secs();
    coral.corruption_level = coral.corruption_level.clamp(0.0, 1.0);
    coral.health -= coral.corruption_level * 10.0 * time.delta_secs();
}

fn update_coral_visuals(coral: &EnhancedCoral, sprite: &mut Sprite, transform: &mut Transform, time: &Res<Time>) {
    let corruption_color = apply_corruption_to_color(coral.original_color, coral.corruption_level);
    sprite.color = match &coral.coral_type {
        CoralType::BioluminescentBeacon { pulse_frequency, .. } => {
            let pulse = (time.elapsed_secs() * pulse_frequency).sin() * 0.5 + 0.5;
            apply_brightness_to_color(corruption_color, 0.6 + pulse * 0.4)
        }
        CoralType::OxygenProducer { .. } => {
            let breath = (time.elapsed_secs() * 2.0).sin() * 0.05 + 1.0;
            transform.scale = Vec3::splat(breath);
            apply_health_tint(corruption_color, 1.0 - coral.corruption_level)
        }
        _ => corruption_color,
    };
}

fn apply_coral_effects(
    coral: &EnhancedCoral,
    coral_transform: &Transform,
    player_query: &mut Query<(&Transform, &mut Health, &mut ATP), (With<Player>, Without<EnhancedCoral>)>,
    chemical_environment: &mut ChemicalEnvironment,
    commands: &mut Commands,
    assets: &Option<Res<GameAssets>>,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    time: &Res<Time>,
) {
    match &coral.gameplay_effect {
        CoralEffect::Beneficial { healing_per_second, atp_per_second, .. } => {
            apply_beneficial_coral_effects(coral, coral_transform, player_query, time, *healing_per_second, *atp_per_second);
        }
        CoralEffect::Harmful { damage_per_second, spawns_enemies, .. } => {
            apply_harmful_coral_effects(coral, coral_transform, player_query, spawn_events, time, *damage_per_second, *spawns_enemies);
        }
        _ => {}
    }
}

fn apply_beneficial_coral_effects(
    coral: &EnhancedCoral,
    coral_transform: &Transform,
    player_query: &mut Query<(&Transform, &mut Health, &mut ATP), (With<Player>, Without<EnhancedCoral>)>,
    time: &Res<Time>,
    healing_rate: f32,
    atp_rate: f32,
) {
    let Ok((player_transform, mut health, mut atp)) = player_query.single_mut() else { return };
    
    let distance = player_transform.translation.distance(coral_transform.translation);
    if distance >= coral.influence_radius { return; }
    
    let effectiveness = (1.0 - coral.corruption_level).max(0.0);
    let proximity_factor = (coral.influence_radius - distance) / coral.influence_radius;
    let effect_strength = effectiveness * proximity_factor;
    
    if healing_rate > 0.0 {
        health.0 = (health.0 + (healing_rate * effect_strength * time.delta_secs()) as i32).min(100);
    }
    
    if atp_rate > 0.0 {
        atp.amount += (atp_rate * effect_strength * time.delta_secs()) as u32;
    }
}

fn apply_harmful_coral_effects(
    coral: &EnhancedCoral,
    coral_transform: &Transform,
    player_query: &mut Query<(&Transform, &mut Health, &mut ATP), (With<Player>, Without<EnhancedCoral>)>,
    spawn_events: &mut EventWriter<SpawnEnemy>,
    time: &Res<Time>,
    damage_rate: f32,
    spawn_rate: f32,
) {
    let harm_effectiveness = coral.corruption_level;
    
    if damage_rate > 0.0 && harm_effectiveness > 0.3 {
        if let Ok((player_transform, mut health, _)) = player_query.single_mut() {
            let distance = player_transform.translation.distance(coral_transform.translation);
            if distance < coral.influence_radius {
                let proximity_factor = (coral.influence_radius - distance) / coral.influence_radius;
                let damage = (damage_rate * harm_effectiveness * proximity_factor * time.delta_secs()) as i32;
                health.0 -= damage;
            }
        }
    }
    
    if spawn_rate > 0.0 && (time.elapsed_secs() % (10.0 / spawn_rate)) < 0.1 {
        spawn_coral_enemy(spawn_events, coral_transform.translation);
    }
}

fn spawn_coral_enemy(spawn_events: &mut EventWriter<SpawnEnemy>, position: Vec3) {
    spawn_events.write(SpawnEnemy {
        position,
        ai_type: EnemyAI::Chemotaxis {
            target_chemical: ChemicalType::PlayerPheromones,
            sensitivity: 1.5,
            current_direction: Vec2::new(0.0, -1.0),
        },
        enemy_type: EnemyType::ViralParticle,
    });
}

fn update_contamination_clouds(
    mut contamination_query: Query<(&mut ContaminationCloud, &mut Transform, &mut Sprite)>,
    ecosystem: &EcosystemState,
    time: &Res<Time>,
) {
    for (mut cloud, mut transform, mut sprite) in contamination_query.iter_mut() {
        let expansion_factor = 1.0 + (1.0 - ecosystem.health) * 2.0;
        transform.scale += Vec3::splat(cloud.expansion_rate * expansion_factor * time.delta_secs());
        
        cloud.toxicity_level = (cloud.toxicity_level + 0.1 * time.delta_secs()).clamp(0.0, 2.0);
        sprite.color = get_contamination_color(&cloud.source_type, cloud.toxicity_level, time);
    }
}

fn update_microscopic_debris(
    commands: &mut Commands,
    mut debris_query: Query<(Entity, &mut MicroscopicDebris, &mut Transform, &mut Sprite)>,
    player_query: Query<&Transform, (With<Player>, Without<MicroscopicDebris>)>,
    time: &Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let mut entities_to_despawn = Vec::new();
    
    for (entity, mut debris, mut debris_transform, mut sprite) in debris_query.iter_mut() {
        debris.age += time.delta_secs();
        
        let distance = player_transform.translation.distance(debris_transform.translation);
        if distance < debris.reveal_distance {
            let reveal_intensity = (debris.reveal_distance - distance) / debris.reveal_distance;
            sprite.color.set_alpha(0.4 + reveal_intensity * 0.6);
            debris_transform.scale = Vec3::splat(1.0 + reveal_intensity * 0.5);
        }
        
        if debris.age > 60.0 {
            entities_to_despawn.push(entity);
        }
    }
    
    for entity in entities_to_despawn {
        commands.entity(entity).safe_despawn();
    }
}

fn spawn_environmental_elements(
    commands: &mut Commands,
    assets: &Option<Res<GameAssets>>,
    contamination_timer: &mut f32,
    debris_timer: &mut f32,
    time: &Res<Time>,
) {
    *contamination_timer += time.delta_secs();
    *debris_timer += time.delta_secs();
    
    if *contamination_timer >= CHEMICAL_ZONE_SPAWN_INTERVAL {
        *contamination_timer = 0.0;
        spawn_contamination_event(commands);
    }
    
    if *debris_timer >= DEBRIS_SPAWN_INTERVAL {
        *debris_timer = 0.0;
        spawn_story_debris(commands, assets);
    }
}

fn update_population_counts(ecosystem: &mut EcosystemState, enemy_query: Query<&Enemy>) {
    let mut pathogenic = 0;
    let mut beneficial = 0;
    
    for enemy in enemy_query.iter() {
        match enemy.enemy_type {
            EnemyType::AggressiveBacteria | EnemyType::ViralParticle => pathogenic += 1,
            EnemyType::SwarmCell => beneficial += 1,
            _ => {}
        }
    }
    
    ecosystem.population_balance.pathogenic_threats = pathogenic;
    ecosystem.population_balance.beneficial_microbes = beneficial;
}

fn update_ecosystem_health(
    ecosystem: &mut EcosystemState,
    chemical_environment: &ChemicalEnvironment,
    player_query: Query<&Health, With<Player>>,
) {
    let pathogen_ratio = ecosystem.population_balance.pathogenic_threats as f32 
        / (ecosystem.population_balance.pathogenic_threats + ecosystem.population_balance.beneficial_microbes + 1) as f32;
    
    ecosystem.infection_level = pathogen_ratio;
    ecosystem.health = 1.0 - (pathogen_ratio * 0.8);
    
    // pH stability calculation
    if !chemical_environment.ph_zones.is_empty() {
        let avg_ph = chemical_environment.ph_zones.iter()
            .map(|z| z.ph_level)
            .sum::<f32>() / chemical_environment.ph_zones.len() as f32;
        ecosystem.ph_stability = 1.0 - ((avg_ph - 7.0).abs() / 7.0);
    }
    
    // Player health affects ecosystem
    if let Ok(player_health) = player_query.single() {
        ecosystem.symbiotic_activity = (player_health.0 as f32 / 100.0) * 0.6 + 0.4;
    }
}

fn spawn_contamination_event(commands: &mut Commands) {
    let contamination_type = match rand::random::<u32>() % 3 {
        0 => ContaminationType::IndustrialWaste,
        1 => ContaminationType::ChemicalSpill,
        _ => ContaminationType::BiologicalToxin,
    };
    
    let spawn_pos = Vec3::new(
        (rand::random::<f32>() - 0.5) * 800.0,
        200.0 + rand::random::<f32>() * 100.0,
        -0.5
    );
    
    commands.spawn((
        Sprite {
            color: Color::srgba(0.6, 0.3, 0.3, 0.4),
            custom_size: Some(Vec2::splat(80.0)),
            ..default()
        },
        Transform::from_translation(spawn_pos),
        ContaminationCloud {
            toxicity_level: 0.5,
            expansion_rate: 0.2,
            source_type: contamination_type,
            warning_intensity: 1.0,
        },
        ParallaxLayer { speed: 0.1, depth: -0.5 },
    ));
}

fn spawn_story_debris(commands: &mut Commands, assets: &Option<Res<GameAssets>>) {
    let Some(assets) = assets else { return };
    
    let debris_stories = [
        ("Microplastic fragment from ocean surface", Color::srgb(0.8, 0.8, 0.9)),
        ("Industrial metal particle, heavily corroded", Color::srgb(0.6, 0.6, 0.7)),
        ("Chemical residue from agricultural runoff", Color::srgb(0.8, 0.7, 0.3)),
        ("Decomposing plankton, ecosystem disruption", Color::srgb(0.5, 0.4, 0.3)),
        ("Synthetic fiber from clothing waste", Color::srgb(0.7, 0.3, 0.8)),
    ];
    
    let (story, color) = &debris_stories[rand::random::<u32>() as usize % debris_stories.len()];
    let x = (rand::random::<f32>() - 0.5) * 1200.0;
    let y = 400.0 + rand::random::<f32>() * 100.0;
    
    commands.spawn((
        Sprite {
            image: assets.particle_texture.clone(),
            color: *color,
            custom_size: Some(Vec2::splat(8.0)),
            ..default()
        },
        Transform::from_xyz(x, y, -0.3),
        MicroscopicDebris {
            debris_type: DebrisType::PlasticFragment { size: 2.0, color: *color },
            story_fragment: story.to_string(),
            age: 0.0,
            reveal_distance: 60.0,
        },
        Particle {
            velocity: Vec2::new(0.0, -40.0),
            lifetime: 0.0,
            max_lifetime: 45.0,
            size: 8.0,
            fade_rate: 0.8,
            bioluminescent: false,
            drift_pattern: DriftPattern::Brownian,
        },
        ParallaxLayer { speed: 0.3, depth: -0.3 },
    ));
}

fn spawn_coral_formation(commands: &mut Commands, assets: &Option<Res<GameAssets>>, ecosystem: &EcosystemState) {
    let Some(assets) = assets else { return };
    
    let x = (rand::random::<f32>() - 0.5) * 1000.0;
    let y = (rand::random::<f32>() - 0.5) * 400.0;
    
    let (coral_type, coral_effect, color, influence_radius) = get_coral_type_for_ecosystem_health(ecosystem.health);
    let initial_corruption = (1.0 - ecosystem.health) * 0.3;
    
    commands.spawn((
        Sprite {
            image: assets.enemy_texture.clone(),
            color,
            custom_size: Some(Vec2::splat(50.0 + rand::random::<f32>() * 30.0)),
            ..default()
        },
        Transform::from_xyz(x, y, -0.7),
        EnhancedCoral {
            coral_type,
            health: 100.0,
            corruption_level: initial_corruption,
            spread_rate: 0.02 + rand::random::<f32>() * 0.03,
            bioluminescent_warning: rand::random::<bool>(),
            original_color: color,
            size: Vec2::splat(50.0),
            gameplay_effect: coral_effect,
            influence_radius,
            last_spawn_time: 0.0,
        },
        ParallaxLayer { speed: 0.15, depth: -0.7 },
    ));
}

fn get_coral_type_for_ecosystem_health(health: f32) -> (CoralType, CoralEffect, Color, f32) {
    if health > 0.7 {
        // Healthy ecosystem - beneficial corals
        let beneficial_types = [
            (
                CoralType::FilterFeeder { purification_rate: 0.5, ph_stabilization: 0.3 },
                CoralEffect::Beneficial { healing_per_second: 0.0, atp_per_second: 0.0, ph_stabilization: 0.3, oxygen_boost: 0.2 },
                Color::srgb(0.3, 0.8, 0.6),
                100.0,
            ),
            (
                CoralType::OxygenProducer { oxygen_output: 0.4, photosynthesis_rate: 0.6 },
                CoralEffect::Beneficial { healing_per_second: 2.0, atp_per_second: 1.0, ph_stabilization: 0.0, oxygen_boost: 0.4 },
                Color::srgb(0.2, 0.9, 0.3),
                80.0,
            ),
        ];
        beneficial_types[rand::random::<u32>() as usize % beneficial_types.len()].clone()
    } else if health > 0.4 {
        // Neutral ecosystem
        (
            CoralType::BioluminescentBeacon { pulse_frequency: 2.0, detection_range: 150.0 },
            CoralEffect::Neutral { provides_cover: true, navigation_aid: true },
            Color::srgb(0.8, 0.7, 0.2),
            90.0,
        )
    } else {
        // Degraded ecosystem - harmful corals
        let harmful_types = [
            (
                CoralType::CorruptedColony { toxin_production: 0.3, spawn_hostiles: true },
                CoralEffect::Harmful { damage_per_second: 2.0, ph_reduction: 0.1, spawns_enemies: 0.5, corruption_spread: 0.2 },
                Color::srgb(0.8, 0.3, 0.3),
                110.0,
            ),
            (
                CoralType::AcidicFormation { acid_strength: 0.8, corrosion_rate: 0.4 },
                CoralEffect::Harmful { damage_per_second: 1.5, ph_reduction: 0.3, spawns_enemies: 0.0, corruption_spread: 0.1 },
                Color::srgb(0.9, 0.8, 0.2),
                95.0,
            ),
        ];
        harmful_types[rand::random::<u32>() as usize % harmful_types.len()].clone()
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn grid_to_world_pos(grid_x: usize, grid_y: usize, fluid_env: &FluidEnvironment) -> Vec2 {
    Vec2::new(
        grid_x as f32 * fluid_env.cell_size - 640.0,
        grid_y as f32 * fluid_env.cell_size - 360.0,
    )
}

fn world_to_grid_pos(world_pos: Vec2, fluid_env: &FluidEnvironment) -> (usize, usize) {
    let grid_x = ((world_pos.x + 640.0) / fluid_env.cell_size) as usize;
    let grid_y = ((world_pos.y + 360.0) / fluid_env.cell_size) as usize;
    (grid_x.clamp(0, fluid_env.grid_size - 1), grid_y.clamp(0, fluid_env.grid_size - 1))
}

fn sample_current(fluid_env: &FluidEnvironment, grid_pos: (usize, usize)) -> Vec2 {
    let index = grid_pos.1 * fluid_env.grid_size + grid_pos.0;
    fluid_env.current_field.get(index).copied().unwrap_or(Vec2::ZERO)
}

pub fn sample_ph(chemical_env: &ChemicalEnvironment, position: Vec2) -> f32 {
    let mut ph = chemical_env.base_ph;
    for zone in &chemical_env.ph_zones {
        let distance = position.distance(zone.position);
        if distance < zone.radius {
            let influence = (1.0 - distance / zone.radius) * zone.intensity;
            ph = ph * (1.0 - influence) + zone.ph_level * influence;
        }
    }
    ph.clamp(0.0, 14.0)
}

pub fn sample_oxygen(chemical_env: &ChemicalEnvironment, position: Vec2) -> f32 {
    let mut oxygen = chemical_env.base_oxygen;
    for zone in &chemical_env.oxygen_zones {
        let distance = position.distance(zone.position);
        if distance < zone.radius {
            let influence = 1.0 - distance / zone.radius;
            oxygen = oxygen * (1.0 - influence) + zone.oxygen_level * influence;
        }
    }
    oxygen.clamp(0.0, 1.0)
}

fn apply_brightness_to_color(color: Color, brightness: f32) -> Color {
    Color::srgba(
        color.to_srgba().red * brightness,
        color.to_srgba().green * brightness,
        color.to_srgba().blue * brightness,
        color.to_srgba().alpha,
    )
}

fn apply_corruption_to_color(color: Color, corruption_level: f32) -> Color {
    Color::srgb(
        color.to_srgba().red * (1.0 - corruption_level * 0.7),
        color.to_srgba().green * (1.0 - corruption_level * 0.9),
        color.to_srgba().blue * (1.0 - corruption_level * 0.5),
    )
}

fn apply_health_tint(color: Color, health_factor: f32) -> Color {
    Color::srgb(
        color.to_srgba().red * (1.0 - health_factor * 0.3),
        color.to_srgba().green + health_factor * 0.3,
        color.to_srgba().blue * (1.0 - health_factor * 0.2),
    )
}

fn apply_corruption_color_shift(color: Color, factor: f32) -> Color {
    Color::srgba(
        color.to_srgba().red * (1.0 - factor * 0.3),
        color.to_srgba().green * (1.0 + factor * 0.2),
        color.to_srgba().blue * (1.0 - factor * 0.5),
        color.to_srgba().alpha,
    )
}

fn get_contamination_color(source_type: &ContaminationType, toxicity_level: f32, time: &Res<Time>) -> Color {
    match source_type {
        ContaminationType::IndustrialWaste => {
            Color::srgba(0.6, 0.4, 0.2, 0.4 + toxicity_level * 0.3)
        }
        ContaminationType::BiologicalToxin => {
            let pulse = (time.elapsed_secs() * 3.0).sin() * 0.2 + 0.8;
            Color::srgba(0.3, 0.8, 0.3, (0.3 + toxicity_level * 0.2) * pulse)
        }
        ContaminationType::RadioactiveSeepage => {
            let flicker = if (time.elapsed_secs() * 10.0).sin() > 0.7 { 1.0 } else { 0.6 };
            Color::srgba(0.9, 1.0, 0.3, (0.2 + toxicity_level * 0.3) * flicker)
        }
        ContaminationType::ChemicalSpill => {
            let shift = (time.elapsed_secs()).sin() * 0.5 + 0.5;
            Color::srgba(
                0.8 + shift * 0.2,
                0.4 + shift * 0.4,
                0.9 - shift * 0.3,
                0.3 + toxicity_level * 0.2
            )
        }
        ContaminationType::PlasticPollution => {
            Color::srgba(0.7, 0.7, 0.7, 0.5 + toxicity_level * 0.3)
        }
    }
}