// src/physics.rs - Simple fluid dynamics helpers
use bevy::prelude::*;
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