// cosmic_ui/src/bindings.rs
//! Data binding system for automatic UI updates

use bevy::prelude::*;
use std::hash::{Hash, Hasher, DefaultHasher};
use ordered_float::OrderedFloat;

/// Trait for bindable data sources
pub trait Bindable: Send + Sync + 'static {
    type Output: Clone + PartialEq + Hash;
    
    fn extract(&self, world: &World) -> Option<Self::Output>;
    
    fn hash_value(value: &Self::Output) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}

/// Generic binding for any component query
pub struct ComponentBinding<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ComponentBinding<T> {
    pub fn new() -> Self {
        Self { _phantom: std::marker::PhantomData }
    }
}
