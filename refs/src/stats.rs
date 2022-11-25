use std::collections::BTreeMap;
use std::sync::Mutex;

static STATS: Mutex<BTreeMap<String, Stat>> = Mutex::new(BTreeMap::new());
static STATS_ENABLED: Mutex<bool> = Mutex::new(false);

#[derive(Clone, Default)]
pub(crate) struct Stat {
    pub(crate) count: i64,
    pub(crate) size: usize,
    pub(crate) total_size: usize,
}

impl Stat {
    fn new(size: usize) -> Self {
        Self {
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

pub(crate) fn adjust_stat<T: ?Sized>(change: i64, size: usize) {
    if !stats_enabled() {
        return;
    }

    let mut stats = STATS.lock().unwrap();

    let name = std::any::type_name::<T>().to_string();

    let stat = match stats.get_mut(&name) {
        Some(stat) => stat,
        None => {
            stats.insert(name.clone(), Stat::new(size));
            stats.get_mut(&name).unwrap()
        }
    };

    stat.count += change;

    match change {
        1 => stat.total_size += size,
        -1 => stat.total_size -= size,
        _ => panic!("Invalid change: {change}"),
    };

    debug_assert!(stat.count >= 0);
}

#[cfg(test)]
pub(crate) fn get_stat<T>() -> Stat {
    let name = std::any::type_name::<T>().to_string();
    STATS
        .lock()
        .unwrap()
        .get(&name)
        .unwrap_or(&Stat::default())
        .clone()
}

pub fn dump_ref_stats() {
    let stats = STATS.lock().unwrap();

    if stats.is_empty() {
        println!("No managed objects.");
    }

    for (name, stat) in stats.iter() {
        let count = stat.count;
        let size = stat.size;
        let total_size = size * count as usize;
        println!("Type: {name}, count: {count}, size: {size}, total size: {total_size}");
    }
}
