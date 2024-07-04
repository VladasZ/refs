#![allow(internal_features)]
#![feature(core_intrinsics)]

use std::{intrinsics::black_box, time::Instant};

use fake::Fake;
use refs::Own;

fn _generate() -> Vec<u32> {
    (0..10_000_000).map(|_| (0..5).fake()).collect()
}

fn _generate_box() -> Vec<Box<u32>> {
    (0..10_000_000).map(|_| Box::new((0..5).fake())).collect()
}

fn generate_own() -> Vec<Own<u32>> {
    (0..10_000_000).map(|_| Own::new((0..5).fake())).collect()
}

fn calculate_sum(data: &Vec<Own<u32>>) -> u32 {
    let mut sum = 0;

    for val in data {
        sum += **val;
    }

    sum
}

fn main() {
    let data = black_box(generate_own());

    // let data_weak: Vec<_> = data.iter().map(|o| o.weak()).collect();

    let start = Instant::now();

    let sum = calculate_sum(&data);

    dbg!(sum);
    dbg!(start.elapsed());
    dbg!(data.len());
}
