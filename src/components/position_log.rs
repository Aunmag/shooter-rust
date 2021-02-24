use std::collections::VecDeque;
use std::time::Duration;
use amethyst::core::math::Vector2;
use amethyst::ecs::Component;
use amethyst::ecs::DenseVecStorage;
use crate::utils::RingVec;
use std::cmp::Ordering;
use crate::systems::PositionLogSystem;
use crate::systems::net::TransformSyncSystem;
use crate::data::MAX_PING;
use crate::utils;

const KEEP_TIME: Duration = Duration::from_secs(2);

pub struct PositionLog {
    data: RingVec<Record>,
}

#[derive(Debug, PartialEq)]
struct Record {
    time: Duration,
    position: Vector2<f32>,
}

impl PositionLog {
    pub fn new() -> Self {
        let size = f64::max(
            KEEP_TIME.as_secs_f64() / PositionLogSystem::INTERVAL.as_secs_f64(),
            0.0,
        ).ceil() as usize;

        return Self {
            data: RingVec::new(size),
        };
    }

    pub fn store(&mut self, position: Vector2<f32>, time: Duration) {
        self.data.push(Record {
            time,
            position,
        });
    }

    pub fn find(
        &self,
        time_target: Duration,
        time_current: Duration,
        position_current: Vector2<f32>,
    ) -> Vector2<f32> {
        let current = Record {
            time: time_current,
            position: position_current,
        };

        let mut r1: Option<&Record> = None;
        let mut r2 = &current;

        for record in self.data.iter() {
            match record.time.cmp(&time_target) {
                Ordering::Less => {
                    if r1.as_ref().map_or(true, |r| r.time < record.time) {
                        r1.replace(record);
                    }
                }
                Ordering::Equal => {
                    return record.position;
                }
                Ordering::Greater => {
                    if r2.time > record.time {
                        r2 = record;
                    }
                }
            }
        }

        return r1.map_or(r2.position, |r1| {
            let t1 = utils::sub_duration(time_target, r1.time).as_secs_f64();
            let t2 = utils::sub_duration(r2.time, time_target).as_secs_f64();
            let ratio;

            #[allow(clippy::cast_possible_truncation)]
            {
                ratio = (t2 / (t1 + t2)) as f32;
            }

            return r2.position + (r1.position - r2.position) * ratio;
        });
    }
}

impl Component for PositionLog {
    type Storage = DenseVecStorage<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find() {
        let mut log = PositionLog::new();
        log.store(Vector2::new(2.0, 2.2), Duration::from_secs(1));
        log.store(Vector2::new(8.0, 8.2), Duration::from_secs(4));
        log.store(Vector2::new(32.0, 32.2), Duration::from_secs(16));
        log.store(Vector2::new(38.0, 38.2), Duration::from_secs(19));

        let current_time = Duration::from_secs(21);
        let current_position = Vector2::new(40.0, 40.2);

        assert_eq!(
            log.find(Duration::from_secs(1), current_time, current_position),
            Vector2::new(2.0, 2.2),
            "Precise. The first logged position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(4), current_time, current_position),
            Vector2::new(8.0, 8.2),
            "Precise. The second logged position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(16), current_time, current_position),
            Vector2::new(32.0, 32.2),
            "Precise. The third logged position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(19), current_time, current_position),
            Vector2::new(38.0, 38.2),
            "Precise. The fourth and last logged position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(21), current_time, current_position),
            Vector2::new(40.0, 40.2),
            "Precise. The current position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(5), current_time, current_position),
            Vector2::new(10.0, 10.200001),
            "Average. Near the second logged position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(15), current_time, current_position),
            Vector2::new(30.0, 30.2),
            "Average. Near the third logged position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(20), current_time, current_position),
            Vector2::new(39.0, 39.2),
            "Average. Between the last logged and current position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(0), current_time, current_position),
            Vector2::new(2.0, 2.2),
            "Out of range. Before first log. Return the earliest (first logged) position.",
        );

        assert_eq!(
            log.find(Duration::from_secs(22), current_time, current_position),
            Vector2::new(40.0, 40.2),
            "Out of range. After last log and current time. Return the latest (current) position.",
        );
    }
}
