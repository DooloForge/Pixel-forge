# Pixel Forge - Component-Based Game Architecture

A 2D underwater survival game built with Rust and Turbo, featuring a clean, modular component-based architecture.

## Architecture Overview

The game has been refactored from a monolithic structure into a clean, component-based architecture that separates concerns and makes the codebase more maintainable and extensible.

### Core Components

#### 1. Systems (`src/components/systems/`)
- **PhysicsSystem**: Handles all physics calculations, forces, and collisions
- **SpawnSystem**: Manages spawning of entities, items, and resources
- **WorldSystem**: Handles world generation, chunk management, and terrain
- **AISystem**: Manages AI behavior for fish, monsters, and other entities

#### 2. Entities (`src/components/entities/`)
- **GameEntity**: Base trait for all game entities
- **EntityManager**: Manages entity lifecycle and spatial organization
- **EntityFactory**: Creates different types of game entities
- **Component System**: Health, Inventory, Stats components

#### 3. Renderer (`src/components/renderer/`)
- **RenderSystem**: Main rendering system with layer-based rendering
- **CameraSystem**: Smooth camera movement and positioning
- **UIRenderer**: Handles all UI rendering separately from game rendering

#### 4. Input (`src/components/input/`)
- **InputSystem**: Centralized input processing and state management
- **InputMapping**: Configurable key bindings and input configuration

#### 5. Managers (`src/components/managers/`)
- **GameManager**: Coordinates all systems and manages game state
- **SceneManager**: Handles different game scenes and transitions
- **ResourceManager**: Manages game resources (textures, sounds, data)

### Key Benefits of the New Architecture

1. **Separation of Concerns**: Each system handles one specific aspect of the game
2. **Modularity**: Systems can be easily added, removed, or modified independently
3. **Testability**: Individual components can be tested in isolation
4. **Maintainability**: Code is organized logically and easier to understand
5. **Extensibility**: New features can be added without affecting existing systems
6. **Performance**: Better organization allows for more efficient updates and rendering

### System Flow

```
Input → GameManager → Systems → Entities → Renderer → Screen
  ↑                                                      ↓
  └─────────────── UI Events ←───────────────────────────┘
```

1. **Input System** processes user input and updates input state
2. **Game Manager** coordinates all systems based on current scene
3. **Systems** update game logic (physics, AI, spawning, world)
4. **Entity Manager** updates all entities and manages lifecycle
5. **Renderer** renders the game world and UI
6. **UI Events** can trigger input changes

### Entity Component System

Entities are composed of multiple components:
- **HealthComponent**: Manages health, damage, and invulnerability
- **InventoryComponent**: Handles item storage and management
- **StatsComponent**: Manages speed, strength, defense, and stamina
- **RenderData**: Contains visual information for rendering

### Rendering Layers

The renderer uses a layer-based system for proper depth ordering:
1. Background (sky, ocean)
2. Terrain (blocks, chunks)
3. Underwater (water effects)
4. Entity (fish, items, particles)
5. Player (player character)
6. UI (HUD, menus)
7. Foreground (effects, overlays)

### Scene Management

The game supports multiple scenes:
- **MainMenu**: Title screen and options
- **Playing**: Main gameplay
- **Inventory**: Item management
- **Crafting**: Crafting interface
- **Paused**: Pause menu

### Performance Features

- **Spatial Hashing**: Efficient entity queries by location
- **Chunked World**: Only loads and renders nearby world chunks
- **Render Queue**: Sorts render commands by layer for efficient rendering
- **Entity Pooling**: Reuses entity objects to reduce allocation

### Adding New Features

To add new features to the game:

1. **New System**: Create a new system in `src/components/systems/`
2. **New Entity**: Add entity type to `EntityType` enum and implement `GameEntity`
3. **New Component**: Create component struct and add to entities
4. **New Renderer**: Extend `RenderSystem` with new rendering logic
5. **New Input**: Add input handling to `InputSystem`

### Example: Adding a New Enemy Type

```rust
// 1. Add to EntityType enum
pub enum EntityType {
    // ... existing types
    NewEnemy,
}

// 2. Create enemy entity
pub struct NewEnemyEntity {
    // ... implementation
}

// 3. Add to EntityFactory
impl EntityFactory {
    pub fn create_new_enemy(&mut self, position: V2) -> Box<dyn GameEntity> {
        Box::new(NewEnemyEntity::new(self.next_entity_id(), position))
    }
}

// 4. Add AI behavior in AISystem
// 5. Add rendering in RenderSystem
```

## Building and Running

```bash
# Build the project
cargo build

# Run the game
cargo run
```

## Dependencies

- **Turbo**: Game engine and rendering
- **Rust**: Programming language

## Future Improvements

- **Audio System**: Sound effects and music
- **Save System**: Game state persistence
- **Multiplayer**: Network play support
- **Modding**: Plugin system for custom content
- **Mobile**: Touch controls and mobile optimization

## Contributing

When contributing to this project:

1. Follow the existing component architecture
2. Keep systems focused on single responsibilities
3. Use the entity component system for new game objects
4. Add proper error handling and logging
5. Write tests for new systems
6. Update documentation for new features

## License

This project is licensed under the MIT License.
