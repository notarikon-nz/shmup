// src/consolidated_pause_system.rs - Proper Bevy 0.16.1 SubState pause system
use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::input::*;
use crate::pause_menu::*;
use crate::constants::*;
use crate::weapon_systems::{setup_player_weapons};
use crate::despawn::*;

// ===== CONSOLIDATED PAUSE INPUT HANDLING =====
// This replaces all the scattered pause input handling across the codebase

pub fn unified_pause_input_system(
    input_manager: Res<InputManager>,
    current_pause_state: Res<State<IsPaused>>,
    current_game_state: Res<State<GameState>>,
    mut next_pause_state: ResMut<NextState<IsPaused>>,
    mut menu_state: ResMut<PauseMenuState>,
) {
    // Only handle pause input when in Playing state
    if current_game_state.get() != &GameState::Playing {
        return;
    }
    
    // Handle pause toggle
    if input_manager.just_pressed(InputAction::Pause) {
        match current_pause_state.get() {
            IsPaused::Running => {
                next_pause_state.set(IsPaused::Paused);
                menu_state.menu_active = true;
                menu_state.selected_index = 0; // Reset to first menu item
            },
            IsPaused::Paused => {
                next_pause_state.set(IsPaused::Running);
                menu_state.menu_active = false;
            },
        }
    }
}

// ===== PAUSE MENU NAVIGATION =====
// Consolidated and improved from pause_menu.rs

pub fn enhanced_pause_menu_navigation(
    mut commands: Commands,
    mut menu_state: ResMut<PauseMenuState>,
    input_manager: Res<InputManager>,
    mut menu_items: Query<(&mut BackgroundColor, &EvolutionMenuItem)>,
    mut selector: Query<&mut Node, With<MenuSelector>>,
    mut player_query: Query<(
        &mut ATP, 
        &mut UpgradeLimits, 
        &mut EvolutionSystem, 
        &mut CellularUpgrades
    ), With<Player>>,
    current_state: Res<State<IsPaused>>,
    mut next_state: ResMut<NextState<IsPaused>>,
) {
    // Only process navigation when actually paused
    if current_state.get() != &IsPaused::Paused || !menu_state.menu_active {
        return;
    }

    // Navigation with wrapping
    if input_manager.just_pressed(InputAction::MoveUp) {
        info!("pushed up");
        menu_state.selected_index = if menu_state.selected_index == 0 {
            EVOLUTION_MENU_ITEMS - 1
        } else {
            menu_state.selected_index - 1
        };
    }
    
    if input_manager.just_pressed(InputAction::MoveDown) {
        info!("pushed down");
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
        selector_node.top = Val::Px(120.0 + y_offset);
    }

    // Selection confirmation
    if input_manager.just_pressed(InputAction::Shoot) {
        info!("pushed select");
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
                        // new
                        process_evolution_upgrade(&upgrade_type, &mut atp, &mut limits, &mut evolution_system, &mut upgrades);
                    }
                }
            }
        }
    }

    // ESC or Pause key to exit menu
    if input_manager.just_pressed(InputAction::Pause) {
        menu_state.menu_active = false;
        next_state.set(IsPaused::Running);
    }
}

fn sync_weapon_components_to_upgrades(
    mut commands: Commands,
    mut player_query: Query<(Entity, &UpgradeLimits, Option<&mut WingCannon>, Option<&mut MissileSystem>), With<Player>>,
) {
    if let Ok((player_entity, limits, wing_cannon, missile_system)) = player_query.single_mut() {
        
        // Wing Cannon handling
        match (limits.wing_cannon_level, wing_cannon) {
            (0, Some(_)) => {
                // Level 0 but has component - remove it
                commands.entity(player_entity).remove::<WingCannon>();
            },
            (level @ 1..=5, Some(mut cannon)) => {
                // Has component, update stats
                let (fire_rate, damage, size, pierce) = WING_CANNON_STATS[(level - 1) as usize];
                cannon.level = level;
                cannon.fire_rate = fire_rate;
                cannon.damage = damage;
                cannon.projectile_size = size;
            },
            (level @ 1..=5, None) => {
                // Needs component, insert it
                let (fire_rate, damage, size, pierce) = WING_CANNON_STATS[(level - 1) as usize];
                commands.entity(player_entity).insert(WingCannon {
                    level,
                    fire_timer: 0.0,
                    fire_rate,
                    damage,
                    projectile_size: size,
                    side: WingCannonSide::Left, // Default to left
                });
            },
            _ => {} // Level 0 and no component - correct state
        }
        
        // Missile System handling (same pattern)
        match (limits.missile_level, missile_system) {
            (0, Some(_)) => {
                commands.entity(player_entity).remove::<MissileSystem>();
            },
            (level @ 1..=5, Some(mut system)) => {
                let (fire_rate, damage, speed, range, dual) = MISSILE_STATS[(level - 1) as usize];
                system.level = level;
                system.fire_rate = fire_rate;
                system.damage = damage;
                system.missile_speed = speed;
                system.homing_range = range;
                system.dual_launch = dual;
            },
            (level @ 1..=5, None) => {
                let (fire_rate, damage, speed, range, dual) = MISSILE_STATS[(level - 1) as usize];
                commands.entity(player_entity).insert(MissileSystem {
                    level,
                    fire_timer: 0.0,
                    fire_rate,
                    damage,
                    missile_speed: speed,
                    homing_range: range,
                    dual_launch: dual,
                });
            },
            _ => {}
        }
    }
}

// ===== UPGRADE PROCESSING =====
// Moved from pause_menu.rs and enhanced

fn process_evolution_upgrade(
    upgrade_type: &UpgradeType,
    atp: &mut ATP,
    limits: &mut UpgradeLimits,
    evolution_system: &mut EvolutionSystem,
    upgrades: &mut CellularUpgrades,
) {
    let (_, _, cost, can_afford, can_upgrade) = upgrade_type.get_display_info(&limits, atp.amount);
    
    if !can_afford || !can_upgrade { 
        info!("can't afford, or no upgrade left");
        return; 
    }

    // Deduct ATP cost
    atp.amount = atp.amount.saturating_sub(cost);

    // Apply upgrades with enhanced effects
    match upgrade_type {
        UpgradeType::MembraneReinforcement => {
            limits.damage_level += 1;
            upgrades.damage_amplification *= 1.25; // Increased from 1.2 for better progression
            
            // Additional projectiles at higher levels
            match limits.damage_level {
                2 => { /* Second projectile already handled in shooting system */ }
                4 => { /* Fourth projectile at level 4 */ }
                _ => { }
            }
        }
        
        UpgradeType::WingCannons => {
            info!("Wing Cannons Upgraded");
            limits.wing_cannon_level += 1;
            // Wing cannon stats are handled in weapon_systems.rs WING_CANNON_STATS
        }
        
        UpgradeType::MissileSystem => {
            info!("MissileSystem Upgraded");
            limits.missile_level += 1;
            // Missile stats are handled in weapon_systems.rs MISSILE_STATS
        }
        
        UpgradeType::MetabolicEnhancement => {
            info!("MetabolicEnhancement Upgraded");
            limits.metabolic_level += 1;
            upgrades.metabolic_rate *= 1.18; // Affects fire rate and movement
            upgrades.movement_efficiency *= 1.12;
        }
        
        UpgradeType::CellularIntegrity => {
            info!("CellularIntegrity Upgraded");
            limits.cellular_level += 1;
            let health_boost = 25 + (limits.cellular_level * 5); // Scaling health increase
            upgrades.max_health += health_boost as i32;
        }
        
        UpgradeType::EnzymeProduction => {
            info!("EnzymeProduction Upgraded");
            limits.enzyme_level += 1;
            evolution_system.cellular_adaptations.extremophile_traits = true;
            evolution_system.cellular_adaptations.membrane_permeability *= 1.15;
        }
        
        UpgradeType::Bioluminescence => {
            info!("Bioluminescence Upgraded");
            limits.bioluminescence_level += 1;
            evolution_system.cellular_adaptations.biofilm_formation = true;
            evolution_system.cellular_adaptations.chemoreceptor_sensitivity *= 1.2;
        }
        
        UpgradeType::EmergencySpore => {
            info!("EmergencySpore Upgraded");
            if evolution_system.emergency_spores < 7 { // Increased cap
                evolution_system.emergency_spores += 1;
            }
        }
        
        UpgradeType::MagnetRadius => {
            info!("Magnet Radius Upgraded");
            limits.magnet_radius_level += 1;
            upgrades.magnet_radius += 30.0; // Increased from 25.0
        }
        
        UpgradeType::MagnetStrength => {
            info!("Magnet Strength Upgraded");
            limits.magnet_strength_level += 1;
            upgrades.magnet_strength += 0.5; // Increased from 0.4
        }
        
        UpgradeType::ExitMenu => {} // Handled above
    }

    
}

// ===== PAUSE STATE MANAGEMENT =====
// Replaces scattered OnEnter/OnExit systems

pub fn on_pause_enter(
    mut commands: Commands,
    fonts: Res<GameFonts>,
    player_query: Query<(&ATP, &UpgradeLimits), With<Player>>,
    mut menu_state: ResMut<PauseMenuState>,
) {
    menu_state.menu_active = true;
    menu_state.selected_index = 0;
    
    // Set up pause menu UI
    setup_enhanced_pause_menu(commands, fonts, player_query);
}

pub fn on_pause_exit(
    mut commands: Commands,
    pause_menu_query: Query<Entity, With<PauseMenuRoot>>,
    mut menu_state: ResMut<PauseMenuState>,
) {
    menu_state.menu_active = false;
    
    // Clean up pause menu UI
    for entity in pause_menu_query.iter() {
        commands.entity(entity)
            .safe_despawn();
    }
}

// ===== INPUT BLOCKING =====
// Prevents game input during pause

pub fn pause_input_blocking_system(
    mut input_manager: ResMut<InputManager>,
    current_state: Res<State<IsPaused>>,
) {
    match current_state.get() {
        IsPaused::Paused => {
            // Block all game inputs except pause and menu navigation
            input_manager.blocked_actions = vec![
                InputAction::MoveLeft,
                InputAction::MoveRight, 
                // don't block these, they're our upgrade menu selection
                //InputAction::MoveUp,
                //InputAction::MoveDown,
                //InputAction::Shoot,
                InputAction::EmergencySpore,
                InputAction::Restart,
            ];
            
            // Keep debug inputs blocked in pause
            if input_manager.debug_enabled {
                input_manager.blocked_actions.extend([
                    InputAction::DebugSpawnATP,
                    InputAction::DebugSpawnEvolutionChamber,
                    InputAction::DebugTriggerKingTide,
                ]);
            }
        }
        IsPaused::Running => {
            // Unblock all inputs when unpaused
            input_manager.blocked_actions.clear();
        }
    }
}

// ===== PLUGIN SETUP =====
// This replaces the scattered pause system setup

pub struct ConsolidatedPausePlugin;

impl Plugin for ConsolidatedPausePlugin {
    fn build(&self, app: &mut App) {
        app
            // Remove duplicate pause systems and consolidate
            .add_systems(Update, (
                unified_pause_input_system,
                pause_input_blocking_system,
            ).run_if(in_state(GameState::Playing)))
            
            // Pause state transitions using proper SubState system
            .add_systems(OnEnter(IsPaused::Paused), on_pause_enter)
            .add_systems(OnExit(IsPaused::Paused), (
                sync_weapon_components_to_upgrades,
                on_pause_exit,
            ))
            
            // Menu navigation only when paused
            .add_systems(Update, (
                enhanced_pause_menu_navigation,
                // update_pause_menu_text,
            )
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(IsPaused::Paused)));
    }
}

// ===== PERFORMANCE MONITORING =====
// Optional: Monitor pause system performance

pub fn pause_performance_monitor(
    pause_menu_query: Query<Entity, With<PauseMenuRoot>>,
    menu_items: Query<&EvolutionMenuItem>,
    current_state: Res<State<IsPaused>>,
) {
    if current_state.get() == &IsPaused::Paused {
        let menu_entities = pause_menu_query.iter().count();
        let menu_item_count = menu_items.iter().count();
        
        if menu_entities > 1 {
            warn!("Multiple pause menus detected: {}", menu_entities);
        }
        
        if menu_item_count != EVOLUTION_MENU_ITEMS {
            warn!("Menu item count mismatch: expected {}, found {}", 
                   EVOLUTION_MENU_ITEMS, menu_item_count);
        }
    }
}
