use bevy::prelude::*;
use crate::components::{AlreadyDespawned, PendingDespawn};

// Safer despawning system - prevents double despawns and crashes
pub fn robust_despawn_system(
    mut commands: Commands,
    mut pending_query: Query<(Entity, &mut PendingDespawn)>,
    already_despawned_query: Query<Entity, With<PendingDespawn>>,
    time: Res<Time>,
) {
    // Clean up any entities that got marked as already despawned
    for entity in already_despawned_query.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.try_despawn();
        }
    }
    
    // Process pending despawns with delay for safety
    for (entity, mut pending) in pending_query.iter_mut() {
        pending.delay -= time.delta_secs();
        
        if pending.delay <= 0.0 {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.try_despawn();
            }
        }
    }
}

// Safe despawn helper - use instead of direct .despawn()
pub trait SafeDespawn {
    fn safe_despawn(&mut self) -> &mut Self;
    fn safe_despawn_delayed(&mut self, delay: f32) -> &mut Self;
}

impl SafeDespawn for EntityCommands<'_> {
    fn safe_despawn(&mut self) -> &mut Self {
        self.try_insert(PendingDespawn { delay: 0.016 }); // One frame delay
        self
    }
    
    fn safe_despawn_delayed(&mut self, delay: f32) -> &mut Self {
        self.try_insert(PendingDespawn { delay });
        self
    }
}
