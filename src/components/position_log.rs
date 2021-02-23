use std::collections::VecDeque;
use std::time::Duration;
use amethyst::core::math::Vector2;
use amethyst::ecs::Component;
use amethyst::ecs::DenseVecStorage;
use crate::utils::CircularVec;
use std::cmp::Ordering;

pub struct PositionLog {
    data: CircularVec<Record>,
}

#[derive(Debug, PartialEq)]
struct Record {
    time: Duration,
    position: Vector2<f32>,
}

struct RecordTemp<'a> {
    interval: Duration,
    position: &'a Vector2<f32>,
}

impl PositionLog {
    pub fn new() -> Self {
        return Self {
            data: CircularVec::new(60), // TODO: Calc size
        };
    }

    pub fn store(&mut self, position: Vector2<f32>, time: Duration) {
        self.data.push(Record {
            time,
            position,
        });
    }

    pub fn find(&self, time: Duration) -> Option<Vector2<f32>> {
        let mut r1: Option<RecordTemp> = None;
        let mut r2: Option<RecordTemp> = None;

        for record in self.data.iter() {
            match record.time.cmp(&time) {
                Ordering::Less => {
                    let interval = time - record.time;

                    if r1.as_ref().map_or(true, |r| r.interval > interval) {
                        r1.replace(RecordTemp {
                            interval,
                            position: &record.position,
                        });
                    }
                }
                Ordering::Equal => {
                    return Some(record.position);
                }
                Ordering::Greater => {
                    let interval = record.time - time;

                    if r2.as_ref().map_or(true, |r| r.interval > interval) {
                        r2.replace(RecordTemp {
                            interval,
                            position: &record.position,
                        });
                    }
                }
            }
        }

        if let (Some(r1), Some(r2)) = (&r1, &r2) {
            let t1 = r1.interval.as_secs_f64();
            let t2 = r2.interval.as_secs_f64();
            let ratio;

            #[allow(clippy::cast_possible_truncation)]
            {
                ratio = (t2 / (t1 + t2)) as f32;
            }

            return Some(r2.position + (r1.position - r2.position) * ratio);
        } else {
            return r1.or(r2).map(|r| *r.position);
        }
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

        // Precise
        assert_eq!(log.find(Duration::from_secs(1)), Some(Vector2::new(2.0, 2.2)));
        assert_eq!(log.find(Duration::from_secs(4)), Some(Vector2::new(8.0, 8.2)));
        assert_eq!(log.find(Duration::from_secs(16)), Some(Vector2::new(32.0, 32.2)));
        assert_eq!(log.find(Duration::from_secs(19)), Some(Vector2::new(38.0, 38.2)));

        // Out range
        assert_eq!(log.find(Duration::from_secs(0)), Some(Vector2::new(2.0, 2.2)));
        assert_eq!(log.find(Duration::from_secs(20)), Some(Vector2::new(38.0, 38.2)));

        // Average
        assert_eq!(log.find(Duration::from_secs(5)), Some(Vector2::new(10.0, 10.200001)));
        assert_eq!(log.find(Duration::from_secs(15)), Some(Vector2::new(30.0, 30.2)));

        // Empty
        assert_eq!(PositionLog::new().find(Duration::from_secs(0)), None);
    }
}
