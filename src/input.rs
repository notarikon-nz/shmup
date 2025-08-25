use bevy::prelude::*;
use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::resources::{GameState};

// ===== INPUT ACTIONS =====
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputAction {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    
    // Combat
    Shoot,
    EmergencySpore,  // Space bar special attack
    
    // Game Control
    Pause,
    Restart,
    
    // Debug (remove in release)
    DebugSpawnATP,
    DebugSpawnEvolutionChamber,
    DebugTriggerKingTide,
    
    // Evolution Chamber Upgrades
    UpgradeDamage,         // 1
    UpgradeMetabolic,      // 2
    UpgradeCellular,       // 3
    UpgradeEnzyme,         // 4
    UpgradeBioluminescence,// 5
    UpgradeSpore,          // 6
    EvolvePseudopod,       // 7
    EvolveSymbiotic,       // 8
    EvolveBioluminescent,  // 9
}

// ===== INPUT BINDINGS =====
#[derive(Debug, Clone)]
pub struct KeyboardBinding {
    pub key: KeyCode,
    pub modifier: Option<KeyCode>, // For shift+key combinations
}

#[derive(Debug, Clone)]
pub struct GamepadBinding {
    pub button: Option<GamepadButton>,
    pub axis: Option<(GamepadAxis, f32)>, // axis and threshold
    pub axis_negative: bool, // for negative axis values
}

#[derive(Debug, Clone)]
pub struct InputBinding {
    pub keyboard: Option<KeyboardBinding>,
    pub gamepad: Option<GamepadBinding>,
    pub mouse: Option<MouseButton>,
}

// ===== INPUT STATE =====
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputState {
    Released,
    JustPressed,
    Pressed,
    JustReleased,
}

impl InputState {
    pub fn is_pressed(self) -> bool {
        matches!(self, InputState::Pressed | InputState::JustPressed)
    }
    
    pub fn just_pressed(self) -> bool {
        matches!(self, InputState::JustPressed)
    }
    
    pub fn just_released(self) -> bool {
        matches!(self, InputState::JustReleased)
    }
}

// ===== INPUT MANAGER RESOURCE =====
#[derive(Resource)]
pub struct InputManager {
    // Current frame input states
    pub current_states: HashMap<InputAction, InputState>,
    pub previous_states: HashMap<InputAction, InputState>,
    
    // Analog input values (for movement)
    pub analog_values: HashMap<InputAction, f32>,
    
    // Input bindings
    pub bindings: HashMap<InputAction, InputBinding>,
    
    // Connected gamepad
    pub active_gamepad: Option<Entity>,
    
    // AI override system
    pub ai_override: bool,
    pub ai_states: HashMap<InputAction, InputState>,
    pub ai_analog: HashMap<InputAction, f32>,
    
    // Input blocking (for UI, pause, etc.)
    pub blocked_actions: Vec<InputAction>,
    
    // Debug mode
    pub debug_enabled: bool,
}

impl Default for InputManager {
    fn default() -> Self {
        let mut manager = Self {
            current_states: HashMap::new(),
            previous_states: HashMap::new(),
            analog_values: HashMap::new(),
            bindings: HashMap::new(),
            active_gamepad: None,
            ai_override: false,
            ai_states: HashMap::new(),
            ai_analog: HashMap::new(),
            blocked_actions: Vec::new(),
            debug_enabled: cfg!(debug_assertions),
        };
        
        manager.setup_default_bindings();
        manager
    }
}

impl InputManager {
    fn setup_default_bindings(&mut self) {
        use InputAction::*;
        
        // Movement bindings
        self.bind_action(MoveLeft, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::KeyA, modifier: None }),
            gamepad: Some(GamepadBinding { 
                button: None, 
                axis: Some((GamepadAxis::LeftStickX, 0.1)), 
                axis_negative: true 
            }),
            mouse: None,
        });
        
        self.bind_action(MoveRight, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::KeyD, modifier: None }),
            gamepad: Some(GamepadBinding { 
                button: None, 
                axis: Some((GamepadAxis::LeftStickX, 0.1)), 
                axis_negative: false 
            }),
            mouse: None,
        });
        
        self.bind_action(MoveUp, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::KeyW, modifier: None }),
            gamepad: Some(GamepadBinding { 
                button: None, 
                axis: Some((GamepadAxis::LeftStickY, 0.1)), 
                axis_negative: false 
            }),
            mouse: None,
        });
        
        self.bind_action(MoveDown, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::KeyS, modifier: None }),
            gamepad: Some(GamepadBinding { 
                button: None, 
                axis: Some((GamepadAxis::LeftStickY, 0.1)), 
                axis_negative: true 
            }),
            mouse: None,
        });
        
        // Also bind arrow keys for movement
        self.add_keyboard_binding(MoveLeft, KeyCode::ArrowLeft);
        self.add_keyboard_binding(MoveRight, KeyCode::ArrowRight);
        self.add_keyboard_binding(MoveUp, KeyCode::ArrowUp);
        self.add_keyboard_binding(MoveDown, KeyCode::ArrowDown);
        
        // Combat bindings
        self.bind_action(Shoot, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::Space, modifier: None }),
            gamepad: Some(GamepadBinding { 
                button: Some(GamepadButton::RightTrigger2), 
                axis: None, 
                axis_negative: false 
            }),
            mouse: Some(MouseButton::Left),
        });
        
        self.bind_action(EmergencySpore, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::Space, modifier: Some(KeyCode::ShiftLeft) }),
            gamepad: Some(GamepadBinding { 
                button: Some(GamepadButton::South), 
                axis: None, 
                axis_negative: false 
            }),
            mouse: Some(MouseButton::Right),
        });
        
        // Game control
        self.bind_action(Pause, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::KeyP, modifier: None }),
            gamepad: Some(GamepadBinding { 
                button: Some(GamepadButton::Start), 
                axis: None, 
                axis_negative: false 
            }),
            mouse: None,
        });
        
        self.bind_action(Restart, InputBinding {
            keyboard: Some(KeyboardBinding { key: KeyCode::KeyR, modifier: None }),
            gamepad: Some(GamepadBinding { 
                button: Some(GamepadButton::Select), 
                axis: None, 
                axis_negative: false 
            }),
            mouse: None,
        });
        
        // Evolution chamber upgrades
        for (i, action) in [
            UpgradeDamage, UpgradeMetabolic, UpgradeCellular, UpgradeEnzyme,
            UpgradeBioluminescence, UpgradeSpore, EvolvePseudopod, 
            EvolveSymbiotic, EvolveBioluminescent
        ].iter().enumerate() {
            let key = match i {
                0 => KeyCode::Digit1,
                1 => KeyCode::Digit2,
                2 => KeyCode::Digit3,
                3 => KeyCode::Digit4,
                4 => KeyCode::Digit5,
                5 => KeyCode::Digit6,
                6 => KeyCode::Digit7,
                7 => KeyCode::Digit8,
                8 => KeyCode::Digit9,
                _ => KeyCode::Digit0,
            };
            
            self.bind_action(*action, InputBinding {
                keyboard: Some(KeyboardBinding { key, modifier: None }),
                gamepad: None,
                mouse: None,
            });
        }
        
        // Debug bindings (only in debug builds)
        if cfg!(debug_assertions) {
            self.bind_action(DebugSpawnATP, InputBinding {
                keyboard: Some(KeyboardBinding { key: KeyCode::F2, modifier: None }),
                gamepad: None,
                mouse: None,
            });
            
            self.bind_action(DebugSpawnEvolutionChamber, InputBinding {
                keyboard: Some(KeyboardBinding { key: KeyCode::F3, modifier: None }),
                gamepad: None,
                mouse: None,
            });
            
            self.bind_action(DebugTriggerKingTide, InputBinding {
                keyboard: Some(KeyboardBinding { key: KeyCode::F4, modifier: None }),
                gamepad: None,
                mouse: None,
            });
        }
    }
    
    // ===== BINDING MANAGEMENT =====
    pub fn bind_action(&mut self, action: InputAction, binding: InputBinding) {
        self.bindings.insert(action, binding);
    }
    
    pub fn add_keyboard_binding(&mut self, action: InputAction, key: KeyCode) {
        if let Some(binding) = self.bindings.get_mut(&action) {
            if binding.keyboard.is_none() {
                binding.keyboard = Some(KeyboardBinding { key, modifier: None });
            }
        }
    }
    
    pub fn clear_binding(&mut self, action: InputAction) {
        self.bindings.remove(&action);
    }
    
    // ===== INPUT QUERY METHODS =====
    pub fn action_state(&self, action: InputAction) -> InputState {
        if self.blocked_actions.contains(&action) {
            return InputState::Released;
        }
        
        if self.ai_override {
            return self.ai_states.get(&action).copied().unwrap_or(InputState::Released);
        }
        
        self.current_states.get(&action).copied().unwrap_or(InputState::Released)
    }
    
    pub fn action_value(&self, action: InputAction) -> f32 {
        if self.blocked_actions.contains(&action) {
            return 0.0;
        }
        
        if self.ai_override {
            return self.ai_analog.get(&action).copied().unwrap_or(0.0);
        }
        
        self.analog_values.get(&action).copied().unwrap_or(0.0)
    }
    
    pub fn pressed(&self, action: InputAction) -> bool {
        self.action_state(action).is_pressed()
    }
    
    pub fn just_pressed(&self, action: InputAction) -> bool {
        self.action_state(action).just_pressed()
    }
    
    pub fn just_released(&self, action: InputAction) -> bool {
        self.action_state(action).just_released()
    }
    
    // ===== MOVEMENT VECTOR HELPERS =====
    pub fn movement_vector(&self) -> Vec2 {
        let x = self.action_value(InputAction::MoveRight) - self.action_value(InputAction::MoveLeft);
        let y = self.action_value(InputAction::MoveUp) - self.action_value(InputAction::MoveDown);
        Vec2::new(x, y).clamp_length_max(1.0)
    }
    
    pub fn movement_digital(&self) -> Vec2 {
        let mut movement = Vec2::ZERO;
        
        if self.pressed(InputAction::MoveLeft) { movement.x -= 1.0; }
        if self.pressed(InputAction::MoveRight) { movement.x += 1.0; }
        if self.pressed(InputAction::MoveDown) { movement.y -= 1.0; }
        if self.pressed(InputAction::MoveUp) { movement.y += 1.0; }
        
        movement.clamp_length_max(1.0)
    }
    
    // ===== AI OVERRIDE SYSTEM =====
    pub fn enable_ai_override(&mut self) {
        self.ai_override = true;
    }
    
    pub fn disable_ai_override(&mut self) {
        self.ai_override = false;
        self.ai_states.clear();
        self.ai_analog.clear();
    }
    
    pub fn set_ai_action(&mut self, action: InputAction, state: InputState) {
        self.ai_states.insert(action, state);
    }
    
    pub fn set_ai_analog(&mut self, action: InputAction, value: f32) {
        self.ai_analog.insert(action, value);
    }
    
    // ===== INPUT BLOCKING =====
    pub fn block_action(&mut self, action: InputAction) {
        if !self.blocked_actions.contains(&action) {
            self.blocked_actions.push(action);
        }
    }
    
    pub fn unblock_action(&mut self, action: InputAction) {
        self.blocked_actions.retain(|&a| a != action);
    }
    
    pub fn block_all_input(&mut self) {
        use InputAction::*;
        self.blocked_actions = vec![
            MoveLeft, MoveRight, MoveUp, MoveDown, Shoot, EmergencySpore,
            Pause, Restart, UpgradeDamage, UpgradeMetabolic, UpgradeCellular,
            UpgradeEnzyme, UpgradeBioluminescence, UpgradeSpore, EvolvePseudopod,
            EvolveSymbiotic, EvolveBioluminescent
        ];
        
        if self.debug_enabled {
            self.blocked_actions.extend([DebugSpawnATP, DebugSpawnEvolutionChamber, DebugTriggerKingTide]);
        }
    }
    
    pub fn unblock_all_input(&mut self) {
        self.blocked_actions.clear();
    }
    
    // ===== GAMEPAD MANAGEMENT =====
    pub fn set_active_gamepad(&mut self, gamepad: Entity) {
        self.active_gamepad = Some(gamepad);
    }
    
    pub fn clear_active_gamepad(&mut self) {
        self.active_gamepad = None;
    }
}

// ===== INPUT UPDATE SYSTEM =====
pub fn input_update_system(
    mut input_manager: ResMut<InputManager>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepad_query: Query<(Entity, &Gamepad)>,
) {
    // Store previous states for edge detection
    input_manager.previous_states = input_manager.current_states.clone();
    
    // Auto-detect gamepad if none is active
    if input_manager.active_gamepad.is_none() {
        // Get the first gamepad entity
        if let Some((entity, _gamepad)) = gamepad_query.iter().next() {
            // Store the Entity directly as the gamepad identifier
            input_manager.set_active_gamepad(entity);
        }
    }
    
    // Update all input states
    for (&action, binding) in &input_manager.bindings.clone() {
        let mut pressed = false;
        let mut analog_value = 0.0f32;
        
        // Check keyboard input
        if let Some(kb_binding) = &binding.keyboard {
            let key_pressed = if let Some(modifier) = kb_binding.modifier {
                keyboard.pressed(modifier) && keyboard.pressed(kb_binding.key)
            } else {
                keyboard.pressed(kb_binding.key)
            };
            
            if key_pressed {
                pressed = true;
                analog_value = 1.0;
            }
        }
        
        // Check mouse input
        if let Some(mouse_button) = binding.mouse {
            if mouse.pressed(mouse_button) {
                pressed = true;
                analog_value = 1.0;
            }
        }
        
        // Check gamepad input using the Gamepad component directly
        if let (Some(gamepad_binding), Some(gamepad_entity)) = (&binding.gamepad, input_manager.active_gamepad) {
            if let Ok((_entity, gamepad)) = gamepad_query.get(gamepad_entity) {
                // Check button
                if let Some(button_type) = gamepad_binding.button {
                    // In Bevy 0.16.1, use gamepad.pressed() directly
                    if gamepad.pressed(button_type) {
                        pressed = true;
                        analog_value = 1.0;
                    }
                }
                
                // Check axis
                if let Some((axis_type, threshold)) = gamepad_binding.axis {
                    // In Bevy 0.16.1, use gamepad.get() directly
                    if let Some(axis_value) = gamepad.get(axis_type) {
                        let abs_value = axis_value.abs();
                        if abs_value >= threshold {
                            let normalized_value = if gamepad_binding.axis_negative {
                                if axis_value < -threshold { -axis_value } else { 0.0 }
                            } else {
                                if axis_value > threshold { axis_value } else { 0.0 }
                            };
                            
                            analog_value = normalized_value.clamp(0.0, 1.0);
                            if analog_value > 0.0 {
                                pressed = true;
                            }
                        }
                    }
                }
            }
        }
        
        // Store analog value
        input_manager.analog_values.insert(action, analog_value);
        
        // Determine input state based on current and previous frames
        let previous_state = input_manager.previous_states.get(&action).copied().unwrap_or(InputState::Released);
        let new_state = match (previous_state.is_pressed(), pressed) {
            (false, false) => InputState::Released,
            (false, true) => InputState::JustPressed,
            (true, true) => InputState::Pressed,
            (true, false) => InputState::JustReleased,
        };
        
        input_manager.current_states.insert(action, new_state);
    }
}

// ===== GAMEPAD CONNECTION SYSTEM =====
pub fn gamepad_connection_system(
    mut input_manager: ResMut<InputManager>,
    mut gamepad_events: EventReader<GamepadConnectionEvent>,
) {
    for connection_event in gamepad_events.read() {
        match &connection_event.connection {
            GamepadConnection::Connected { .. } => {
                if input_manager.active_gamepad.is_none() {
                    // connection_event.gamepad is an Entity in Bevy 0.16
                    input_manager.set_active_gamepad(connection_event.gamepad);
                    info!("Gamepad connected: {:?}", connection_event.gamepad);
                }
            }
            GamepadConnection::Disconnected => {
                if Some(connection_event.gamepad) == input_manager.active_gamepad {
                    input_manager.clear_active_gamepad();
                    info!("Active gamepad disconnected: {:?}", connection_event.gamepad);
                }
            }
        }
    }
}

// ===== CONVENIENCE SYSTEMS FOR COMMON PATTERNS =====

// System for handling pause input
pub fn handle_pause_input(
    input_manager: Res<InputManager>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input_manager.just_pressed(InputAction::Pause) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

// System for handling restart input
pub fn handle_restart_input(
    input_manager: Res<InputManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input_manager.just_pressed(InputAction::Restart) {
        next_state.set(GameState::Playing);
    }
}


// ===== PLUGIN SETUP =====
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputManager>()
            .add_systems(PreUpdate, (
                gamepad_connection_system,
                input_update_system,
            ))
            .add_systems(Update, (
                handle_pause_input,
                handle_restart_input,
            ));
    }
}

// ===== SERIALIZATION SUPPORT =====
impl InputManager {
    pub fn save_bindings(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Create a simplified representation for saving
        let mut simple_bindings = std::collections::HashMap::new();
        
        for (action, binding) in &self.bindings {
            let mut binding_info = std::collections::HashMap::new();
            
            if let Some(kb) = &binding.keyboard {
                binding_info.insert("keyboard".to_string(), format!("{:?}", kb.key));
            }
            if let Some(mouse) = &binding.mouse {
                binding_info.insert("mouse".to_string(), format!("{:?}", mouse));
            }
            if binding.gamepad.is_some() {
                binding_info.insert("gamepad".to_string(), "configured".to_string());
            }
            
            simple_bindings.insert(format!("{:?}", action), binding_info);
        }
        
        Ok(serde_json::to_string_pretty(&simple_bindings)?)
    }
    
    pub fn load_bindings(&mut self, _json: &str) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just reset to defaults when loading
        // Full implementation would need custom KeyCode/GamepadButton mapping
        self.setup_default_bindings();
        Ok(())
    }
}

// ===== AI HELPER TRAITS =====
pub trait AIInput {
    fn set_movement(&mut self, direction: Vec2);
    fn set_shooting(&mut self, shooting: bool);
    fn press_action(&mut self, action: InputAction);
    fn release_action(&mut self, action: InputAction);
}

impl AIInput for InputManager {
    fn set_movement(&mut self, direction: Vec2) {
        let clamped = direction.clamp_length_max(1.0);
        self.set_ai_analog(InputAction::MoveRight, clamped.x.max(0.0));
        self.set_ai_analog(InputAction::MoveLeft, (-clamped.x).max(0.0));
        self.set_ai_analog(InputAction::MoveUp, clamped.y.max(0.0));
        self.set_ai_analog(InputAction::MoveDown, (-clamped.y).max(0.0));
    }
    
    fn set_shooting(&mut self, shooting: bool) {
        self.set_ai_action(InputAction::Shoot, if shooting { InputState::Pressed } else { InputState::Released });
    }
    
    fn press_action(&mut self, action: InputAction) {
        self.set_ai_action(action, InputState::JustPressed);
    }
    
    fn release_action(&mut self, action: InputAction) {
        self.set_ai_action(action, InputState::JustReleased);
    }
}