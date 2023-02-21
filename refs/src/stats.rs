use log::trace;
use std::collections::BTreeMap;
use std::sync::Mutex;

static STATS: Mutex<BTreeMap<String, Stat>> = Mutex::new(BTreeMap::new());
static STATS_ENABLED: Mutex<bool> = Mutex::new(false);

#[derive(Clone, Default)]
pub struct Stat {
    pub type_name: String,
    pub count: i64,
    pub size: usize,
    pub total_size: usize,
}

impl Stat {
    fn new(type_name: impl ToString, size: usize) -> Self {
        Self {
            type_name: type_name.to_string(),
            count: 0,
            size,
            total_size: 0,
        }
    }
}

pub fn enable_ref_stats_counter(enable: bool) {
    *STATS_ENABLED.lock().unwrap() = enable;
}

pub(crate) fn stats_enabled() -> bool {
    *STATS_ENABLED.lock().unwrap()
}

pub(crate) fn adjust_stat<T: ?Sized>(name: &str, change: i64, size: usize) {
    if !stats_enabled() {
        return;
    }

    let mut stats = STATS.lock().unwrap();

    let stat = match stats.get_mut(name) {
        Some(stat) => stat,
        None => {
            stats.insert(name.to_string(), Stat::new(name.to_string(), size));
            stats.get_mut(name).unwrap()
        }
    };

    trace!(
        "Stat change for {name}: size: {size}, change: {change}, count: {}, total: {}",
        stat.count,
        stat.total_size
    );

    stat.count += change;

    match change {
        1 => stat.total_size += size,
        -1 => stat.total_size -= size,
        _ => panic!("BUG: Invalid change: {change}"),
    };

    debug_assert!(stat.count >= 0);
}

pub fn dump_ref_stats() {
    let stats = STATS.lock().unwrap();

    if stats.is_empty() {
        println!("No managed objects.");
    }

    println!("==================Memory stats==================");
    for (name, stat) in stats.iter() {
        let name = clear_name(name);
        let count = stat.count;
        let size = stat.size;
        let total_size = size * count as usize;
        println!("Type: {name}, count: {count}, size: {size}, total size: {total_size}");
    }
    println!("================================================");
}

fn clear_name(name: &str) -> String {
    if let Some(last) = name.rfind(":") {
        name[last + 1..].to_string()
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::stats::STATS;
    use crate::{enable_ref_stats_counter, Own, Stat, Strong, TotalSize};
    use serial_test::serial;

    pub(crate) fn get_stat<T>() -> Stat {
        let name = std::any::type_name::<T>().to_string();
        STATS
            .lock()
            .unwrap()
            .get(&name)
            .unwrap_or(&Stat::default())
            .clone()
    }

    trait Trait {}

    struct Test {
        size: usize,
    }

    impl Trait for Test {}

    impl TotalSize for Test {
        fn total_size(&self) -> usize {
            self.size
        }
    }

    #[test]
    #[serial]
    fn stats_count() {
        enable_ref_stats_counter(true);

        assert_eq!(get_stat::<i32>().count, 0);

        let _1 = Own::new(5);
        let _2 = Own::new(5);
        let _3 = Own::new(5);

        assert_eq!(get_stat::<i32>().count, 3);
        drop(_1);
        assert_eq!(get_stat::<i32>().count, 2);
        drop(_2);
        assert_eq!(get_stat::<i32>().count, 1);
        drop(_3);
        assert_eq!(get_stat::<i32>().count, 0);

        let _1 = Strong::new(5);
        let _11 = _1.clone();
        let _2 = Strong::new(5);
        let _3 = Strong::new(5);

        assert_eq!(get_stat::<i32>().count, 3);
        drop(_1);
        assert_eq!(get_stat::<i32>().count, 3);
        drop(_11);
        assert_eq!(get_stat::<i32>().count, 2);
        drop(_2);
        assert_eq!(get_stat::<i32>().count, 1);
        drop(_3);
        assert_eq!(get_stat::<i32>().count, 0);
    }

    #[test]
    #[serial]
    fn stats_total_size() {
        enable_ref_stats_counter(true);

        assert_eq!(get_stat::<Test>().total_size, 0);

        let _1 = Own::new(Test { size: 200 });
        assert_eq!(get_stat::<Test>().total_size, 200);
        let _2 = Own::new(Test { size: 300 });
        assert_eq!(get_stat::<Test>().total_size, 500);

        drop(_1);
        assert_eq!(get_stat::<Test>().total_size, 300);
        drop(_2);
        assert_eq!(get_stat::<Test>().total_size, 0);

        let _1 = Strong::new(Test { size: 200 });
        let _11 = _1.clone();
        assert_eq!(get_stat::<Test>().total_size, 200);
        let _2 = Strong::new(Test { size: 300 });
        assert_eq!(get_stat::<Test>().total_size, 500);

        drop(_1);
        assert_eq!(get_stat::<Test>().total_size, 500);
        drop(_11);
        assert_eq!(get_stat::<Test>().total_size, 300);
        drop(_2);
        assert_eq!(get_stat::<Test>().total_size, 0);
    }

    #[test]
    #[serial]
    fn stats_dyn() {
        enable_ref_stats_counter(true);
        let _rf: Own<dyn Trait> = Own::<Test>::new(Test { size: 222 });
    }
}
