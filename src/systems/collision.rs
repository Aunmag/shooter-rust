use crate::components::Collision;
use amethyst::core::math::Point2;
use amethyst::core::transform::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::WriteStorage;
use amethyst::ecs::Entities;
use amethyst::ecs::prelude::Write;
use crate::resources::DebugTimer;

#[derive(SystemDesc)]
pub struct CollisionSystem {
    is_optimal: bool,
    previous_collisions_count: usize,
}

struct Solution {
    entity_id: u32,
    push_x: f32,
    push_y: f32,
}

impl CollisionSystem {
    pub fn new() -> Self {
        return Self {
            is_optimal: true,
            previous_collisions_count: 0,
        };
    }
}

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Collision>,
        WriteStorage<'a, Transform>,
        Write<'a, DebugTimer>,
    );

    fn run(&mut self, (e, c, mut t, mut dt): Self::SystemData) {
        let s = dt.start();
        let mut last_checked_entity_id = 0;
        let mut solutions = Vec::with_capacity(self.previous_collisions_count * 2);

        self.previous_collisions_count = 0;

        for (e1, t1, c1) in (&e, &t, &c).join() {
            let p1 = to_point(t1);

            for (e2, t2, c2) in (&e, &t, &c).join() {
                if e1.id() == e2.id() || e2.id() <= last_checked_entity_id {
                    continue;
                }

                let p2 = to_point(t2);

                if let Some(solution) = Collision::resolve(c1, c2, p1, p2) {
                    append_solution(&mut solutions, e1.id(), solution.x, solution.y);
                    append_solution(&mut solutions, e2.id(), -solution.x, -solution.y);
                    self.previous_collisions_count += 1;
                }
            }

            if self.is_optimal {
                if e1.id() < last_checked_entity_id {
                    last_checked_entity_id = 0;
                    self.is_optimal = false;
                    log::warn!("The system may not work optimally since entities aren't sorted");
                } else {
                    last_checked_entity_id = e1.id();
                }
            }
        }

        if !solutions.is_empty() {
            for (entity, transform) in (&e, &mut t).join() {
                let mut to_remove = None;

                for (i, solution) in solutions.iter().enumerate() {
                    if entity.id() == solution.entity_id {
                        let translation = transform.translation_mut();
                        translation.x += solution.push_x;
                        translation.y += solution.push_y;
                        to_remove.replace(i);
                        break;
                    }
                }

                if let Some(to_remove) = to_remove {
                    solutions.swap_remove(to_remove);
                }

                if solutions.is_empty() {
                    break;
                }
            }
        }

        dt.end("Collision", s);
    }
}

fn to_point(transform: &Transform) -> Point2<f32> {
    return Point2::from([transform.translation().x, transform.translation().y]);
}

fn append_solution(solutions: &mut Vec<Solution>, entity_id: u32, push_x: f32, push_y: f32) {
    for solution in solutions.iter_mut() {
        if solution.entity_id == entity_id {
            solution.push_x += push_x;
            solution.push_y += push_y;
            return; // Return if solution has found and modified
        }
    }

    // Push a new one otherwise
    solutions.push(Solution {
        entity_id,
        push_x,
        push_y,
    });
}
