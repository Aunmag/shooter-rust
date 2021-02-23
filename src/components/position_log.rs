use std::collections::VecDeque;
use std::time::Duration;
use amethyst::core::math::Vector2;
use amethyst::ecs::Component;
use amethyst::ecs::DenseVecStorage;

pub struct PositionLog {
    data: VecDeque<Record>,
}

#[derive(Debug, PartialEq)]
struct Record {
    time: Duration,
    position: Vector2<f32>,
}

impl PositionLog {
    pub fn new() -> Self {
        return Self {
            data: VecDeque::new(),
        };
    }

    pub fn store(&mut self, position: Vector2<f32>, time: Duration) {
        self.data.push_front(Record {
            time,
            position,
        });
    }

    pub fn find(&self, time: Duration) -> Option<Vector2<f32>> {
        let mut r1 = None;
        let mut r2 = None;

        for record in self.data.iter() {
            if record.time >= time {
                r1.replace(record);
            } else {
                r2.replace(record);
                break;
            }
        }

        if let (Some(r1), Some(r2)) = (r1, r2) {
            let t1 = r1.time.as_secs_f64() - time.as_secs_f64();
            let t2 = time.as_secs_f64() - r2.time.as_secs_f64();
            let ratio;

            #[allow(clippy::cast_possible_truncation)]
            {
                ratio = (t2 / (t1 + t2)) as f32;
            }

            return Some(r2.position + (r1.position - r2.position) * ratio);
        } else {
            return r1.or(r2).map(|l| l.position);
        }
    }

    pub fn cleanup(&mut self, before: Duration) {
        loop {
            if self.data.back().map_or(false, |r| r.time <= before) {
                self.data.pop_back();
            } else {
                break;
            }
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
        assert_eq!(log.find(Duration::from_secs(5)), Some(Vector2::new(10.0, 10.2)));
        assert_eq!(log.find(Duration::from_secs(15)), Some(Vector2::new(30.0, 30.2)));

        // Empty
        assert_eq!(PositionLog::new().find(Duration::from_secs(0)), None);
    }

    #[test]
    fn test_cleanup() {
        let mut log = PositionLog::new();
        log.store(Vector2::new(1.0, 1.1), Duration::from_secs(1));
        log.store(Vector2::new(2.0, 2.1), Duration::from_secs(2));
        log.store(Vector2::new(3.0, 3.1), Duration::from_secs(3));
        log.store(Vector2::new(4.0, 4.1), Duration::from_secs(4));
        log.cleanup(Duration::from_secs(1));
        let mut expected = PositionLog::new();
        expected.store(Vector2::new(2.0, 2.1), Duration::from_secs(2));
        expected.store(Vector2::new(3.0, 3.1), Duration::from_secs(3));
        expected.store(Vector2::new(4.0, 4.1), Duration::from_secs(4));
        assert_eq!(log.data, expected.data);

        let mut log = PositionLog::new();
        log.store(Vector2::new(1.0, 1.1), Duration::from_secs(1));
        log.store(Vector2::new(2.0, 2.1), Duration::from_secs(2));
        log.store(Vector2::new(3.0, 3.1), Duration::from_secs(3));
        log.store(Vector2::new(4.0, 4.1), Duration::from_secs(4));
        log.cleanup(Duration::from_secs(3));
        let mut expected = PositionLog::new();
        expected.store(Vector2::new(4.0, 4.1), Duration::from_secs(4));
        assert_eq!(log.data, expected.data);

        let mut log = PositionLog::new();
        log.store(Vector2::new(1.0, 1.1), Duration::from_secs(1));
        log.store(Vector2::new(2.0, 2.1), Duration::from_secs(2));
        log.store(Vector2::new(3.0, 3.1), Duration::from_secs(3));
        log.store(Vector2::new(4.0, 4.1), Duration::from_secs(4));
        log.cleanup(Duration::from_secs(10));
        assert_eq!(log.data, VecDeque::new());
    }
}
