use rtools::platform::Platform;
use std::ops::Deref;
use std::sync::Mutex;
use std::thread::{current, Thread};

static MAIN_THREAD_NAME: Mutex<Option<String>> = Mutex::new(None);

pub fn thread_id() -> String {
    match Thread::name(&current()) {
        Some(name) => name.into(),
        None => {
            let id = current().id().as_u64();
            format!("{id}")
        }
    }
}

pub fn is_main_thread() -> bool {
    thread_id() == supposed_main_id()
}

pub fn set_current_thread_as_main() {
    let mut main = MAIN_THREAD_NAME.lock().unwrap();
    *main = thread_id().into();
}

pub(crate) fn supposed_main_id() -> String {
    let name = MAIN_THREAD_NAME.lock().unwrap();
    if let Some(name) = name.deref() {
        return name.clone();
    }
    if Platform::IOS { "1" } else { "main" }.into()
}
