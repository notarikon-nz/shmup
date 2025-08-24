use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::biological_systems::{world_to_grid_pos,sample_current,sample_ph};
use crate::events::{SpawnParticles};

pub fn unified_particle_system(
    mut commands: Commands,
    // Use ParamSet to separate conflicting queries
    mut particle_queries: ParamSet<(
        // Query 0: Standard particles
        Query<(
            Entity,
            &mut Transform,
            &mut Particle,
            &mut Sprite,
            Option<&BioluminescentParticle>
        ), (With<Particle>, Without<ParticleEmitter>, Without<ThermalParticle>, Without<PheromoneParticle>, Without<AlreadyDespawned>)>,

        // Query 1: Standalone bioluminescent particles
        Query<(
            &mut Transform,
            &mut Sprite,
            &mut BioluminescentParticle
        ), (With<BioluminescentParticle>, Without<Particle>, Without<ParticleEmitter>, Without<ThermalParticle>, Without<PheromoneParticle>, Without<AlreadyDespawned>)>,

        // Query 2: Particle emitters
        Query<(&Transform, &mut ParticleEmitter), (Without<Particle>, Without<AlreadyDespawned>)>,

        // Query 3: Thermal particles
        Query<(Entity, &mut Transform, &mut ThermalParticle, &mut Sprite), (With<ThermalParticle>, Without<Particle>, Without<BioluminescentParticle>, Without<ParticleEmitter>, Without<AlreadyDespawned>)>,

        // Query 4: Pheromone particles
        Query<(Entity, &mut Transform, &mut PheromoneParticle, &mut Sprite), (With<PheromoneParticle>, Without<Particle>, Without<BioluminescentParticle>, Without<ParticleEmitter>, Without<ThermalParticle>, Without<AlreadyDespawned>)>,
    )>,

    // Separate player query
    player_query: Query<&Transform, (With<Player>, Without<Particle>, Without<BioluminescentParticle>, Without<ParticleEmitter>, Without<ThermalParticle>, Without<PheromoneParticle>)>,

    // Environmental resources
    fluid_environment: Res<FluidEnvironment>,
    chemical_environment: Res<ChemicalEnvironment>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    // 1. UPDATE STANDARD PARTICLES
    {
        let mut standard_particles = particle_queries.p0();
        for (entity, mut transform, mut particle, mut sprite, bio_particle) in standard_particles.iter_mut() {
            particle.lifetime += time.delta_secs();

            if particle.lifetime >= particle.max_lifetime {
                commands.entity(entity)
                .insert(AlreadyDespawned)
                .despawn();
                continue;
            }

            // Apply organic motion based on drift pattern
            match particle.drift_pattern {
                DriftPattern::Floating => {
                    let bob = (time.elapsed_secs() * 2.0 + transform.translation.x * 0.01).sin();
                    transform.translation.y += bob * 15.0 * time.delta_secs();
                    transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.5);
                }
                DriftPattern::Pulsing => {
                    let pulse = (time.elapsed_secs() * 4.0).sin();
                    let scale = particle.size * (0.8 + pulse * 0.2);
                    transform.scale = Vec3::splat(scale);
                }
                DriftPattern::Spiraling => {
                    let angle = time.elapsed_secs() * 2.0;
                    let spiral_radius = 10.0;
                    particle.velocity.x += angle.cos() * spiral_radius * time.delta_secs();
                    particle.velocity.y += angle.sin() * spiral_radius * time.delta_secs();
                }
                DriftPattern::Brownian => {
                    let random_force = Vec2::new(
                        (time.elapsed_secs() * 123.45 + transform.translation.x * 0.1).sin() * 50.0,
                        (time.elapsed_secs() * 678.90 + transform.translation.y * 0.1).cos() * 50.0,
                    );
                    particle.velocity += random_force * time.delta_secs();
                }
            }

            // Apply fluid current influence
            let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
            let current = sample_current(&fluid_environment, grid_pos);
            particle.velocity += current * 0.3 * time.delta_secs();

            // Update position and apply drag
            transform.translation += particle.velocity.extend(0.0) * time.delta_secs();
            particle.velocity *= 0.98;

            // Handle bioluminescent effects
            if let Some(bio_particle) = bio_particle {
                let pulse = (time.elapsed_secs() * bio_particle.pulse_frequency).sin();
                let brightness = 0.7 + pulse * bio_particle.pulse_intensity;

                let mut color = bio_particle.base_color;
                let alpha = (1.0 - particle.lifetime / particle.max_lifetime) * particle.fade_rate;
                color.set_alpha(alpha * brightness);
                sprite.color = color;

                // Organic size variation
                let size_pulse = (time.elapsed_secs() * 3.0 + particle.lifetime * 2.0).sin();
                let scale = particle.size * (0.9 + size_pulse * 0.1);
                transform.scale = Vec3::splat(scale);
            } else {
                // Standard particle fade
                let progress = particle.lifetime / particle.max_lifetime;
                let alpha = 1.0 - progress;
                sprite.color.set_alpha(alpha * particle.fade_rate);
            }
        }
    }

    // 2. UPDATE STANDALONE BIOLUMINESCENT PARTICLES
    let player_pos = player_query.single().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    {
        let mut bio_particles = particle_queries.p1();
        for (mut transform, mut sprite, bio_particle) in bio_particles.iter_mut() {
            // Distance-based intensity
            let distance_to_player = transform.translation.distance(player_pos);
            let proximity_boost = (200.0 - distance_to_player.min(200.0)) / 200.0;

            // Chemical environment affects bioluminescence
            let ph = sample_ph(&chemical_environment, transform.translation.truncate());
            let ph_factor = 1.0 - ((ph - 7.0).abs() * 0.1).min(0.5);

            // Dynamic pulsing
            let pulse_phase = time.elapsed_secs() * bio_particle.pulse_frequency;
            let pulse = (pulse_phase.sin() * 0.5 + 0.5) * bio_particle.pulse_intensity;

            let final_intensity = (0.4 + pulse * 0.6 + proximity_boost * 0.3) * ph_factor;

            let mut color = bio_particle.base_color;
            color = Color::srgba(
                color.to_srgba().red * final_intensity,
                color.to_srgba().green * final_intensity,
                color.to_srgba().blue * final_intensity,
                color.to_srgba().alpha,
            );
            sprite.color = color;

            // Organic undulation motion
            let undulation = (time.elapsed_secs() * bio_particle.organic_motion.undulation_speed).sin();
            transform.translation.y += undulation * 3.0 * time.delta_secs();
            transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 0.2);
        }
    }

    // 3. UPDATE PARTICLE EMITTERS
    if let Some(assets) = &assets {
        let mut emitters = particle_queries.p2();
        for (emitter_transform, mut emitter) in emitters.iter_mut() {
            if !emitter.active { continue; }

            emitter.spawn_timer -= time.delta_secs();

            if emitter.spawn_timer <= 0.0 {
                let config = &emitter.particle_config;
                let rand_x = (time.elapsed_secs() * 1234.56).fract();
                let rand_y = (time.elapsed_secs() * 5678.90).fract();
                let rand_lifetime = (time.elapsed_secs() * 9012.34).fract();
                let rand_size = (time.elapsed_secs() * 3456.78).fract();

                let velocity = Vec2::new(
                    config.velocity_range.0.x + (config.velocity_range.1.x - config.velocity_range.0.x) * rand_x,
                    config.velocity_range.0.y + (config.velocity_range.1.y - config.velocity_range.0.y) * rand_y,
                );
                let lifetime = config.lifetime_range.0 + (config.lifetime_range.1 - config.lifetime_range.0) * rand_lifetime;
                let size = config.size_range.0 + (config.size_range.1 - config.size_range.0) * rand_size;

                let mut particle_commands = commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: config.color_start,
                        ..default()
                    },
                    Transform::from_translation(emitter_transform.translation).with_scale(Vec3::splat(size)),
                    Particle {
                        velocity,
                        lifetime: 0.0,
                        max_lifetime: lifetime,
                        size,
                        fade_rate: 1.0,
                        bioluminescent: config.organic_motion,
                        drift_pattern: if config.organic_motion { DriftPattern::Floating } else { DriftPattern::Brownian },
                    },
                ));

                if config.organic_motion {
                    particle_commands.insert(BioluminescentParticle {
                        base_color: config.color_start,
                        pulse_frequency: 2.0 + rand_x * 2.0,
                        pulse_intensity: config.bioluminescence,
                        organic_motion: OrganicMotion {
                            undulation_speed: 1.0 + rand_y,
                            response_to_current: 0.6,
                        },
                    });
                }

                emitter.spawn_timer = 1.0 / emitter.spawn_rate;
            }
        }
    }


    // 4. UPDATE THERMAL PARTICLES
    {
        let mut thermal_particles = particle_queries.p3();
        for (entity, mut transform, mut thermal, mut sprite) in thermal_particles.iter_mut() {
            // FIXED: Update lifetime
            thermal.lifetime += time.delta_secs();

            // FIXED: Check both heat intensity AND lifetime for despawning
            if thermal.heat_intensity <= 0.0 || thermal.lifetime >= thermal.max_lifetime {
                commands.entity(entity)
                    .insert(AlreadyDespawned)
                    .despawn();
                continue;
            }

            transform.translation.y += thermal.rise_speed * time.delta_secs();

            // Heat shimmer effect
            let shimmer = (time.elapsed_secs() * 8.0 + transform.translation.x * 0.1).sin() * 3.0;
            transform.translation.x += shimmer * time.delta_secs();

            thermal.heat_intensity -= time.delta_secs() * 0.4;

            // FIXED: Use both lifetime and heat intensity for alpha
            let heat_alpha = thermal.heat_intensity.clamp(0.0, 1.0);
            let lifetime_alpha = (1.0 - thermal.lifetime / thermal.max_lifetime).clamp(0.0, 1.0);
            let final_alpha = heat_alpha * lifetime_alpha;

            sprite.color = Color::srgba(1.0, 0.6 + heat_alpha * 0.4, 0.2, final_alpha * 0.8);
        }
    }

    // 5. UPDATE PHEROMONE PARTICLES
    {
        let mut pheromone_particles = particle_queries.p4();
        for (entity, mut transform, mut pheromone, mut sprite) in pheromone_particles.iter_mut() {
            pheromone.strength -= pheromone.decay_rate * time.delta_secs();

            // Drift with currents
            let grid_pos = world_to_grid_pos(transform.translation.truncate(), &fluid_environment);
            let current = sample_current(&fluid_environment, grid_pos);
            transform.translation += (current * 0.5).extend(0.0) * time.delta_secs();

            // Fade based on strength
            let alpha = pheromone.strength.clamp(0.0, 1.0);
            sprite.color.set_alpha(alpha * 0.4);

            // Organic expansion
            let expansion = (1.0 - pheromone.strength) * 0.5 + 1.0;
            transform.scale = Vec3::splat(expansion);

            if pheromone.strength <= 0.0 {
                commands.entity(entity)
                    .insert(AlreadyDespawned)
                    .despawn();
            }
        }
    }
}


pub fn spawn_particles_system(
    mut commands: Commands,
    mut particle_events: EventReader<SpawnParticles>,
    assets: Option<Res<GameAssets>>,
    time: Res<Time>,
) {
    if let Some(assets) = assets {
        for event in particle_events.read() {
            for i in 0..event.count {
                let config = &event.config;
                let rand_seed = time.elapsed_secs() * 1000.0 + i as f32;
                let rand_x = (rand_seed * 12.9898).sin().abs().fract();
                let rand_y = (rand_seed * 78.233).sin().abs().fract();
                let rand_lifetime = (rand_seed * 35.456).sin().abs().fract();
                let rand_size = (rand_seed * 91.123).sin().abs().fract();
                
                let velocity = Vec2::new(
                    config.velocity_range.0.x + (config.velocity_range.1.x - config.velocity_range.0.x) * rand_x,
                    config.velocity_range.0.y + (config.velocity_range.1.y - config.velocity_range.0.y) * rand_y,
                );
                let lifetime = config.lifetime_range.0 + (config.lifetime_range.1 - config.lifetime_range.0) * rand_lifetime;
                let size = config.size_range.0 + (config.size_range.1 - config.size_range.0) * rand_size;
                
                let drift_pattern = if config.organic_motion {
                    match i % 4 {
                        0 => DriftPattern::Floating,
                        1 => DriftPattern::Pulsing,
                        2 => DriftPattern::Spiraling,
                        _ => DriftPattern::Brownian,
                    }
                } else {
                    DriftPattern::Brownian
                };
                
                let mut particle_commands = commands.spawn((
                    Sprite {
                        image: assets.particle_texture.clone(),
                        color: config.color_start,
                        ..default()
                    },
                    Transform::from_translation(event.position).with_scale(Vec3::splat(size)),
                    Particle {
                        velocity,
                        lifetime: 0.0,
                        max_lifetime: lifetime,
                        size,
                        fade_rate: 1.0,
                        bioluminescent: config.organic_motion,
                        drift_pattern,
                    },
                ));
                
                // Add bioluminescent properties for organic particles
                if config.organic_motion {
                    particle_commands.insert(BioluminescentParticle {
                        base_color: config.color_start,
                        pulse_frequency: 1.5 + rand_x * 3.0,
                        pulse_intensity: config.bioluminescence,
                        organic_motion: OrganicMotion {
                            undulation_speed: 1.0 + rand_y * 2.0,
                            response_to_current: 0.6 + rand_size * 0.4,
                        },
                    });
                }
            }
        }
    }
}


pub fn particle_cleanup(
    mut commands: Commands,
    particle_query: Query<(Entity, &Particle), (With<Particle>, Without<AlreadyDespawned>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    let mut cleanup_count = 0;
    
    for (entity, particle) in particle_query.iter() {
        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity)
                .insert(AlreadyDespawned)
                .despawn();
            cleanup_count += 1;
            
            // Limit cleanup per frame to prevent frame drops
            if cleanup_count >= 50 {
                break;
            }
        }
    }
}