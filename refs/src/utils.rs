use std::thread::{current, Thread};

pub fn thread_id() -> String {
    Thread::name(&current()).unwrap_or("failed_to_get").into()
}

pub fn is_main_thread() -> bool {
    thread_id() == "main"
}
