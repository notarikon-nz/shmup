// src/pause_menu.rs
use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::input::*;

// ===== CONSTANTS =====
const MENU_ITEM_HEIGHT: f32 = 40.0;
const MENU_PADDING: f32 = 20.0;
const MENU_WIDTH: f32 = 600.0;
const EVOLUTION_MENU_ITEMS: usize = 11;

// Enhanced evolution costs and limits
const MEMBRANE_REINFORCEMENT_COSTS: [u32; 5] = [10, 15, 25, 40, 60];
const WING_CANNON_COSTS: [u32; 5] = [25, 40, 60, 85, 120];
const MISSILE_SYSTEM_COSTS: [u32; 5] = [35, 55, 80, 110, 150];
const METABOLIC_COSTS: [u32; 5] = [15, 25, 40, 60, 90];
const CELLULAR_COSTS: [u32; 5] = [20, 35, 55, 80, 115];
const ENZYME_COSTS: [u32; 5] = [25, 45, 70, 100, 140];
const BIOLUMINESCENCE_COSTS: [u32; 3] = [30, 60, 100];
const MAGNET_RADIUS_COSTS: [u32; 4] = [25, 45, 70, 100];
const MAGNET_STRENGTH_COSTS: [u32; 4] = [30, 50, 75, 105];

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct EvolutionMenuItem {
    pub index: usize,
    pub upgrade_type: UpgradeType,
}

#[derive(Component)]
pub struct MenuSelector;

#[derive(Clone, Debug)]
pub enum UpgradeType {
    MembraneReinforcement,
    WingCannons,
    MissileSystem,
    MetabolicEnhancement,
    CellularIntegrity,
    EnzymeProduction,
    Bioluminescence,
    EmergencySpore,
    MagnetRadius,
    MagnetStrength,
    ExitMenu,
}

#[derive(Resource, Default)]
pub struct PauseMenuState {
    pub selected_index: usize,
    pub menu_active: bool,
}

impl UpgradeType {
    pub fn get_display_info(&self, limits: &UpgradeLimits, atp: u32) -> (String, String, u32, bool, bool) {
        match self {
            UpgradeType::MembraneReinforcement => {
                let level = limits.damage_level as usize;
                let cost = if level < MEMBRANE_REINFORCEMENT_COSTS.len() { MEMBRANE_REINFORCEMENT_COSTS[level] } else { 0 };
                (
                    format!("Membrane Reinforcement [{}/{}]", limits.damage_level, limits.damage_max),
                    "Enhanced projectile damage and size. Adds additional projectiles at higher tiers.".to_string(),
                    cost,
                    atp >= cost,
                    limits.damage_level < limits.damage_max
                )
            }
            UpgradeType::WingCannons => {
                let level = limits.wing_cannon_level as usize;
                let cost = if level < WING_CANNON_COSTS.len() { WING_CANNON_COSTS[level] } else { 0 };
                (
                    format!("Wing Cannons [{}/{}]", limits.wing_cannon_level, limits.wing_cannon_max),
                    "Side-mounted piercing cannons. Yellow to green projectile evolution.".to_string(),
                    cost,
                    atp >= cost,
                    limits.wing_cannon_level < limits.wing_cannon_max
                )
            }
            UpgradeType::MissileSystem => {
                let level = limits.missile_level as usize;
                let cost = if level < MISSILE_SYSTEM_COSTS.len() { MISSILE_SYSTEM_COSTS[level] } else { 0 };
                (
                    format!("Missile System [{}/{}]", limits.missile_level, limits.missile_max),
                    "Auto-targeting guided missiles. Prioritizes stronger enemies.".to_string(),
                    cost,
                    atp >= cost,
                    limits.missile_level < limits.missile_max
                )
            }
            UpgradeType::MetabolicEnhancement => {
                let level = limits.metabolic_level as usize;
                let cost = if level < METABOLIC_COSTS.len() { METABOLIC_COSTS[level] } else { 0 };
                (
                    format!("Metabolic Enhancement [{}/{}]", limits.metabolic_level, limits.metabolic_max),
                    "Increased movement speed and fire rate efficiency.".to_string(),
                    cost,
                    atp >= cost,
                    limits.metabolic_level < limits.metabolic_max
                )
            }
            UpgradeType::CellularIntegrity => {
                let level = limits.cellular_level as usize;
                let cost = if level < CELLULAR_COSTS.len() { CELLULAR_COSTS[level] } else { 0 };
                (
                    format!("Cellular Integrity [{}/{}]", limits.cellular_level, limits.cellular_max),
                    "Increased maximum health points for better survival.".to_string(),
                    cost,
                    atp >= cost,
                    limits.cellular_level < limits.cellular_max
                )
            }
            UpgradeType::EnzymeProduction => {
                let level = limits.enzyme_level as usize;
                let cost = if level < ENZYME_COSTS.len() { ENZYME_COSTS[level] } else { 0 };
                (
                    format!("Enzyme Production [{}/{}]", limits.enzyme_level, limits.enzyme_max),
                    "Immunity to environmental toxins and enhanced projectile effects.".to_string(),
                    cost,
                    atp >= cost,
                    limits.enzyme_level < limits.enzyme_max
                )
            }
            UpgradeType::Bioluminescence => {
                let level = limits.bioluminescence_level as usize;
                let cost = if level < BIOLUMINESCENCE_COSTS.len() { BIOLUMINESCENCE_COSTS[level] } else { 0 };
                (
                    format!("Bioluminescence [{}/{}]", limits.bioluminescence_level, limits.bioluminescence_max),
                    "Enhanced coordination and visual effects for all abilities.".to_string(),
                    cost,
                    atp >= cost,
                    limits.bioluminescence_level < limits.bioluminescence_max
                )
            }
            UpgradeType::EmergencySpore => {
                (
                    "Emergency Spore [+1]".to_string(),
                    "Additional emergency reproductive blast charge.".to_string(),
                    20,
                    atp >= 20,
                    true // Always available if you have ATP
                )
            }
            UpgradeType::MagnetRadius => {
                let level = limits.magnet_radius_level as usize;
                let cost = if level < MAGNET_RADIUS_COSTS.len() { MAGNET_RADIUS_COSTS[level] } else { 0 };
                (
                    format!("ATP Absorption Range [{}/{}]", limits.magnet_radius_level, limits.magnet_radius_max),
                    "Increases ATP collection radius by 25px per level.".to_string(),
                    cost,
                    atp >= cost,
                    limits.magnet_radius_level < limits.magnet_radius_max
                )
            }
            UpgradeType::MagnetStrength => {
                let level = limits.magnet_strength_level as usize;
                let cost = if level < MAGNET_STRENGTH_COSTS.len() { MAGNET_STRENGTH_COSTS[level] } else { 0 };
                (
                    format!("ATP Absorption Force [{}/{}]", limits.magnet_strength_level, limits.magnet_strength_max),
                    "Increases magnetic pull force for faster ATP collection.".to_string(),
                    cost,
                    atp >= cost,
                    limits.magnet_strength_level < limits.magnet_strength_max
                )
            }
            UpgradeType::ExitMenu => {
                ("Exit Evolution Chamber".to_string(), "Resume cellular activities".to_string(), 0, true, true)
            }
        }
    }
}

pub fn setup_enhanced_pause_menu(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    player_query: Query<(&ATP, &UpgradeLimits), With<Player>>,
) {
    if let Ok((atp, limits)) = player_query.single() {
        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.1, 0.05, 0.95)),
            PauseMenuRoot,
        )).with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(MENU_WIDTH),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(MENU_PADDING)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.2, 0.15, 0.98)),
                BorderColor(Color::srgb(0.4, 0.8, 0.6)),
            )).with_children(|menu| {
                // Title
                menu.spawn((
                    Text::new("EVOLUTION CHAMBER"),
                    TextFont { font: fonts.default_font.clone(), font_size: 28.0, ..default() },
                    TextColor(Color::srgb(0.3, 1.0, 0.7)),
                    Node { margin: UiRect::bottom(Val::Px(10.0)), align_self: AlignSelf::Center, ..default() },
                ));
                
                // ATP Display
                menu.spawn((
                    Text::new(&format!("Available ATP: {}⚡", atp.amount)),
                    TextFont { font: fonts.default_font.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::srgb(1.0, 1.0, 0.3)),
                    Node { margin: UiRect::bottom(Val::Px(20.0)), align_self: AlignSelf::Center, ..default() },
                ));

                // Menu Items
                let upgrade_types = vec![
                    UpgradeType::MembraneReinforcement,
                    UpgradeType::WingCannons,
                    UpgradeType::MissileSystem,
                    UpgradeType::MetabolicEnhancement,
                    UpgradeType::CellularIntegrity,
                    UpgradeType::EnzymeProduction,
                    UpgradeType::Bioluminescence,
                    UpgradeType::EmergencySpore,
                    UpgradeType::MagnetRadius,
                    UpgradeType::MagnetStrength,
                    UpgradeType::ExitMenu,
                ];

                for (index, upgrade_type) in upgrade_types.into_iter().enumerate() {
                    let (title, description, cost, can_afford, can_upgrade) = upgrade_type.get_display_info(&limits, atp.amount);
                    
                    let text_color = if index == 0 { // Selected item
                        Color::srgb(1.0, 1.0, 0.3)
                    } else if can_afford && can_upgrade {
                        Color::srgb(0.9, 1.0, 0.9)
                    } else if !can_upgrade {
                        Color::srgb(0.8, 0.8, 0.2) // Maxed out
                    } else {
                        Color::srgb(0.5, 0.6, 0.5) // Can't afford
                    };

                    let bg_color = if index == 0 {
                        Color::srgba(0.2, 0.4, 0.3, 0.8)
                    } else {
                        Color::srgba(0.0, 0.0, 0.0, 0.0)
                    };

                    menu.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(MENU_ITEM_HEIGHT),
                            padding: UiRect::all(Val::Px(8.0)),
                            margin: UiRect::bottom(Val::Px(4.0)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(bg_color),
                        EvolutionMenuItem { index, upgrade_type: upgrade_type.clone() },
                    )).with_children(|item| {
                        // Title with cost
                        let title_text = if matches!(upgrade_type, UpgradeType::ExitMenu) {
                            title
                        } else if cost > 0 {
                            format!("{} - {} ATP", title, cost)
                        } else {
                            title
                        };
                        
                        item.spawn((
                            Text::new(title_text),
                            TextFont { font: fonts.default_font.clone(), font_size: 16.0, ..default() },
                            TextColor(text_color),
                        ));
                        
                        // Description
                        if !matches!(upgrade_type, UpgradeType::ExitMenu) {
                            item.spawn((
                                Text::new(description),
                                TextFont { font: fonts.default_font.clone(), font_size: 12.0, ..default() },
                                TextColor(Color::srgb(0.8, 0.9, 0.8)),
                                Node { margin: UiRect::top(Val::Px(2.0)), ..default() },
                            ));
                        }
                    });

                    if index == 0 {
                        menu.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(10.0),
                                width: Val::Px(4.0),
                                height: Val::Px(MENU_ITEM_HEIGHT),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 1.0, 0.7)),
                            MenuSelector,
                        ));
                    }
                }

                // Controls
                menu.spawn((
                    Text::new("↑↓ Navigate | SPACE Select | ESC Resume"),
                    TextFont { font: fonts.default_font.clone(), font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.9, 0.8)),
                    Node { 
                        margin: UiRect::top(Val::Px(20.0)),
                        align_self: AlignSelf::Center,
                        ..default() 
                    },
                ));
            });
        });
    }
}

pub fn pause_menu_navigation_system(
    mut menu_state: ResMut<PauseMenuState>,
    input_manager: Res<InputManager>,
    mut menu_items: Query<(&mut BackgroundColor, &EvolutionMenuItem)>,
    mut selector: Query<&mut Node, With<MenuSelector>>,
    mut commands: Commands,
    mut player_query: Query<(&mut ATP, &mut UpgradeLimits, &mut EvolutionSystem, &mut CellularUpgrades), With<Player>>,
    current_state: Res<State<IsPaused>>,
    mut next_state: ResMut<NextState<IsPaused>>,
) {
    // if !menu_state.menu_active { return; }

    // Navigation
    if input_manager.just_pressed(InputAction::MoveUp) {
        menu_state.selected_index = if menu_state.selected_index == 0 {
            EVOLUTION_MENU_ITEMS - 1
        } else {
            menu_state.selected_index - 1
        };
    }
    
    if input_manager.just_pressed(InputAction::MoveDown) {
        menu_state.selected_index = (menu_state.selected_index + 1) % EVOLUTION_MENU_ITEMS;
    }

    // Update visual selection
    for (mut bg_color, menu_item) in menu_items.iter_mut() {
        *bg_color = if menu_item.index == menu_state.selected_index {
            BackgroundColor(Color::srgba(0.2, 0.4, 0.3, 0.8))
        } else {
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0))
        };
    }

    // Update selector position
    if let Ok(mut selector_node) = selector.single_mut() {
        let y_offset = menu_state.selected_index as f32 * (MENU_ITEM_HEIGHT + 4.0);
        selector_node.top = Val::Px(120.0 + y_offset); // Account for title and ATP display
    }

    // Selection
    if input_manager.just_pressed(InputAction::Shoot) { // Using Shoot as confirm
        if let Ok((mut atp, mut limits, mut evolution_system, mut upgrades)) = player_query.single_mut() {
            let selected_item = menu_items.iter()
                .find(|(_, item)| item.index == menu_state.selected_index)
                .map(|(_, item)| item.upgrade_type.clone());

            if let Some(upgrade_type) = selected_item {
                match upgrade_type {
                    UpgradeType::ExitMenu => {
                        menu_state.menu_active = false;
                        next_state.set(IsPaused::Running);
                    }
                    _ => {
                        process_upgrade(&upgrade_type, &mut atp, &mut limits, &mut evolution_system, &mut upgrades);
                    }
                }
            }
        }
    }

    // ESC to exit
    if input_manager.just_pressed(InputAction::Pause) {
        menu_state.menu_active = false;
        next_state.set(IsPaused::Running);
    }
}

fn process_upgrade(
    upgrade_type: &UpgradeType,
    atp: &mut ATP,
    limits: &mut UpgradeLimits,
    evolution_system: &mut EvolutionSystem,
    upgrades: &mut CellularUpgrades,
) {
    let (_, _, cost, can_afford, can_upgrade) = upgrade_type.get_display_info(&limits, atp.amount);
    
    if !can_afford || !can_upgrade { return; }

    atp.amount = atp.amount.saturating_sub(cost);

    match upgrade_type {
        UpgradeType::MembraneReinforcement => {
            limits.damage_level += 1;
            upgrades.damage_amplification *= 1.2; // 20% increase per level
        }
        UpgradeType::WingCannons => {
            limits.wing_cannon_level += 1;
            // Wing cannon functionality will be implemented in weapon systems
        }
        UpgradeType::MissileSystem => {
            limits.missile_level += 1;
            // Missile system functionality will be implemented in weapon systems
        }
        UpgradeType::MetabolicEnhancement => {
            limits.metabolic_level += 1;
            upgrades.metabolic_rate *= 1.15;
            upgrades.movement_efficiency *= 1.1;
        }
        UpgradeType::CellularIntegrity => {
            limits.cellular_level += 1;
            upgrades.max_health += 30;
        }
        UpgradeType::EnzymeProduction => {
            limits.enzyme_level += 1;
            evolution_system.cellular_adaptations.extremophile_traits = true;
        }
        UpgradeType::Bioluminescence => {
            limits.bioluminescence_level += 1;
            evolution_system.cellular_adaptations.biofilm_formation = true;
        }
        UpgradeType::EmergencySpore => {
            if evolution_system.emergency_spores < 5 { // Cap at 5
                evolution_system.emergency_spores += 1;
            }
        }
        UpgradeType::MagnetRadius => {
            limits.magnet_radius_level += 1;
            upgrades.magnet_radius += 25.0;
        }
        UpgradeType::MagnetStrength => {
            limits.magnet_strength_level += 1;
            upgrades.magnet_strength += 0.4;
        }
        UpgradeType::ExitMenu => {} // Handled above
    }
}

pub fn cleanup_pause_menu_system(
    mut commands: Commands,
    pause_menu_query: Query<Entity, With<PauseMenuRoot>>,
) {
    for entity in pause_menu_query.iter() {
        commands.entity(entity).despawn();
    }
}