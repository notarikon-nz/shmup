# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Cosmic Tidal Pool** (formerly "Tidal Pool Cosmos") is a biological shoot-'em-up game built with Rust and Bevy 0.16.1. Players control an evolving microorganism in an underwater ecosystem, battling cosmic infections while utilizing environmental mechanics like fluid dynamics, chemical zones, and tidal effects.

## Development Commands

### Build and Run
```bash
cargo run                    # Run in debug mode
cargo run --release         # Run optimized build
cargo build                 # Build without running
cargo check                 # Fast syntax/type checking
```

### Development Tools
```bash
cargo clippy                 # Linting
cargo fmt                   # Code formatting
cargo test                  # Run tests
```

### Performance Testing
```bash
cargo run --release --features bevy/trace  # With performance tracing
```

## Architecture Overview

### Core Game Loop Structure
The game uses Bevy's ECS (Entity-Component-System) architecture with a complex system scheduling approach:

1. **Startup Systems**: One-time initialization (camera, assets, player spawn, environments)
2. **Core Game Loop**: Running during `GameState::Playing` and `IsPaused::Running`
3. **State Transitions**: Systems that run on entering/exiting game states
4. **Background Systems**: Environmental simulation (fluid dynamics, tidal mechanics)

### Key System Categories

#### Core Gameplay Systems (`main.rs:132-158`)
- Player movement with fluid dynamics
- Enhanced shooting system with evolution-based weapons
- Wave progression and spawning
- Achievement tracking
- ATP (currency) collection with magnetic field

#### Environmental Simulation (`main.rs:294-321`)
- **Fluid Dynamics**: Water current field generation affecting all entities
- **Chemical Environment**: pH and oxygen zones with damage/buff effects  
- **Tidal Mechanics**: King tide events, current reversals, debris movement
- **Thermal Vents**: Heat effects and thermal particle systems

#### Combat and AI Systems (`main.rs:263-282`)
- Enemy AI with biological behaviors (chemotaxis, fluid response)
- Formation coordination and chemical signaling
- Procedural colony spawning
- Multiple enemy types with unique sprites and behaviors

### Evolution and Upgrade System
The game features a complex biological evolution system:
- **Primary Evolutions**: 7 weapon types (CytoplasmicSpray, PseudopodNetwork, etc.)
- **Cellular Upgrades**: Damage amplification, movement efficiency, metabolic rate
- **ATP Economy**: Currency system with collection, spending, and balance analysis
- **Upgrade Limits**: Prevents unlimited upgrades for balance

### Balance Analysis Framework (`balance_systems.rs`)
Comprehensive real-time balance analysis system that tracks:
- Weapon performance and cost efficiency
- ATP economy health and generation rates
- Player progression metrics and difficulty scaling
- Dynamic difficulty adjustment based on performance
- Export capabilities for external analysis (F10 key)

### Custom UI System (Cosmic UI)
The project includes a custom UI framework in the `cosmic_ui/` workspace:
- Biological-themed HUD elements
- Progress bars and status indicators
- Theme system (cyberpunk, organic themes in `assets/ui_themes/`)

### Key Directories and Modules

#### Core Game Logic (`src/`)
- `main.rs`: Application setup and system scheduling
- `lib.rs`: Module declarations and re-exports
- `components.rs`: ECS components (Player, Projectile, FluidDynamics, etc.)
- `systems.rs`: Core gameplay systems with performance optimizations
- `enemy_types.rs` & `enemy_systems.rs`: Enemy AI and behaviors
- `weapon_systems.rs`: Evolution-based weapon mechanics
- `balance_systems.rs`: Real-time balance analysis and adjustment

#### Specialized Systems
- `tidal_mechanics.rs`: Environmental tide system
- `biological_systems.rs`: Organic AI behaviors (chemotaxis, symbiosis)
- `achievements.rs`: Steam-ready achievement system
- `wave_systems.rs`: Enemy wave progression
- `card_system.rs` & `stage_summary.rs`: Progression and upgrade cards

#### Assets Structure
- `assets/textures/enemies/`: Unique sprites for each enemy type
- `assets/audio/`: Biological sound effects (cell_burst.ogg, organic_pulse.ogg)
- `assets/ui_themes/`: TOML-based UI theming system
- `balance_data.json`: Persistent balance analysis data

### Performance Considerations
- Entity despawn system (`despawn.rs`) with `SafeDespawn` trait
- Particle system limits (MAX_PARTICLES: 200)
- Collision optimization with spatial grid
- Performance monitoring for balance system impact
- Conditional system execution based on game state

### State Management
- `GameState`: Playing, GameOver, StageSummary states
- `IsPaused`: Running/Paused substates
- Complex state transitions with cleanup systems
- Persistent data (achievements, high scores, balance data)

## Development Notes

### Common Patterns
- All systems use `.run_if(in_state(...))` conditions for performance
- Events are used for cross-system communication
- Resources hold global game state
- Components are kept lightweight with data-only structs

### Testing and Balance
- F2-F4 keys provide debug spawning (ATP, evolution chambers, king tides)
- F10 exports balance data for analysis
- Real-time balance monitoring affects gameplay dynamically
- Achievement system tracks detailed player metrics

### Working with Assets
- Enemy textures follow naming pattern: `textures/enemies/{enemy_type}.png`
- Audio files use biological terminology (organic_pulse, cell_burst, evolution)
- UI layouts stored as JSON in `assets/ui_layouts/`