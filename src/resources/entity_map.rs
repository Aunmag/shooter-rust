use amethyst::ecs::Entity;
use std::collections::HashMap;

pub struct EntityMap {
    by_internal: HashMap<u32, u16>,
    by_external: HashMap<u16, Entity>,
    next_external_id: u16,
}

impl EntityMap {
    pub fn new() -> Self {
        return Self {
            by_internal: HashMap::new(),
            by_external: HashMap::new(),
            next_external_id: 0,
        };
    }

    pub fn store(&mut self, entity: Entity, external_id: u16) {
        let previous_external_id = self.by_internal.insert(entity.id(), external_id);
        let previous_entity = self.by_external.insert(external_id, entity);

        // if previous_external_id.is_some() || previous_entity.is_some() {
        //     log::warn!(
        //         "Entity({}) with external ID = {} was already stored",
        //         entity.id(),
        //         external_id,
        //     );
        // }
    }

    pub fn generate_external_id(&mut self) -> u16 {
        let id = self.next_external_id;

        if let Some(next_external_id) = self.next_external_id.checked_add(1) {
            self.next_external_id = next_external_id;
        } else {
            self.next_external_id = 0;
            log::warn!("External IDs have reached their limit thus were restarted");
        }

        return id;
    }

    pub fn get_entity(&self, external_id: u16) -> Option<Entity> {
        return self.by_external.get(&external_id).copied();
    }

    pub fn get_external_id(&self, entity: Entity) -> Option<u16> {
        return self.by_internal.get(&entity.id()).copied();
    }
}
