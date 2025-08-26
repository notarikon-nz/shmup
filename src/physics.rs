// src/physics.rs - Simple fluid dynamics helpers
use bevy::prelude::*;
use crate::resources::*;
use crate::resources::FluidEnvironment;

/// Convert world position to grid coordinates for fluid simulation
#[inline]
pub fn world_to_grid_pos(world_pos: Vec2, fluid_env: &FluidEnvironment) -> (usize, usize) {
    let grid_x = ((world_pos.x + 640.0) / fluid_env.cell_size).clamp(0.0, (fluid_env.grid_size - 1) as f32) as usize;
    let grid_y = ((world_pos.y + 360.0) / fluid_env.cell_size).clamp(0.0, (fluid_env.grid_size - 1) as f32) as usize;
    (grid_x, grid_y)
}

/// Sample fluid current at grid position with bounds checking
#[inline]
pub fn sample_current(fluid_env: &FluidEnvironment, grid_pos: (usize, usize)) -> Vec2 {
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