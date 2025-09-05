use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::components::*;
use crate::events::*;

#[derive(Component)]
pub struct HanabiParticleEffect {
    pub effect_type: ParticleEffectType,
    pub is_active: bool,
}

#[derive(Clone)]
pub enum ParticleEffectType {
    BioluminescentTrail,
    ThermalVent,
    Explosion,
    ATP,
    Pheromone,
    EngineTrail,
    CellBurst,
    SporeCloud,
    Background,
}

pub struct HanabiParticlePlugin;

impl Plugin for HanabiParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(Startup, setup_particle_effects)
            .add_systems(Update, (
                spawn_hanabi_particles_system,
            ));
    }
}

#[derive(Resource)]
pub struct ParticleEffects {
    pub bioluminescent_trail: Handle<EffectAsset>,
    pub thermal_vent: Handle<EffectAsset>,
    pub explosion: Handle<EffectAsset>,
    pub atp_collection: Handle<EffectAsset>,
    pub pheromone_trail: Handle<EffectAsset>,
    pub engine_trail: Handle<EffectAsset>,
    pub cell_burst: Handle<EffectAsset>,
    pub spore_cloud: Handle<EffectAsset>,
    pub background_plankton: Handle<EffectAsset>,
}

pub fn setup_particle_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let writer = ExprWriter::new();

    // Common attributes
    let age = writer.lit(0.0).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Bioluminescent trail effect for organic particles
    let bio_lifetime = writer.lit(2.0).uniform(writer.lit(4.0)).expr();
    let bio_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, bio_lifetime);
    
    let bio_init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(3.0).expr(),
        dimension: ShapeDimension::Surface,
    };
    
    let bio_init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(20.0).uniform(writer.lit(60.0)).expr(),
    };

    let mut bio_module = writer.finish();
    let bio_spawner = SpawnerSettings::rate(200.0.into());
    
    let mut bio_gradient = Gradient::new();
    bio_gradient.add_key(0.0, Vec4::new(0.4, 0.9, 0.7, 0.0));
    bio_gradient.add_key(0.1, Vec4::new(0.4, 0.9, 0.7, 1.0));
    bio_gradient.add_key(0.9, Vec4::new(0.6, 1.0, 0.8, 0.8));
    bio_gradient.add_key(1.0, Vec4::new(0.8, 1.0, 0.9, 0.0));

    let bioluminescent_trail = effects.add(
        EffectAsset::new(32768, bio_spawner, bio_module)
            .with_name("bioluminescent_trail")
            .init(bio_init_pos)
            .init(bio_init_vel)
            .init(init_age.clone())
            .init(bio_init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.02)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(bio_gradient)),
    );

    // Thermal vent effect with heat shimmer and rising particles
    let writer2 = ExprWriter::new();
    let thermal_lifetime = writer2.lit(3.0).uniform(writer2.lit(6.0)).expr();
    let thermal_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, thermal_lifetime);
    
    let thermal_init_pos = SetPositionCircleModifier {
        center: writer2.lit(Vec3::ZERO).expr(),
        axis: writer2.lit(Vec3::Y).expr(),
        radius: writer2.lit(10.0).expr(),
        dimension: ShapeDimension::Surface,
    };
    
    let thermal_init_vel = SetVelocityCircleModifier {
        center: writer2.lit(Vec3::Y).expr(),
        axis: writer2.lit(Vec3::Y).expr(),
        speed: writer2.lit(15.0).uniform(writer2.lit(45.0)).expr(),
    };

    let mut thermal_module = writer2.finish();
    let thermal_accel = AccelModifier::constant(&mut thermal_module, Vec3::new(0.0, 30.0, 0.0));
    let thermal_spawner = SpawnerSettings::rate(60.0.into());
    
    let mut thermal_gradient = Gradient::new();
    thermal_gradient.add_key(0.0, Vec4::new(1.0, 0.6, 0.2, 0.8));
    thermal_gradient.add_key(0.5, Vec4::new(1.0, 0.8, 0.4, 0.6));
    thermal_gradient.add_key(1.0, Vec4::new(1.0, 1.0, 0.8, 0.0));

    let thermal_vent = effects.add(
        EffectAsset::new(1024, thermal_spawner, thermal_module)
            .with_name("thermal_vent")
            .init(thermal_init_pos)
            .init(thermal_init_vel)
            .init(init_age.clone())
            .init(thermal_init_lifetime)
            .update(thermal_accel) // Rising heat
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.04)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(thermal_gradient)),
    );

    // Explosion effect with burst pattern
    let writer3 = ExprWriter::new();
    let explosion_lifetime = writer3.lit(0.5).uniform(writer3.lit(2.0)).expr();
    let explosion_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, explosion_lifetime);
    
    let explosion_init_pos = SetPositionSphereModifier {
        center: writer3.lit(Vec3::ZERO).expr(),
        radius: writer3.lit(5.0).expr(),
        dimension: ShapeDimension::Surface,
    };
    
    let explosion_init_vel = SetVelocitySphereModifier {
        center: writer3.lit(Vec3::ZERO).expr(),
        speed: writer3.lit(50.0).uniform(writer3.lit(200.0)).expr(),
    };

    let mut explosion_module = writer3.finish();
    let explosion_accel = AccelModifier::constant(&mut explosion_module, Vec3::new(0.0, -50.0, 0.0));
    let explosion_spawner = SpawnerSettings::burst(150.0.into(), 0.1.into());
    
    let mut explosion_gradient = Gradient::new();
    explosion_gradient.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
    explosion_gradient.add_key(0.1, Vec4::new(1.0, 0.8, 0.3, 1.0));
    explosion_gradient.add_key(0.5, Vec4::new(0.8, 0.4, 0.1, 0.8));
    explosion_gradient.add_key(1.0, Vec4::new(0.2, 0.2, 0.2, 0.0));

    let explosion = effects.add(
        EffectAsset::new(2048, explosion_spawner, explosion_module)
            .with_name("explosion")
            .init(explosion_init_pos)
            .init(explosion_init_vel)
            .init(init_age.clone())
            .init(explosion_init_lifetime)
            .update(explosion_accel) // Gravity
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.06)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(explosion_gradient)),
    );

    // ATP collection effect with attraction pattern
    let writer4 = ExprWriter::new();
    let atp_lifetime = writer4.lit(1.0).uniform(writer4.lit(2.5)).expr();
    let atp_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, atp_lifetime);
    
    let atp_init_pos = SetPositionSphereModifier {
        center: writer4.lit(Vec3::ZERO).expr(),
        radius: writer4.lit(15.0).expr(),
        dimension: ShapeDimension::Surface,
    };
    
    let atp_init_vel = SetVelocitySphereModifier {
        center: writer4.lit(Vec3::ZERO).expr(),
        speed: writer4.lit(25.0).uniform(writer4.lit(75.0)).expr(),
    };

    let mut atp_module = writer4.finish();
    let atp_spawner = SpawnerSettings::rate(80.0.into());
    
    let mut atp_gradient = Gradient::new();
    atp_gradient.add_key(0.0, Vec4::new(1.0, 1.0, 0.3, 0.0));
    atp_gradient.add_key(0.1, Vec4::new(1.0, 1.0, 0.5, 1.0));
    atp_gradient.add_key(0.9, Vec4::new(1.0, 0.8, 0.3, 0.8));
    atp_gradient.add_key(1.0, Vec4::new(0.8, 0.6, 0.2, 0.0));

    let atp_collection = effects.add(
        EffectAsset::new(1024, atp_spawner, atp_module)
            .with_name("atp_collection")
            .init(atp_init_pos)
            .init(atp_init_vel)
            .init(init_age.clone())
            .init(atp_init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.025)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(atp_gradient)),
    );

    // Pheromone trail effect for chemical communication
    let writer5 = ExprWriter::new();
    let pheromone_lifetime = writer5.lit(4.0).uniform(writer5.lit(8.0)).expr();
    let pheromone_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, pheromone_lifetime);
    
    let pheromone_init_pos = SetPositionCircleModifier {
        center: writer5.lit(Vec3::ZERO).expr(),
        axis: writer5.lit(Vec3::Z).expr(),
        radius: writer5.lit(5.0).expr(),
        dimension: ShapeDimension::Surface,
    };
    
    let pheromone_init_vel = SetVelocityCircleModifier {
        center: writer5.lit(Vec3::ZERO).expr(),
        axis: writer5.lit(Vec3::Z).expr(),
        speed: writer5.lit(8.0).uniform(writer5.lit(20.0)).expr(),
    };

    let mut pheromone_module = writer5.finish();
    let pheromone_spawner = SpawnerSettings::rate(25.0.into());
    
    let mut pheromone_gradient = Gradient::new();
    pheromone_gradient.add_key(0.0, Vec4::new(0.8, 0.3, 0.8, 0.0));
    pheromone_gradient.add_key(0.2, Vec4::new(0.9, 0.4, 0.9, 0.6));
    pheromone_gradient.add_key(0.8, Vec4::new(0.7, 0.2, 0.7, 0.4));
    pheromone_gradient.add_key(1.0, Vec4::new(0.5, 0.1, 0.5, 0.0));

    let pheromone_trail = effects.add(
        EffectAsset::new(512, pheromone_spawner, pheromone_module)
            .with_name("pheromone_trail")
            .init(pheromone_init_pos)
            .init(pheromone_init_vel)
            .init(init_age.clone())
            .init(pheromone_init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.02)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(pheromone_gradient)),
    );

    // Engine trail effect for player
    let writer6 = ExprWriter::new();
    let engine_lifetime = writer6.lit(0.8).uniform(writer6.lit(2.0)).expr();
    let engine_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, engine_lifetime);
    
    let engine_init_pos = SetPositionCircleModifier {
        center: writer6.lit(Vec3::ZERO).expr(),
        axis: writer6.lit(Vec3::Z).expr(),
        radius: writer6.lit(2.0).expr(),
        dimension: ShapeDimension::Surface,
    };
    
    let engine_init_vel = SetVelocityCircleModifier {
        center: writer6.lit(Vec3::new(0.0, -1.0, 0.0)).expr(),
        axis: writer6.lit(Vec3::Z).expr(),
        speed: writer6.lit(15.0).uniform(writer6.lit(35.0)).expr(),
    };

    let mut engine_module = writer6.finish();
    let engine_spawner = SpawnerSettings::rate(150.0.into());
    
    let mut engine_gradient = Gradient::new();
    engine_gradient.add_key(0.0, Vec4::new(0.3, 0.8, 1.0, 1.0));
    engine_gradient.add_key(0.5, Vec4::new(0.4, 0.9, 1.0, 0.8));
    engine_gradient.add_key(1.0, Vec4::new(0.6, 1.0, 1.0, 0.0));

    let engine_trail = effects.add(
        EffectAsset::new(1024, engine_spawner, engine_module)
            .with_name("engine_trail")
            .init(engine_init_pos)
            .init(engine_init_vel)
            .init(init_age.clone())
            .init(engine_init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.015)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(engine_gradient)),
    );

    // Cell burst effect for enemy destruction
    let writer7 = ExprWriter::new();
    let cell_lifetime = writer7.lit(0.4).uniform(writer7.lit(1.2)).expr();
    let cell_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, cell_lifetime);
    
    let cell_init_pos = SetPositionSphereModifier {
        center: writer7.lit(Vec3::ZERO).expr(),
        radius: writer7.lit(3.0).expr(),
        dimension: ShapeDimension::Surface,
    };
    
    let cell_init_vel = SetVelocitySphereModifier {
        center: writer7.lit(Vec3::ZERO).expr(),
        speed: writer7.lit(40.0).uniform(writer7.lit(120.0)).expr(),
    };

    let mut cell_module = writer7.finish();
    let cell_spawner = SpawnerSettings::burst(60.0.into(), 0.05.into());
    
    let mut cell_gradient = Gradient::new();
    cell_gradient.add_key(0.0, Vec4::new(0.8, 1.0, 0.9, 1.0));
    cell_gradient.add_key(0.3, Vec4::new(0.6, 0.9, 0.7, 0.8));
    cell_gradient.add_key(1.0, Vec4::new(0.4, 0.7, 0.5, 0.0));

    let cell_burst = effects.add(
        EffectAsset::new(512, cell_spawner, cell_module)
            .with_name("cell_burst")
            .init(cell_init_pos)
            .init(cell_init_vel)
            .init(init_age.clone())
            .init(cell_init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.035)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(cell_gradient)),
    );

    // Spore cloud effect for toxin clouds
    let writer8 = ExprWriter::new();
    let spore_lifetime = writer8.lit(5.0).uniform(writer8.lit(10.0)).expr();
    let spore_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, spore_lifetime);
    
    let spore_init_pos = SetPositionSphereModifier {
        center: writer8.lit(Vec3::ZERO).expr(),
        radius: writer8.lit(25.0).expr(),
        dimension: ShapeDimension::Volume,
    };
    
    let spore_init_vel = SetVelocitySphereModifier {
        center: writer8.lit(Vec3::ZERO).expr(),
        speed: writer8.lit(8.0).uniform(writer8.lit(25.0)).expr(),
    };

    let mut spore_module = writer8.finish();
    let spore_spawner = SpawnerSettings::rate(100.0.into());
    
    let mut spore_gradient = Gradient::new();
    spore_gradient.add_key(0.0, Vec4::new(0.8, 0.6, 1.0, 0.0));
    spore_gradient.add_key(0.2, Vec4::new(0.9, 0.7, 1.0, 0.8));
    spore_gradient.add_key(0.8, Vec4::new(0.7, 0.5, 0.9, 0.6));
    spore_gradient.add_key(1.0, Vec4::new(0.6, 0.4, 0.8, 0.0));

    let spore_cloud = effects.add(
        EffectAsset::new(2048, spore_spawner, spore_module)
            .with_name("spore_cloud")
            .init(spore_init_pos)
            .init(spore_init_vel)
            .init(init_age.clone())
            .init(spore_init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.025)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(spore_gradient)),
    );

    // Background plankton effect
    let writer9 = ExprWriter::new();
    let plankton_lifetime = writer9.lit(15.0).uniform(writer9.lit(25.0)).expr();
    let plankton_init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, plankton_lifetime);
    
    let plankton_init_pos = SetPositionSphereModifier {
        center: writer9.lit(Vec3::ZERO).expr(),
        radius: writer9.lit(200.0).expr(),
        dimension: ShapeDimension::Volume,
    };
    
    let plankton_init_vel = SetVelocitySphereModifier {
        center: writer9.lit(Vec3::new(0.0, -0.5, 0.0)).expr(),
        speed: writer9.lit(5.0).uniform(writer9.lit(15.0)).expr(),
    };

    let mut plankton_module = writer9.finish();
    let plankton_spawner = SpawnerSettings::rate(20.0.into());
    
    let mut plankton_gradient = Gradient::new();
    plankton_gradient.add_key(0.0, Vec4::new(0.8, 1.0, 0.9, 0.0));
    plankton_gradient.add_key(0.1, Vec4::new(0.8, 1.0, 0.9, 0.4));
    plankton_gradient.add_key(0.9, Vec4::new(0.6, 0.9, 0.7, 0.3));
    plankton_gradient.add_key(1.0, Vec4::new(0.5, 0.8, 0.6, 0.0));

    let background_plankton = effects.add(
        EffectAsset::new(1024, plankton_spawner, plankton_module)
            .with_name("background_plankton")
            .init(plankton_init_pos)
            .init(plankton_init_vel)
            .init(init_age)
            .init(plankton_init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(0.01)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier::new(plankton_gradient)),
    );

    commands.insert_resource(ParticleEffects {
        bioluminescent_trail,
        thermal_vent,
        explosion,
        atp_collection,
        pheromone_trail,
        engine_trail,
        cell_burst,
        spore_cloud,
        background_plankton,
    });
}

pub fn spawn_hanabi_particles_system(
    mut commands: Commands,
    mut particle_events: EventReader<SpawnParticles>,
    effects: Res<ParticleEffects>,
) {
    for event in particle_events.read() {
        let effect_handle = if event.config.organic_motion {
            effects.bioluminescent_trail.clone()
        } else {
            effects.explosion.clone()
        };

        commands.spawn((
            ParticleEffect::new(effect_handle),
            Transform::from_translation(event.position),
            HanabiParticleEffect {
                effect_type: ParticleEffectType::BioluminescentTrail,
                is_active: true,
            },
        ));
    }
}

// Note: The main particle spawning functions are now in particle_bridge.rs
// to avoid duplication and ambiguous re-exports