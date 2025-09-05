pub const EVOLUTION_MENU_ITEMS: usize = 11;
pub const MENU_ITEM_HEIGHT: f32 = 40.0;
pub const MENU_PADDING: f32 = 20.0;
pub const MENU_WIDTH: f32 = 600.0;

// Missile trail constants
pub const MAX_TRAIL_SEGMENTS: usize = 8;
pub const TRAIL_SEGMENT_DISTANCE: f32 = 15.0;
pub const TRAIL_FADE_TIME: f32 = 0.8;
pub const TRAIL_WIDTH: f32 = 3.0;

// Upgrade costs
pub const MEMBRANE_REINFORCEMENT_COSTS: [u32; 5] = [10, 15, 25, 40, 60];
pub const WING_CANNON_COSTS: [u32; 5] = [25, 40, 60, 85, 120];
pub const MISSILE_SYSTEM_COSTS: [u32; 5] = [35, 55, 80, 110, 150];
pub const METABOLIC_COSTS: [u32; 5] = [15, 25, 40, 60, 90];
pub const CELLULAR_COSTS: [u32; 5] = [20, 35, 55, 80, 115];
pub const ENZYME_COSTS: [u32; 5] = [25, 45, 70, 100, 140];
pub const BIOLUMINESCENCE_COSTS: [u32; 3] = [30, 60, 100];
pub const MAGNET_RADIUS_COSTS: [u32; 4] = [25, 45, 70, 100];
pub const MAGNET_STRENGTH_COSTS: [u32; 4] = [30, 50, 75, 105];

// ===== CONSTANTS =====
pub const WING_CANNON_OFFSET: f32 = 25.0; // Distance from center
pub const WING_CANNON_Y_OFFSET: f32 = 10.0;
pub const MISSILE_LAUNCH_OFFSET: f32 = 20.0;
pub const MISSILE_Y_OFFSET: f32 = -15.0;

// Wing Cannon stats per level
pub const WING_CANNON_STATS: [(f32, i32, f32, u32); 5] = [
    (1.2, 25, 12.0, 2), // fire_rate, damage, size, pierce
    (1.0, 35, 14.0, 3),
    (0.8, 50, 16.0, 4),
    (0.6, 70, 18.0, 5),
    (0.5, 95, 20.0, 6),
];

// Missile system stats per level
pub const MISSILE_STATS: [(f32, i32, f32, f32, bool); 5] = [
    (2.5, 80, 400.0, 200.0, false), // fire_rate, damage, speed, range, dual
    (2.0, 110, 450.0, 250.0, false),
    (1.8, 150, 500.0, 300.0, false),
    (1.5, 200, 550.0, 350.0, true),
    (1.2, 260, 600.0, 400.0, true),
];