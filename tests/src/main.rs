#![allow(internal_features)]
#![allow(clippy::vec_box)]
#![feature(core_intrinsics)]

use std::{
    intrinsics::black_box,
    io::Error,
    path::Path,
    sync::atomic::{AtomicU32, Ordering},
    time::Instant,
};

use anyhow::Result;
use fake::Fake;
use refs::{
    Own, Weak,
    hreads::set_current_thread_as_main,
    manage::{DataManager, ExistsManaged, ResourceLoader},
    managed,
};

fn _generate() -> Vec<u32> {
    (0..50_000).map(|_| (0..5).fake()).collect()
}

fn generate_box() -> Vec<Box<u32>> {
    (0..50_000).map(|_| Box::new((0..5).fake())).collect()
}

fn generate_own() -> Vec<Own<u32>> {
    (0..50_000).map(|_| Own::new((0..5).fake())).collect()
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

fn main() -> Result<()> {
    let start = Instant::now();
    let data_own = generate_own();
    dbg!(start.elapsed());
    let data_box = generate_box();
    dbg!(start.elapsed());

    let data_weak: Vec<_> = data_own.iter().map(Own::weak).collect();
    dbg!(start.elapsed());

    for _ in 0..4 {
        let start_own = Instant::now();
        let sum = calculate_own_sum(black_box(&data_own));
        dbg!(sum);
        dbg!(start_own.elapsed());
    }

    for _ in 0..4 {
        let start_box = Instant::now();
        let sum = calculate_box_sum(black_box(&data_box));
        dbg!(sum);
        dbg!(start_box.elapsed());
    }

    for _ in 0..4 {
        let start_weak = Instant::now();
        let sum = calculate_weak_sum(black_box(&data_weak));
        dbg!(sum);
        dbg!(start_weak.elapsed());
    }

    dbg!(data_own.len());
    dbg!(data_box.len());
    dbg!(data_weak.len());

    set_current_thread_as_main();

    Data::set_root_path("a");

    let data = Data::get("a");
    assert_eq!(data.a, 0);
    assert_eq!(data.name, "some_data");

    data.free();

    let data = Data::get("a");

    assert_eq!(data.a, 1);

    Data::store_with_name::<Error>("b", || {
        Ok(Data {
            a:    COUNTER.fetch_add(1, Ordering::Relaxed),
            name: String::new(),
        })
    })?;

    assert_eq!(Data::get("b").a, 2);

    Data::store_with_name::<Error>("b", || {
        Ok(Data {
            a:    COUNTER.fetch_add(1, Ordering::Relaxed),
            name: String::new(),
        })
    })?;

    assert_eq!(Data::get("b").a, 2);
    assert_eq!(unsafe { Data::get("b").get_static().a }, 2);

    assert!(data.exists_managed());
    assert!(!Weak::<Data>::default().exists_managed());

    Ok(())
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

struct Data {
    a:    u32,
    name: String,
}

impl ResourceLoader for Data {
    fn load_path(_path: &Path) -> Self {
        Data {
            a:    COUNTER.fetch_add(1, Ordering::Relaxed),
            name: "some_data".to_string(),
        }
    }

    fn load_data(_data: &[u8], _name: impl ToString) -> Self {
        unimplemented!()
    }
}

managed!(Data);

#[cfg(test)]
mod test {
    use refs::Own;

    #[test]
    #[ignore]
    fn test_pointers_info() {
        let val = Own::new(5);

        let weak = val.weak();

        drop(val);

        dbg!(&weak);
    }
}
