use crate::components::Actor;
use crate::components::ActorActions;
use crate::components::Weapon;
use crate::resources::GameTask;
use crate::resources::GameTaskResource;
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::Read;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::Write;
use amethyst::ecs::prelude::WriteStorage;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use std::f32::consts::PI;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const VELOCITY_DEVIATION_FACTOR: f32 = 0.1;
const DIRECTION_DEVIATION: f32 = 0.02;

#[derive(SystemDesc)]
pub struct WeaponSystem {
    randomizer: Pcg32,
}

impl WeaponSystem {
    pub fn new() -> Self {
        let randomizer_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or_else(|e| e.duration().as_secs(), |t| t.as_secs());

        return Self {
            randomizer: Pcg32::seed_from_u64(randomizer_seed),
        };
    }

    fn deviate_velocity(&mut self, velocity: f32) -> f32 {
        let min = 1.0 - VELOCITY_DEVIATION_FACTOR;
        let max = 1.0 + VELOCITY_DEVIATION_FACTOR;
        return velocity * self.randomizer.gen_range(min..max) as f32;
    }

    fn deviate_direction(&mut self, mut direction: f32) -> f32 {
        direction += self.randomizer.gen_range(-DIRECTION_DEVIATION..DIRECTION_DEVIATION) as f32;
        direction %= PI;
        return direction;
    }
}

impl<'a> System<'a> for WeaponSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadStorage<'a, Actor>,
        ReadStorage<'a, Transform>,
        Write<'a, GameTaskResource>,
        WriteStorage<'a, Weapon>,
    );

    fn run(&mut self, (time, actors, transforms, mut tasks, mut weapons): Self::SystemData) {
        for (actor, transform, weapon) in (&actors, &transforms, &mut weapons).join() {
            if actor.actions.contains(ActorActions::ATTACK) && weapon.fire(time.absolute_time()) {
                let velocity = self.deviate_velocity(weapon.config.muzzle_velocity);
                let (sin, cos) = (-self.deviate_direction(transform.euler_angles().2)).sin_cos();

                tasks.push(GameTask::ProjectileSpawn {
                    x: transform.translation().x,
                    y: transform.translation().y,
                    velocity_x: sin * velocity,
                    velocity_y: cos * velocity,
                    acceleration_factor: weapon.config.projectile.acceleration_factor,
                });
            }
        }
    }
}