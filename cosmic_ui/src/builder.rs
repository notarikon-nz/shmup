// cosmic_ui/src/builder.rs
//! Fluent builder for creating optimized UI layouts

use bevy::prelude::*;
use crate::widgets::*;

/// Fluent builder for creating optimized UI layouts
pub struct WidgetBuilder<'a> {
    commands: &'a mut Commands<'a, 'a>,
    font_handle: Handle<Font>,
    current_entity: Option<Entity>,
}

impl<'a> WidgetBuilder<'a> {
    pub fn new(commands: &'a mut Commands<'a, 'a>, font_handle: Handle<Font>) -> Self {
        Self {
            commands,
            font_handle,
            current_entity: None,
        }
    }
    
    /// Create root container with optimal layout
    pub fn root(mut self) -> Self {
        let entity = self.commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(100), // Always on top
        )).id();
        
        self.current_entity = Some(entity);
        self
    }
    
    /// Add text display with automatic formatting
    pub fn text_display(self, initial_text: &str, position: UIPosition) -> (Self, TextDisplay) {
        let entity = self.commands.spawn((
            Text::new(initial_text),
            TextFont {
                font: self.font_handle.clone(),
                font_size: position.font_size,
                ..default()
            },
            TextColor(position.color),
            Node {
                position_type: PositionType::Absolute,
                left: position.left,
                top: position.top,
                bottom: position.bottom,
                right: position.right,
                ..default()
            },
        )).id();
        
        if let Some(parent) = self.current_entity {
            self.commands.entity(parent).add_child(entity);
        }
        
        let widget = TextDisplay::new(entity, initial_text.to_string());
        
        (self, widget)
    }
    
    /// Add progress bar with GPU-optimized rendering
    pub fn progress_bar(self, position: UIPosition, config: ProgressBarConfig) -> (Self, ProgressBar) {
        // Background
        let bg_entity = self.commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: position.left,
                top: position.top,
                width: Val::Px(config.width),
                height: Val::Px(config.height),
                border: UiRect::all(Val::Px(config.border_width)),
                ..default()
            },
            BackgroundColor(config.background_color),
            BorderColor(config.border_color),
        )).id();
        
        // Fill
        let fill_entity = self.commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(2.0),
                top: Val::Px(2.0),
                width: Val::Px(config.width - 4.0),
                height: Val::Px(config.height - 4.0),
                ..default()
            },
            BackgroundColor(config.fill_color),
        )).id();
        
        self.commands.entity(bg_entity).add_child(fill_entity);
        
        if let Some(parent) = self.current_entity {
            self.commands.entity(parent).add_child(bg_entity);
        }
        
        let widget = ProgressBar::new(bg_entity, fill_entity, config.max_value)
            .with_gradient(config.color_gradient);
        
        (self, widget)
    }
    
/// Add counter with prefix/suffix
    pub fn counter(self, prefix: &str, suffix: &str, position: UIPosition) -> (Self, Counter) {
        let initial_text = format!("{}{}{}", prefix, 0, suffix);
        let (builder, text_widget) = self.text_display(&initial_text, position);
        
        let widget = Counter::new(text_widget.entity, prefix.to_string(), suffix.to_string());
        
        (builder, widget)
    }
    
    /// Add status indicator with multiple states
    pub fn status_indicator(self, states: Vec<StatusState>, position: UIPosition) -> (Self, StatusIndicator) {
        let initial_text = states.first().map(|s| s.text.as_str()).unwrap_or("");
        let initial_color = states.first().map(|s| s.color).unwrap_or(Color::WHITE);
        
        let pos_with_color = UIPosition { color: initial_color, ..position };
        let (builder, text_widget) = self.text_display(initial_text, pos_with_color);
        
        let widget = StatusIndicator::new(text_widget.entity, states);
        
        (builder, widget)
    }
    
    /// Add notification queue
    pub fn notification_queue(self, position: UIPosition, max_visible: usize) -> (Self, NotificationQueue) {
        let container = self.commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: position.left,
                top: position.top,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                ..default()
            },
        )).id();
        
        if let Some(parent) = self.current_entity {
            self.commands.entity(parent).add_child(container);
        }
        
        let widget = NotificationQueue::new(container, max_visible);
        
        (self, widget)
    }
    
    /// Add info panel for multi-line text
    pub fn info_panel(self, max_lines: usize, position: UIPosition) -> (Self, InfoPanel) {
        let container = self.commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: position.left,
                top: position.top,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                ..default()
            },
        )).id();
        
        if let Some(parent) = self.current_entity {
            self.commands.entity(parent).add_child(container);
        }
        
        let widget = InfoPanel::new(container, max_lines);
        
        (self, widget)
    }
    
    /// Get the current root entity
    pub fn entity(&self) -> Option<Entity> {
        self.current_entity
    }
}

/// Position configuration for UI elements
#[derive(Clone, Debug)]
pub struct UIPosition {
    pub left: Val,
    pub top: Val,
    pub right: Val,
    pub bottom: Val,
    pub font_size: f32,
    pub color: Color,
}

impl Default for UIPosition {
    fn default() -> Self {
        Self {
            left: Val::Auto,
            top: Val::Auto,
            right: Val::Auto,
            bottom: Val::Auto,
            font_size: 18.0,
            color: Color::WHITE,
        }
    }
}

impl UIPosition {
    /// Top left corner positioning
    pub fn top_left() -> Self {
        Self {
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        }
    }
    
    /// Top right corner positioning
    pub fn top_right() -> Self {
        Self {
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        }
    }
    
    /// Bottom left corner positioning
    pub fn bottom_left() -> Self {
        Self {
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        }
    }
    
    /// Bottom right corner positioning
    pub fn bottom_right() -> Self {
        Self {
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        }
    }
    
    /// Center positioning
    pub fn center() -> Self {
        Self {
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            ..default()
        }
    }
    
    /// Add offset to current position
    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        match self.left {
            Val::Px(px) => self.left = Val::Px(px + x),
            Val::Percent(pct) => self.left = Val::Percent(pct + x),
            _ => {}
        }
        match self.top {
            Val::Px(px) => self.top = Val::Px(px + y),
            Val::Percent(pct) => self.top = Val::Percent(pct + y),
            _ => {}
        }
        self
    }
    
    /// Set font size
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    
    /// Set text color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Configuration for progress bars
#[derive(Clone, Debug)]
pub struct ProgressBarConfig {
    pub width: f32,
    pub height: f32,
    pub border_width: f32,
    pub background_color: Color,
    pub border_color: Color,
    pub fill_color: Color,
    pub max_value: f32,
    pub color_gradient: Vec<(f32, Color)>,
}

impl Default for ProgressBarConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            height: 20.0,
            border_width: 2.0,
            background_color: Color::srgb(0.1, 0.1, 0.1),
            border_color: Color::srgb(0.5, 0.5, 0.5),
            fill_color: Color::srgb(0.2, 0.8, 0.2),
            max_value: 100.0,
            color_gradient: vec![
                (0.3, Color::srgb(0.8, 0.2, 0.2)), // Red at 30%
                (0.6, Color::srgb(0.8, 0.8, 0.2)), // Yellow at 60%
                (1.0, Color::srgb(0.2, 0.8, 0.2)), // Green at 100%
            ],
        }
    }
}

impl ProgressBarConfig {
    /// Create config for health bar
    pub fn health_bar() -> Self {
        Self {
            width: 200.0,
            height: 24.0,
            border_color: Color::srgb(0.4, 0.8, 0.6),
            fill_color: Color::srgb(0.2, 0.8, 0.4),
            color_gradient: vec![
                (0.0, Color::srgb(0.8, 0.2, 0.2)),   // Red at 0%
                (0.25, Color::srgb(0.8, 0.4, 0.2)),  // Orange at 25%
                (0.5, Color::srgb(0.8, 0.8, 0.2)),   // Yellow at 50%
                (0.75, Color::srgb(0.4, 0.8, 0.2)),  // Light green at 75%
                (1.0, Color::srgb(0.2, 0.8, 0.4)),   // Green at 100%
            ],
            ..default()
        }
    }
    
    /// Create config for energy/mana bar
    pub fn energy_bar() -> Self {
        Self {
            width: 180.0,
            height: 18.0,
            border_color: Color::srgb(0.2, 0.6, 1.0),
            fill_color: Color::srgb(0.1, 0.5, 1.0),
            color_gradient: vec![
                (0.0, Color::srgb(0.2, 0.2, 0.6)),   // Dark blue at 0%
                (0.5, Color::srgb(0.2, 0.4, 0.8)),   // Medium blue at 50%
                (1.0, Color::srgb(0.1, 0.6, 1.0)),   // Bright blue at 100%
            ],
            ..default()
        }
    }
    
    /// Create config for biological theme
    pub fn biological() -> Self {
        Self {
            width: 200.0,
            height: 20.0,
            border_color: Color::srgb(0.3, 0.8, 0.6),
            background_color: Color::srgb(0.05, 0.15, 0.1),
            fill_color: Color::srgb(0.2, 0.8, 0.4),
            color_gradient: vec![
                (0.0, Color::srgb(0.6, 0.2, 0.2)),   // Toxin red
                (0.3, Color::srgb(0.8, 0.6, 0.2)),   // Warning amber
                (0.7, Color::srgb(0.4, 0.8, 0.3)),   // Healing green
                (1.0, Color::srgb(0.2, 1.0, 0.6)),   // Bioluminescent green
            ],
            ..default()
        }
    }
}
