use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use log::trace;

static STATS: Mutex<BTreeMap<String, Stat>> = Mutex::new(BTreeMap::new());
static STATS_ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Default)]
pub struct Stat {
    pub type_name: String,
    pub count:     i64,
}

impl Stat {
    fn new(type_name: impl ToString) -> Self {
        Self {
            type_name: type_name.to_string(),
            count:     0,
        }
    }
}

pub fn enable_ref_stats_counter(enable: bool) {
    STATS_ENABLED.store(enable, Ordering::Relaxed);
}

pub(crate) fn stats_enabled() -> bool {
    STATS_ENABLED.load(Ordering::Relaxed)
}

pub(crate) fn adjust_stat(name: &str, change: i64) {
    if !stats_enabled() {
        return;
    }

    let mut stats = STATS.lock().unwrap();

    let stat = if let Some(stat) = stats.get_mut(name) {
        stat
    } else {
        stats.insert(name.to_string(), Stat::new(name.to_string()));
        stats.get_mut(name).unwrap()
    };

    trace!("Stat change for {name}: change: {change}, count: {}", stat.count,);

    stat.count += change;

    debug_assert!(stat.count >= 0);
}

pub fn dump_ref_stats() {
    let stats = STATS.lock().unwrap();

    if stats.is_empty() {
        println!("No managed objects.");
    }

    let mut total_count = 0;

    println!("==================Memory stats==================");
    for (name, stat) in stats.iter() {
        let name = clear_name(name);
        let count = stat.count;
        total_count += count;
        println!("Type: {name}, count: {count}");
    }
    println!("Total count: {total_count}");
    println!("================================================");
}

fn clear_name(name: &str) -> String {
    if let Some(last) = name.rfind(':') {
        name[last + 1..].to_string()
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod test {
    use serial_test::serial;

    use crate::{
        enable_ref_stats_counter,
        stats::{clear_name, stats_enabled, STATS},
        Own, Stat,
    };

    pub(crate) fn get_stat<T>() -> Stat {
        let name = std::any::type_name::<T>().to_string();
        STATS.lock().unwrap().get(&name).unwrap_or(&Stat::default()).clone()
    }

    trait Trait {}

    struct Test {
        _data: u32,
    }

    impl Trait for Test {}

    #[test]
    #[serial]
    fn stats_count() {
        enable_ref_stats_counter(false);

        assert_eq!(stats_enabled(), false);

        enable_ref_stats_counter(true);

        assert_eq!(stats_enabled(), true);

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
    }

    #[test]
    #[serial]
    fn stats_dyn() {
        enable_ref_stats_counter(true);
        let _rf: Own<dyn Trait> = Own::<Test>::new(Test { _data: 0 });
    }

    #[test]
    fn stats_misc() {
        assert_eq!("clear_name", clear_name("crate::stats::clear_name"));
    }
}
