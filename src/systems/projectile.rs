use crate::components::Collision;
use crate::components::Projectile;
use crate::data::LAYER_PROJECTILE;
use crate::resources::GameTask;
use crate::resources::GameTaskResource;
use crate::utils::math;
use crate::utils::DurationExt;
use amethyst::core::math::Point3;
use amethyst::core::math::Vector2;
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::Read;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::Write;
use amethyst::ecs::Entities;
use amethyst::ecs::Entity;
use amethyst::renderer::debug_drawing::DebugLines;
use amethyst::renderer::palette::Srgba;
use crate::resources::DebugTimer;

const PUSH_FACTOR: f32 = 0.00025;
const VELOCITY_MIN: f32 = 5.0;

#[derive(SystemDesc)]
pub struct ProjectileSystem;

struct Obstacle {
    entity: Entity,
    distance_squared: f32,
}

impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        ReadStorage<'a, Projectile>,
        ReadStorage<'a, Collision>,
        ReadStorage<'a, Transform>,
        Write<'a, GameTaskResource>,
        Write<'a, DebugTimer>,
        Write<'a, DebugLines>,
    );

    fn run(
        &mut self,
        (
            entities,
            time,
            projectiles,
            collisions,
            transforms,
            mut tasks,
            mut dt,
            mut debug
        ): Self::SystemData,
    ) {
        let s = dt.start();
        let time_current = time.absolute_time();
        let time_previous = time_current.sub_safely(time.delta_time());

        for (entity, projectile) in (&entities, &projectiles).join() {
            let (mut head_position, head_velocity) = projectile.calc_data(time_current);
            let (tail_position, _) = projectile.calc_data(time_previous);
            let mut obstacle: Option<Obstacle> = None;

            for (entity, collision, transform) in (&entities, &collisions, &transforms).join() {
                if projectile.shooter == Some(entity) {
                    continue;
                }

                let obstacle_position = transform.translation().xy();

                if is_collision(
                    head_position,
                    tail_position,
                    obstacle_position,
                    collision.radius,
                ) {
                    let distance_squared = math::distance_squared(
                        tail_position.x,
                        tail_position.y,
                        obstacle_position.x,
                        obstacle_position.y,
                    );

                    if obstacle.as_ref().map_or(true, |o| o.distance_squared > distance_squared) {
                        obstacle = Some(Obstacle {
                            entity,
                            distance_squared,
                        });
                    }
                }
            }

            if let Some(obstacle) = obstacle.as_ref() {
                let (sin, cos) = math::angle(
                    head_position.x,
                    head_position.y,
                    tail_position.x,
                    tail_position.y,
                ).sin_cos();

                let length = obstacle.distance_squared.sqrt();
                head_position.x = tail_position.x + length * cos;
                head_position.y = tail_position.y + length * sin;

                // TODO: Better don't use head velocity, calc actual velocity at collision point
                tasks.push(GameTask::ProjectileHit {
                    entity: obstacle.entity,
                    force_x: head_velocity.x * PUSH_FACTOR,
                    force_y: head_velocity.y * PUSH_FACTOR,
                });
            }

            debug.draw_line(
                Point3::from([head_position.x, head_position.y, LAYER_PROJECTILE]),
                Point3::from([tail_position.x, tail_position.y, LAYER_PROJECTILE]),
                Srgba::new(1.0, 1.0, 0.0, 1.0),
            );

            if obstacle.is_some() || has_stopped(head_velocity) {
                if let Err(error) = entities.delete(entity) {
                    log::error!("Failed to delete a stopped bullet: {}", error);
                }
            }
        }

        dt.end("Projectile", s);
    }
}

fn is_collision(
    line_head: Vector2<f32>,
    line_tail: Vector2<f32>,
    obstacle: Vector2<f32>,
    obstacle_radius: f32,
) -> bool {
    let x1 = obstacle.x - line_head.x;
    let y1 = obstacle.y - line_head.y;
    let x2 = obstacle.x - line_tail.x - x1;
    let y2 = obstacle.y - line_tail.y - y1;

    let a = x2 * x2 + y2 * y2;
    let b = (x1 * x2 + y1 * y2) * 2.0;
    let c = x1 * x1 + y1 * y1 - obstacle_radius * obstacle_radius;

    if b > 0.0 {
        return c < 0.0;
    } else if -b < 2.0 * a {
        return a * 4.0 * c - b * b < 0.0;
    } else {
        return a + b + c < 0.0;
    }
}

fn has_stopped(velocity: Vector2<f32>) -> bool {
    return math::are_closer_than(velocity.x, velocity.y, 0.0, 0.0, VELOCITY_MIN);
}
