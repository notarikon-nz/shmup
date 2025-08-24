use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

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

