// cosmic_ui/src/systems.rs
//! ECS systems for high-performance UI updates

use bevy::prelude::*;
use std::time::Instant;
use crate::*;

/// Change detection system - runs in PreUpdate for maximum efficiency
pub fn cosmic_ui_change_detection(
    // This system would be populated by the proc macro based on HUD bindings
    // For now, it's a placeholder that the derive macro would fill in
) {
    // Generated change detection queries go here
}

/// Batch update system - processes all UI updates in one frame
pub fn cosmic_ui_batch_updates(
    mut commands: Commands,
    mut scheduler: ResMut<UIUpdateScheduler>,
    mut metrics: ResMut<UIPerformanceMetrics>,
    mut text_query: Query<&mut Text>,
    mut node_query: Query<&mut Node>,
    mut background_color_query: Query<&mut BackgroundColor>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let start = Instant::now();
    let mut updates_processed = 0;
    
    // Process updates within frame budget
    while let Some(update) = scheduler.pending_updates.pop() {
        match update {
            UIUpdateCommand::TextUpdate { entity, text } => {
                if let Ok(mut text_component) = text_query.get_mut(entity) {
                    **text_component = text;
                    updates_processed += 1;
                }
            }
            UIUpdateCommand::StyleUpdate { entity, style } => {
                if let Ok(mut node_component) = node_query.get_mut(entity) {
                    *node_component = style;
                    updates_processed += 1;
                }
            }
            UIUpdateCommand::ColorUpdate { entity, color } => {
                if let Ok(mut color_component) = background_color_query.get_mut(entity) {
                    *color_component = BackgroundColor(color);
                    updates_processed += 1;
                }
            }
            UIUpdateCommand::VisibilityUpdate { entity, visible } => {
                if let Ok(mut visibility) = visibility_query.get_mut(entity) {
                    *visibility = if visible { Visibility::Visible } else { Visibility::Hidden };
                    updates_processed += 1;
                }
            }
        }
        
        // Frame budget check
        if start.elapsed().as_micros() as u64 > scheduler.frame_budget_us {
            break;
        }
    }
    
    // Update performance metrics
    metrics.frame_time_us = start.elapsed().as_micros() as u64;
    metrics.updates_per_frame = updates_processed;
}

/// Cleanup system for notification lifetimes and temporary UI
pub fn cosmic_ui_cleanup(
    time: Res<Time>,
    mut commands: Commands,
    mut notification_query: Query<&mut NotificationQueue>,
    mut status_query: Query<&mut StatusIndicator>,
) {
    // Update notification lifetimes
    for mut queue in notification_query.iter_mut() {
        queue.notifications.retain_mut(|notification| {
            notification.lifetime -= time.delta_secs();
            if notification.lifetime <= 0.0 {
                commands.entity(notification.entity).despawn_recursive();
                false
            } else {
                true
            }
        });
    }
    
    // Update status indicator animations
    for mut status in status_query.iter_mut() {
        status.animation_timer += time.delta_secs();
        
        if status.current_state < status.states.len() {
            let current_state = &status.states[status.current_state];
            match current_state.animation {
                StatusAnimation::Pulse { frequency } => {
                    if status.animation_timer >= 1.0 / frequency {
                        status.animation_timer = 0.0;
                        // Trigger pulse animation
                    }
                }
                StatusAnimation::Flash { interval } => {
                    if status.animation_timer >= interval {
                        status.animation_timer = 0.0;
                        // Trigger flash
                    }
                }
                StatusAnimation::Fade { duration } => {
                    if status.animation_timer >= duration {
                        // Move to next state or loop
                        status.current_state = (status.current_state + 1) % status.states.len();
                        status.animation_timer = 0.0;
                    }
                }
                StatusAnimation::Static => {
                    // No animation needed
                }
            }
        }
    }
}

/// System for updating progress bars with smooth animations
pub fn update_progress_bars(
    mut progress_bars: Query<&mut ProgressBar>,
    mut node_query: Query<&mut Node>,
    mut background_color_query: Query<&mut BackgroundColor>,
    time: Res<Time>,
) {
    for mut progress_bar in progress_bars.iter_mut() {
        // Smooth animation towards target percentage
        if let Ok(mut node) = node_query.get_mut(progress_bar.fill_entity) {
            let style = &mut node;
            let target_width = Val::Percent(progress_bar.current_percent * 100.0);
            
            // Smooth interpolation
            if let Val::Percent(current_width) = style.width {
                let target_percent = progress_bar.current_percent * 100.0;
                if (current_width - target_percent).abs() > 0.1 {
                    let new_width = current_width + (target_percent - current_width) * time.delta_secs() * 5.0;
                    style.width = Val::Percent(new_width);
                }
            }
        }
        
        // Update color based on current percentage
        if let Ok(mut color) = background_color_query.get_mut(progress_bar.fill_entity) {
            let new_color = calculate_gradient_color(&progress_bar.color_gradient, progress_bar.current_percent);
            color.0 = new_color;
        }
    }
}

/// System for animating status indicators
pub fn animate_status_indicators(
    mut indicators: Query<&mut StatusIndicator>,
    mut text_query: Query<&mut Text>,
    mut color_query: Query<&mut TextColor>,
    time: Res<Time>,
) {
    for mut indicator in indicators.iter_mut() {
        indicator.animation_timer += time.delta_secs();
        
        if indicator.current_state < indicator.states.len() {
            let state = &indicator.states[indicator.current_state];
            
            // Update text if changed
            if let Ok(mut text) = text_query.get_mut(indicator.entity) {
                if **text != state.text {
                    **text = state.text.clone();
                }
            }
            
            // Update color with animation
            if let Ok(mut text_color) = color_query.get_mut(indicator.entity) {
                let base_color = state.color;
                let animated_color = match state.animation {
                    StatusAnimation::Pulse { frequency } => {
                        let pulse = (indicator.animation_timer * frequency * 2.0 * std::f32::consts::PI).sin();
                        let intensity = 0.7 + pulse * 0.3;
                        Color::srgba(
                            base_color.to_srgba().red * intensity,
                            base_color.to_srgba().green * intensity,
                            base_color.to_srgba().blue * intensity,
                            base_color.to_srgba().alpha,
                        )
                    }
                    StatusAnimation::Flash { interval } => {
                        let flash_cycle = indicator.animation_timer % interval;
                        if flash_cycle < interval * 0.1 {
                            Color::WHITE
                        } else {
                            base_color
                        }
                    }
                    StatusAnimation::Fade { duration } => {
                        let fade_progress = (indicator.animation_timer % duration) / duration;
                        let alpha = 1.0 - fade_progress;
                        Color::srgba(
                            base_color.to_srgba().red,
                            base_color.to_srgba().green,
                            base_color.to_srgba().blue,
                            alpha,
                        )
                    }
                    StatusAnimation::Static => base_color,
                };
                text_color.0 = animated_color;
            }
        }
    }
}

/// Utility function to calculate gradient colors for progress bars
pub fn calculate_gradient_color(gradient: &[(f32, Color)], value: f32) -> Color {
    if gradient.is_empty() {
        return Color::WHITE;
    }
    
    if value <= gradient[0].0 {
        return gradient[0].1;
    }
    
    for window in gradient.windows(2) {
        let (low_threshold, low_color) = window[0];
        let (high_threshold, high_color) = window[1];
        
        if value <= high_threshold {
            let t = (value - low_threshold) / (high_threshold - low_threshold);
            return Color::srgba(
                low_color.to_srgba().red + t * (high_color.to_srgba().red - low_color.to_srgba().red),
                low_color.to_srgba().green + t * (high_color.to_srgba().green - low_color.to_srgba().green),
                low_color.to_srgba().blue + t * (high_color.to_srgba().blue - low_color.to_srgba().blue),
                low_color.to_srgba().alpha + t * (high_color.to_srgba().alpha - low_color.to_srgba().alpha),
            );
        }
    }
    
    gradient.last().unwrap().1
}