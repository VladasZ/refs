use std::thread::{current, Thread};

pub fn thread_id() -> String {
    match Thread::name(&current()) {
        Some(name) => name.into(),
        None => {
            let id = current().id().as_u64();
            format!("{id}")
        }
    }
}

#[cfg(not(target_os = "ios"))]
pub fn is_main_thread() -> bool {
    thread_id() == "main"
}

#[cfg(target_os = "ios")]
pub fn is_main_thread() -> bool {
    current().id().as_u64() == std::num::NonZeroU64::new(1).unwrap()
}
