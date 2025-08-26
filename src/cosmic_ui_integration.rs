// src/cosmic_ui_integration.rs
// Complete integration example showing how to use Cosmic UI in your game

use bevy::prelude::*;
use cosmic_ui::prelude::*;

// ===== REPLACE YOUR EXISTING UI SYSTEMS =====

// OLD: Your current complex UI system
/*
pub fn update_biological_ui(
    game_score: Res<GameScore>,
    player_query: Query<(&Player, &ATP, &EvolutionSystem)>,
    // ... 20 more query parameters with complex constraints
    mut atp_query: Query<&mut Text, (With<ATPText>, Without<EvolutionText>, ...)>,
    // ... tons of boilerplate
) {
    // Hundreds of lines of manual UI updates
}
*/

// NEW: Cosmic UI declarative approach
#[derive(Component, GameHUD)]
pub struct BiologicalGameHUD {
    #[bind(PlayerLives)]
    #[format("Lives: {}")]
    #[position(bottom_left)]
    lives: Counter,
    
    #[bind(PlayerATP)]
    #[format("ATP: {}⚡")]
    #[position(top_left)]
    #[style(color = "yellow", font_size = 20.0)]
    atp: TextDisplay,
    
    #[bind(PlayerHealth)]
    #[position(bottom_left, offset_y = 30)]
    health: ProgressBar,
    
    #[bind(GameScore)]
    #[format("Score: {} ({}x)")]
    #[position(top_right)]
    score: TextDisplay,
    
    #[bind(CellWallTimer)]
    #[format("🛡️ Cell Wall: {:.1}s")]
    #[position(bottom_left, offset_y = 130)]
    cell_wall: TextDisplay,
    
    #[bind(EnvironmentStatus)]
    #[format("pH: {:.1} | O2: {:.0}%")]
    #[position(top_left, offset_y = 30)]
    environment: TextDisplay,
    
    #[bind(EcosystemHealth)]
    #[position(bottom_right)]
    ecosystem_status: StatusIndicator,
    
    #[position(top_right, offset_y = 100)]
    notifications: NotificationQueue,
}

// ===== AUTOMATIC SYSTEM GENERATION =====
// The derive macro generates all the update systems automatically!
// No more manual query constraints or update logic needed.

// ===== HOW TO INTEGRATE INTO YOUR GAME =====

pub fn integrate_cosmic_ui(app: &mut App) {
    app
        // Add the Cosmic UI plugin
        .add_plugins(CosmicUIPlugin)
        
        // Register your HUD (this generates all update systems automatically)
        .register_hud::<BiologicalGameHUD>()
        
        // Configure performance settings
        .insert_resource(UIPerformanceConfig {
            frame_budget_microseconds: 500, // 0.5ms budget for UI updates
            batch_size: 100, // Max updates per frame
            change_detection_frequency: 1, // Every frame
        })
        
        // Add startup system to spawn HUD
        .add_systems(Startup, spawn_game_hud);
}

fn spawn_game_hud(
    mut commands: Commands,
    fonts: Res<crate::resources::GameFonts>,
) {
    // Spawn the HUD - all UI elements are created automatically
    BiologicalGameHUD::spawn_ui(&mut commands, fonts.default_font.clone());
}

// ===== EXTERNAL TOOLING SUPPORT =====

// Hot-reload configuration file
// cosmic_ui.toml - can be edited while game is running
const UI_CONFIG: &str = r#"
[hud.biological]
theme = "organic"
layout = "shooter"

[hud.biological.atp]
position = { x = 20, y = 20 }
font_size = 22.0
color = "#FFFF55"
glow = true

[hud.biological.health]
position = { x = 20, y = 680 }
width = 250
height = 25
gradient = [
    { threshold = 0.0, color = "#FF4444" },
    { threshold = 0.3, color = "#FFAA44" },
    { threshold = 1.0, color = "#44FF44" }
]

[hud.biological.notifications]
max_visible = 5
fade_time = 2.0
slide_animation = true

[performance]
frame_budget_us = 500
batch_updates = true
use_dirty_flags = true
"#;

// External theme system
#[derive(Resource)]
pub struct UITheme {
    pub colors: UIColorPalette,
    pub fonts: UIFontConfig,
    pub animations: UIAnimationConfig,
}

#[derive(Clone)]
pub struct UIColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
}

// Theme presets for different game moods
impl UIColorPalette {
    pub fn organic_theme() -> Self {
        Self {
            primary: Color::srgb(0.2, 0.8, 0.4),     // Chlorophyll green
            secondary: Color::srgb(0.4, 1.0, 0.8),   // Bioluminescent cyan
            success: Color::srgb(0.1, 0.9, 0.3),     // Healthy green
            warning: Color::srgb(0.9, 0.7, 0.2),     // Caution amber
            danger: Color::srgb(0.9, 0.2, 0.2),      // Toxic red
            info: Color::srgb(0.3, 0.8, 1.0),        // Water blue
        }
    }
    
    pub fn cyberpunk_theme() -> Self {
        Self {
            primary: Color::srgb(0.8, 0.2, 0.8),     // Neon magenta
            secondary: Color::srgb(0.2, 0.8, 0.8),   // Cyan
            success: Color::srgb(0.0, 1.0, 0.0),     // Matrix green
            warning: Color::srgb(1.0, 0.8, 0.0),     // Electric yellow
            danger: Color::srgb(1.0, 0.0, 0.4),      // Hot pink
            info: Color::srgb(0.4, 0.8, 1.0),        // Electric blue
        }
    }
}

// ===== PERFORMANCE MONITORING =====

#[derive(Resource)]
pub struct UIPerformanceMetrics {
    pub frame_time_us: u64,
    pub updates_per_frame: usize,
    pub widgets_active: usize,
    pub memory_usage_kb: usize,
}

pub fn ui_performance_system(
    mut metrics: ResMut<UIPerformanceMetrics>,
    scheduler: Res<UIUpdateScheduler>,
    hud_query: Query<&BiologicalGameHUD>,
) {
    // Track performance metrics for optimization
    metrics.updates_per_frame = scheduler.pending_updates.len();
    metrics.widgets_active = hud_query.iter().len() * 8; // Approximate widget count
    
    // Log performance warnings
    if metrics.frame_time_us > 1000 {
        warn!("UI frame time exceeded 1ms: {}μs", metrics.frame_time_us);
    }
}

// ===== NOTIFICATION SYSTEM =====

#[derive(Resource)]
pub struct GameNotifications {
    queue: Vec<GameNotification>,
}

pub struct GameNotification {
    pub message: String,
    pub level: NotificationLevel,
    pub duration: f32,
    pub icon: Option<String>,
}

impl GameNotifications {
    pub fn achievement_unlocked(&mut self, name: &str) {
        self.queue.push(GameNotification {
            message: format!("🏆 Achievement Unlocked: {}", name),
            level: NotificationLevel::Achievement,
            duration: 4.0,
            icon: Some("🏆".to_string()),
        });
    }
    
    pub fn evolution_available(&mut self, evolution: &str) {
        self.queue.push(GameNotification {
            message: format!("🧬 Evolution Available: {}", evolution),
            level: NotificationLevel::Info,
            duration: 3.0,
            icon: Some("🧬".to_string()),
        });
    }
    
    pub fn environmental_warning(&mut self, hazard: &str) {
        self.queue.push(GameNotification {
            message: format!("⚠️ Environmental Hazard: {}", hazard),
            level: NotificationLevel::Warning,
            duration: 5.0,
            icon: Some("⚠️".to_string()),
        });
    }
    
    pub fn critical_health(&mut self) {
        self.queue.push(GameNotification {
            message: "💀 Critical Health! Seek healing immediately!".to_string(),
            level: NotificationLevel::Critical,
            duration: 3.0,
            icon: Some("💀".to_string()),
        });
    }
}

// ===== COMPARISON: BEFORE AND AFTER =====

/*
BEFORE - Your current approach:
- 300+ lines of UI update code
- Complex query constraints with 10+ type parameters
- Manual text formatting everywhere  
- Separate systems for each UI element
- Hard to maintain and extend
- No theming support
- Performance issues with frequent updates

AFTER - Cosmic UI approach:
- 30 lines of declarative HUD definition
- Zero manual update code
- Automatic formatting and binding
- Single derive macro generates everything
- Easy to theme and customize
- Built-in performance optimization
- Hot-reload support
*/

// ===== ADVANCED FEATURES =====

// Conditional UI display
#[derive(Component)]
pub struct ConditionalWidget<T: Component> {
    pub widget: T,
    pub condition: Box<dyn Fn(&World) -> bool + Send + Sync>,
}

// Animated transitions
#[derive(Component)]
pub struct UIAnimation {
    pub animation_type: AnimationType,
    pub duration: f32,
    pub easing: EasingFunction,
}

pub enum AnimationType {
    FadeIn,
    FadeOut,
    SlideIn(Vec2),
    SlideOut(Vec2),
    Scale(Vec2),
    Rotate(f32),
}

pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}

// Multi-language support
#[derive(Resource)]
pub struct LocalizationManager {
    pub current_language: String,
    pub translations: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

impl LocalizationManager {
    pub fn translate(&self, key: &str) -> String {
        self.translations
            .get(&self.current_language)
            .and_then(|lang| lang.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

// Accessibility support
#[derive(Component)]
pub struct AccessibilityInfo {
    pub screen_reader_text: String,
    pub keyboard_shortcut: Option<KeyCode>,
    pub high_contrast_compatible: bool,
}

// ===== EXTERNAL DESIGN TOOL INTEGRATION =====

// JSON schema for external UI editors
pub const UI_SCHEMA: &str = r#"
{
  "version": "1.0",
  "hud": {
    "type": "object",
    "properties": {
      "widgets": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "id": {"type": "string"},
            "type": {"enum": ["TextDisplay", "Counter", "ProgressBar", "StatusIndicator"]},
            "position": {
              "type": "object",
              "properties": {
                "x": {"type": "number"},
                "y": {"type": "number"},
                "anchor": {"enum": ["top_left", "top_right", "bottom_left", "bottom_right", "center"]}
              }
            },
            "style": {
              "type": "object",
              "properties": {
                "font_size": {"type": "number"},
                "color": {"type": "string"},
                "background_color": {"type": "string"},
                "border_width": {"type": "number"}
              }
            },
            "binding": {"type": "string"},
            "format": {"type": "string"}
          }
        }
      }
    }
  }
}
"#;

// Export current UI layout for external editing
pub fn export_ui_layout(hud: &BiologicalGameHUD) -> String {
    serde_json::json!({
        "version": "1.0",
        "hud": {
            "widgets": [
                {
                    "id": "lives",
                    "type": "Counter", 
                    "position": {"anchor": "bottom_left", "x": 20, "y": 20},
                    "binding": "PlayerLives",
                    "format": "Lives: {}"
                },
                {
                    "id": "atp",
                    "type": "TextDisplay",
                    "position": {"anchor": "top_left", "x": 20, "y": 20},
                    "style": {"font_size": 20.0, "color": "#FFFF55"},
                    "binding": "PlayerATP", 
                    "format": "ATP: {}⚡"
                }
                // ... other widgets
            ]
        }
    }).to_string()
}

// ===== HOW TO REPLACE YOUR EXISTING SYSTEMS =====

pub fn migrate_from_old_ui(app: &mut App) {
    // 1. Remove old UI systems
    // app.remove_system(update_biological_ui);
    // app.remove_system(update_health_bar);
    // ... remove all your manual UI systems
    
    // 2. Add Cosmic UI
    app.add_plugins(CosmicUIPlugin);
    
    // 3. Replace manual UI spawn with HUD
    // Remove: setup_biological_ui system
    // Add: spawn_game_hud system
    
    // 4. Theme configuration
    app.insert_resource(UITheme {
        colors: UIColorPalette::organic_theme(),
        fonts: UIFontConfig::default(),
        animations: UIAnimationConfig::default(),
    });
    
    // 5. Performance monitoring (optional)
    app.add_systems(Update, ui_performance_system);
}

// ===== EXTERNAL TOOL DEFINITIONS =====

#[derive(Clone)]
pub struct UIFontConfig {
    pub default_size: f32,
    pub heading_size: f32,
    pub small_size: f32,
    pub font_family: String,
}

impl Default for UIFontConfig {
    fn default() -> Self {
        Self {
            default_size: 18.0,
            heading_size: 24.0,
            small_size: 14.0,
            font_family: "fonts/planetary_contact.ttf".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct UIAnimationConfig {
    pub enable_animations: bool,
    pub transition_duration: f32,
    pub notification_slide_speed: f32,
}

impl Default for UIAnimationConfig {
    fn default() -> Self {
        Self {
            enable_animations: true,
            transition_duration: 0.3,
            notification_slide_speed: 200.0,
        }
    }
}

#[derive(Resource)]
pub struct UIPerformanceConfig {
    pub frame_budget_microseconds: u64,
    pub batch_size: usize,
    pub change_detection_frequency: u32,
}

// Extension trait for easy HUD registration
pub trait AppUIExtensions {
    fn register_hud<T: GameHUD + Component>(&mut self) -> &mut Self;
}

impl AppUIExtensions for App {
    fn register_hud<T: GameHUD + Component>(&mut self) -> &mut Self {
        T::register_systems(self);
        self
    }
}