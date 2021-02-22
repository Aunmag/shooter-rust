use crate::components::Interpolation;
use crate::components::PositionLog;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::Read;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::WriteStorage;
use amethyst::core::transform::Transform;
use amethyst::core::timing::Time;
use std::time::Duration;
use std::time::Instant;
use crate::data::MAX_PING;
use crate::utils;

#[allow(clippy::integer_division)]
const INTERVAL: Duration = Duration::from_millis(1000 / 25); // TODO: Maybe sync with TransformSync

#[derive(SystemDesc)]
pub struct PositionLogSystem {
    last_run: Instant,
}

impl PositionLogSystem {
    pub fn new() -> Self {
        return Self {
            last_run: Instant::now(),
        };
    }
}

impl<'a> System<'a> for PositionLogSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Interpolation>,
        WriteStorage<'a, PositionLog>,
    );

    fn run(&mut self, (time, transforms, interpolations, mut log): Self::SystemData) {
        if self.last_run.elapsed() < INTERVAL {
            return;
        }

        self.last_run = Instant::now();

        let now = time.absolute_time();
        let cleanup_before = utils::sub_duration(now, MAX_PING);

        for (transform, interpolation, log) in (&transforms, (&interpolations).maybe(), &mut log).join() {
            let mut position = transform.translation().xy();

            if let Some(interpolation) = interpolation {
                position.x += interpolation.offset_x;
                position.y += interpolation.offset_y;
            }

            log.push(position, now);
            log.cleanup(cleanup_before)
        }
    }
}
