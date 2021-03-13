use crate::components::Actor;
use crate::components::ActorActions;
use crate::components::Ai;
use crate::utils::math;
use amethyst::core::timing::Time;
use amethyst::core::transform::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::Read;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::WriteStorage;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use std::f32::consts::FRAC_PI_2;
use std::f64::consts::FRAC_PI_8;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use amethyst::ecs::prelude::Write;
use crate::resources::DebugTimer;

const CHANGE_MOVEMENT_PROBABILITY: f64 = 1.0;
const TURN_PROBABILITY: f64 = 2.0;
const MAX_DISTANCE_FROM_CENTER: f32 = 7.5;

#[derive(SystemDesc)]
pub struct AiSystem {
    randomizer: Pcg32,
}

impl AiSystem {
    pub fn new() -> Self {
        let randomizer_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or_else(|e| e.duration().as_secs(), |t| t.as_secs());

        return Self {
            randomizer: Pcg32::seed_from_u64(randomizer_seed),
        };
    }

    fn gen_chance(&mut self, probability: f64) -> bool {
        if probability >= 1.0 {
            return true;
        } else if probability <= 0.0 {
            return false;
        } else {
            return self.randomizer.gen_bool(probability);
        }
    }
}

impl<'a> System<'a> for AiSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadStorage<'a, Ai>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Actor>,
        Write<'a, DebugTimer>,
    );

    fn run(&mut self, (time, ais, transforms, mut actors, mut dt): Self::SystemData) {
        let s = dt.start();
        let delta = time.delta_seconds() as f64;
        let change_movement_probability = CHANGE_MOVEMENT_PROBABILITY * delta;
        let turn_probability = TURN_PROBABILITY * delta;

        for (_, actor, transform) in (&ais, &mut actors, &transforms).join() {
            if self.gen_chance(change_movement_probability) {
                actor.actions.toggle(ActorActions::MOVEMENT_FORWARD);
            }

            let is_moving = actor.actions.contains(ActorActions::MOVEMENT_FORWARD);

            if !is_moving && self.gen_chance(turn_probability) {
                let x = transform.translation().x;
                let y = transform.translation().y;

                #[allow(clippy::cast_possible_truncation)]
                if math::are_closer_than(x, y, 0.0, 0.0, MAX_DISTANCE_FROM_CENTER) {
                    actor.rotation = self.randomizer.gen_range(-FRAC_PI_8..FRAC_PI_8) as f32;
                } else {
                    actor.rotation = math::angle_difference(
                        math::angle(x, y, 0.0, 0.0) + FRAC_PI_2,
                        transform.euler_angles().2,
                    );
                }
            } else {
                actor.rotation = 0.0;
            }
        }
        dt.end("Ai", s);
    }
}
