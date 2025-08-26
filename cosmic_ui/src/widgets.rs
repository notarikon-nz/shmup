// cosmic_ui/src/widgets.rs
//! Widget implementations for Cosmic UI

use bevy::prelude::*;
use std::collections::VecDeque;

/// High-performance text display with automatic formatting
#[derive(Component, Clone, Debug)]
pub struct TextDisplay {
    pub entity: Entity,
    pub format: String,
    pub last_value_hash: u64,
}

impl TextDisplay {
    pub fn new(entity: Entity, format: String) -> Self {
        Self {
            entity,
            format,
            last_value_hash: 0,
        }
    }
}

/// Optimized progress bar with GPU-accelerated updates
#[derive(Component, Clone, Debug)]
pub struct ProgressBar {
    pub entity: Entity,
    pub fill_entity: Entity,
    pub max_value: f32,
    pub current_percent: f32,
    pub color_gradient: Vec<(f32, Color)>, // Threshold, Color pairs
}

impl ProgressBar {
    pub fn new(entity: Entity, fill_entity: Entity, max_value: f32) -> Self {
        Self {
            entity,
            fill_entity,
            max_value,
            current_percent: 1.0,
            color_gradient: vec![
                (0.3, Color::srgb(0.8, 0.2, 0.2)), // Red at 30%
                (0.6, Color::srgb(0.8, 0.8, 0.2)), // Yellow at 60%
                (1.0, Color::srgb(0.2, 0.8, 0.2)), // Green at 100%
            ],
        }
    }
    
    pub fn with_gradient(mut self, gradient: Vec<(f32, Color)>) -> Self {
        self.color_gradient = gradient;
        self
    }
}

/// Counter widget for numeric displays (lives, ammo, etc)
#[derive(Component, Clone, Debug)]
pub struct Counter {
    pub entity: Entity,
    pub prefix: String,
    pub suffix: String,
    pub last_value: i32,
}

impl Counter {
    pub fn new(entity: Entity, prefix: String, suffix: String) -> Self {
        Self {
            entity,
            prefix,
            suffix,
            last_value: 0,
        }
    }
}

/// Animated status indicator (pulsing, fading, etc)
#[derive(Component, Clone, Debug)]
pub struct StatusIndicator {
    pub entity: Entity,
    pub states: Vec<StatusState>,
    pub current_state: usize,
    pub animation_timer: f32,
}

impl StatusIndicator {
    pub fn new(entity: Entity, states: Vec<StatusState>) -> Self {
        Self {
            entity,
            states,
            current_state: 0,
            animation_timer: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StatusState {
    pub text: String,
    pub color: Color,
    pub animation: StatusAnimation,
}

#[derive(Clone, Debug)]
pub enum StatusAnimation {
    Static,
    Pulse { frequency: f32 },
    Flash { interval: f32 },
    Fade { duration: f32 },
}

/// Multi-line info panel with automatic layout
#[derive(Component, Clone, Debug)]
pub struct InfoPanel {
    pub entity: Entity,
    pub lines: Vec<Entity>,
    pub max_lines: usize,
}

impl InfoPanel {
    pub fn new(entity: Entity, max_lines: usize) -> Self {
        Self {
            entity,
            lines: Vec::new(),
            max_lines,
        }
    }
    
    pub fn add_line(&mut self, line_entity: Entity) {
        if self.lines.len() >= self.max_lines {
            self.lines.remove(0);
        }
        self.lines.push(line_entity);
    }
}

/// Notification system for temporary messages
#[derive(Component, Debug)]
pub struct NotificationQueue {
    pub container: Entity,
    pub notifications: VecDeque<Notification>,
    pub max_visible: usize,
}

impl NotificationQueue {
    pub fn new(container: Entity, max_visible: usize) -> Self {
        Self {
            container,
            notifications: VecDeque::new(),
            max_visible,
        }
    }
    
    pub fn push_notification(&mut self, notification: Notification) {
        if self.notifications.len() >= self.max_visible {
            self.notifications.pop_front();
        }
        self.notifications.push_back(notification);
    }
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub entity: Entity,
    pub message: String,
    pub severity: NotificationLevel,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub fade_time: f32,
}

impl Notification {
    pub fn new(entity: Entity, message: String, severity: NotificationLevel, lifetime: f32) -> Self {
        Self {
            entity,
            message,
            severity,
            lifetime,
            max_lifetime: lifetime,
            fade_time: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Critical,
    Achievement,
}

impl NotificationLevel {
    pub fn color(&self) -> Color {
        match self {
            NotificationLevel::Info => Color::srgb(0.3, 0.8, 1.0),
            NotificationLevel::Warning => Color::srgb(1.0, 0.8, 0.2),
            NotificationLevel::Critical => Color::srgb(1.0, 0.2, 0.2),
            NotificationLevel::Achievement => Color::srgb(1.0, 0.8, 0.2),
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            NotificationLevel::Info => "ℹ️",
            NotificationLevel::Warning => "⚠️",
            NotificationLevel::Critical => "🚨",
            NotificationLevel::Achievement => "🏆",
        }
    }
}
