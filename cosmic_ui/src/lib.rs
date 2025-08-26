// cosmic_ui/src/lib.rs
//! Cosmic UI Framework - Ultra-performant declarative UI for Bevy games
//! Zero-cost abstractions with compile-time generation

use bevy::prelude::*;

pub mod prelude {
    pub use super::{CosmicUIPlugin, GameHUD, UIBinding, UIUpdateScheduler};
    pub use super::widgets::*;
    pub use super::bindings::*;
    pub use super::builder::*;
    pub use super::systems::*;
    pub use cosmic_ui_derive::*;
}

pub mod widgets;
pub mod bindings;
pub mod systems;
pub mod builder;

// Re-export for easier usage
pub use widgets::*;
pub use bindings::*;
pub use builder::*;
pub use systems::*;

/// Main plugin for Cosmic UI framework
pub struct CosmicUIPlugin;

impl Plugin for CosmicUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UIUpdateScheduler>()
            .init_resource::<UIPerformanceMetrics>()
            .add_systems(PreUpdate, cosmic_ui_change_detection)
            .add_systems(Update, cosmic_ui_batch_updates)
            .add_systems(PostUpdate, cosmic_ui_cleanup);
    }
}

/// Resource for batching UI updates for maximum performance
#[derive(Resource, Default)]
pub struct UIUpdateScheduler {
    pub pending_updates: Vec<UIUpdateCommand>,
    pub frame_budget_us: u64,
}

impl UIUpdateScheduler {
    pub fn new() -> Self {
        Self {
            pending_updates: Vec::new(),
            frame_budget_us: 500, // 0.5ms budget per frame
        }
    }
    
    pub fn queue_update(&mut self, update: UIUpdateCommand) {
        self.pending_updates.push(update);
    }
}

#[derive(Debug, Clone)]
pub enum UIUpdateCommand {
    TextUpdate { entity: Entity, text: String },
    StyleUpdate { entity: Entity, style: Node },
    ColorUpdate { entity: Entity, color: Color },
    VisibilityUpdate { entity: Entity, visible: bool },
}

/// Trait for game HUDs with automatic binding generation
pub trait GameHUD: Component + Sized {
    /// Called once during spawn to create UI entities
    fn spawn_ui(commands: &mut Commands, font_handle: Handle<Font>) -> Entity;
    
    /// Generated method that creates optimized update systems
    fn register_systems(app: &mut App);
    
    /// Update method called when bound data changes
    fn update_bindings(&mut self, world: &World, ui_root: Entity);
}

/// Compile-time UI binding with change detection
pub struct UIBinding<T, W> {
    _phantom: std::marker::PhantomData<(T, W)>,
}

/// Performance metrics tracking
#[derive(Resource, Default)]
pub struct UIPerformanceMetrics {
    pub frame_time_us: u64,
    pub updates_per_frame: usize,
    pub widgets_active: usize,
    pub memory_usage_kb: usize,
}

/// Extension trait for easy HUD registration
pub trait AppUIExtensions {
    fn register_hud<T: GameHUD + Component>(&mut self) -> &mut Self;
}

impl AppUIExtensions for App {
    fn register_hud<T: GameHUD + Component>(&mut self) -> &mut Self {
        T::register_systems(self);
        self
    }
}