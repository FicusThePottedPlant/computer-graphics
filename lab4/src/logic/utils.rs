use std::fs::File;
use crate::logic::algorithms::{DrawType, Measurable};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

pub fn run_profile(obj: &mut impl Measurable, draw_type: DrawType, file: &mut File) {
        for i in (1000..=10000).step_by(1000) {
            obj.measure_time(draw_type, i as f32, file)
        }
}

pub fn prof(file: &mut File) -> Vec<(i32, f64)>{
    file.seek(SeekFrom::Start(0)).unwrap();
    let reader = BufReader::new(file);
    let mut sums: HashMap<i32, (i64, usize)> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let mut parts = line.split_whitespace();
        let key: i32 = parts.next().unwrap().parse().unwrap();
        let value: i64 = parts.next().unwrap().parse().unwrap();

        let (sum, count) = sums.entry(key).or_insert((0, 0));
        *sum += value;
        *count += 1;
    }

    let mut averages: Vec<(i32, f64)> = Vec::new();
    for (key, (sum, count)) in sums {
        averages.push((key, sum as f64 / count as f64));
    }
    averages.sort_by(|a, b| a.0.cmp(&b.0));
    averages
}