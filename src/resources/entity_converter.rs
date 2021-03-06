use amethyst::ecs::Entities;
use amethyst::ecs::Entity;

#[derive(Default)]
pub struct EntityConverter {
    data: Vec<Record>,
}

struct Record {
    entity: Entity,
    external_id: u32,
}

impl EntityConverter {
    pub const fn new() -> Self {
        return Self { data: Vec::new() };
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_internal(&mut self, entities: &Entities, external_id: u32) -> Entity {
        for record in &self.data {
            if record.external_id == external_id {
                return record.entity;
            }
        }

        let entity = entities.create();

        self.data.push(Record {
            entity,
            external_id,
        });

        return entity;
    }

    pub fn remove(&mut self, entity: Entity) {
        for (i, record) in self.data.iter().enumerate() {
            if record.entity == entity {
                self.data.swap_remove(i);
                break;
            }
        }
    }
}
