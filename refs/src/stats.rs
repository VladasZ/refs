use std::collections::BTreeMap;
use std::sync::Mutex;

static STATS: Mutex<BTreeMap<String, i64>> = Mutex::new(BTreeMap::new());
static STATS_ENABLED: Mutex<bool> = Mutex::new(false);

pub fn enable_ref_stats_counter(enable: bool) {
    *STATS_ENABLED.lock().unwrap() = enable;
}

pub(crate) fn stats_enabled() -> bool {
    *STATS_ENABLED.lock().unwrap()
}

pub(crate) fn adjust_stat<T: ?Sized>(change: i64) {
    if !stats_enabled() {
        return;
    }

    let mut stats = STATS.lock().unwrap();

    let name = std::any::type_name::<T>().to_string();

    let count = match stats.get_mut(&name) {
        Some(count) => count,
        None => {
            stats.insert(name.clone(), 0);
            stats.get_mut(&name).unwrap()
        }
    };

    *count += change;

    debug_assert!(*count >= 0);
}

#[cfg(test)]
pub(crate) fn get_stat<T>() -> i64 {
    let name = std::any::type_name::<T>().to_string();
    *STATS.lock().unwrap().get(&name).unwrap_or(&0)
}

pub fn dump_ref_stats() {
    let stats = STATS.lock().unwrap();

    if stats.is_empty() {
        println!("No managed objects.");
    }

    for (name, count) in stats.iter() {
        println!("Type: {name}, count: {count}");
    }
}
