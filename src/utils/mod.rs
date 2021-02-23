mod circular_vec;
pub mod math;
pub mod ui;
pub mod world;
pub mod world_decorations;

use amethyst::core::HiddenPropagate;
use amethyst::ecs::prelude::Entity;
use amethyst::ecs::prelude::World;
use amethyst::prelude::*;
use std::time::Duration;
pub use self::circular_vec::*;

// TODO: Use `Duration::saturating_sub` in future
pub fn sub_duration(a: Duration, b: Duration) -> Duration {
    if a > b {
        return a - b;
    } else {
        return Duration::from_millis(0);
    }
}

pub fn set_entity_visibility(world: &World, entity: Entity, is_visibility: bool) {
    let mut storage = world.write_storage::<HiddenPropagate>();

    if is_visibility {
        if storage.remove(entity).is_none() {
            // TODO: Do I need to warn?
            log::warn!(
                "There was no HiddenPropagate component to delete from {:?}",
                entity,
            );
        }
    } else if let Err(error) = storage.insert(entity, HiddenPropagate::new()) {
        log::error!(
            "Failed to insert HiddenPropagate component. Details: {}",
            error,
        );
    }
}

pub trait TakeContent<T> {
    fn take_content(&mut self) -> T;
}

impl<T> TakeContent<Vec<T>> for Vec<T> {
    fn take_content(&mut self) -> Vec<T> {
        if self.is_empty() {
            return Vec::new();
        } else {
            return std::mem::replace(self, Vec::with_capacity(self.capacity()));
        }
    }
}
