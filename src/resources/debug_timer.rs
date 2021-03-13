use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

#[derive(Default)]
pub struct DebugTimer {
    data: HashMap<&'static str, (Duration, u128)>,
}

impl DebugTimer {
    pub fn new() -> Self {
        return Self {
            data: HashMap::with_capacity(50),
        };
    }

    pub fn start(&self) -> Instant {
        return Instant::now();
    }

    pub fn end(&mut self, name: &'static str, start: Instant) {
        self.add(name, start.elapsed());
    }

    fn add(&mut self, name: &'static str, time: Duration) {
        self
            .data
            .entry(name)
            .and_modify(|e| {
                e.0 += time;
                e.1 += 1;
            })
            .or_insert((time, 0));
    }

    pub fn print(&self) {
        let mut total = Duration::from_millis(0);
        let mut total_count = 0;
        let mut temp = Vec::with_capacity(self.data.len());

        for (name, (time, count)) in self.data.iter() {
            total += *time;
            total_count += count;
            temp.push((name, *time, *count));
        }

        temp.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("--------------------------------------------");
        println!("     % |    total |      average | system   ");
        // print_time("*", total, total_count, 100.0);

        for (name, time, count) in temp.iter() {
            let percentage = time.as_secs_f64() / total.as_secs_f64() * 100.0;
            print_time(name, *time, *count, percentage);
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

fn print_time(name: &'static str, time: Duration, mut count: u128, percentage: f64) {
    if count == 0 {
        count = 1;
    }

    println!(
        "{:>6.2} | {:>10} | {:>10.2} | {}",
        percentage,
        time.as_millis(),
        time.as_millis() as f64 / count as f64,
        name,
    );
}
