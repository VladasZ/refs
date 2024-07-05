#![allow(internal_features)]
#![allow(clippy::vec_box)]
#![feature(core_intrinsics)]

use std::{intrinsics::black_box, time::Instant};

use fake::Fake;
use refs::{Own, Weak};

fn _generate() -> Vec<u32> {
    (0..10_000_000).map(|_| (0..5).fake()).collect()
}

fn generate_box() -> Vec<Box<u32>> {
    (0..10_000_000).map(|_| Box::new((0..5).fake())).collect()
}

fn generate_own() -> Vec<Own<u32>> {
    (0..10_000_000).map(|_| Own::new((0..5).fake())).collect()
}

fn calculate_own_sum(data: &Vec<Own<u32>>) -> u32 {
    let mut sum = 0;
    for val in data {
        sum += **val;
    }
    sum
}

fn calculate_box_sum(data: &Vec<Box<u32>>) -> u32 {
    let mut sum = 0;
    for val in data {
        sum += **val;
    }
    sum
}

fn calculate_weak_sum(data: &Vec<Weak<u32>>) -> u32 {
    let mut sum = 0;
    for val in data {
        sum += **val;
    }
    sum
}

fn main() {
    let data_own = generate_own();
    let data_box = generate_box();

    let data_weak: Vec<_> = data_own.iter().map(|o| o.weak()).collect();

    for _ in 0..10 {
        let start_own = Instant::now();
        let sum = calculate_own_sum(black_box(&data_own));
        dbg!(sum);
        dbg!(start_own.elapsed());
    }

    for _ in 0..10 {
        let start_box = Instant::now();
        let sum = calculate_box_sum(black_box(&data_box));
        dbg!(sum);
        dbg!(start_box.elapsed());
    }

    for _ in 0..10 {
        let start_weak = Instant::now();
        let sum = calculate_weak_sum(black_box(&data_weak));
        dbg!(sum);
        dbg!(start_weak.elapsed());
    }

    dbg!(data_own.len());
    dbg!(data_box.len());
    dbg!(data_weak.len());
}
