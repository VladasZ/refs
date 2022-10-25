use refs::{is_main_thread, thread_id, MainState, Own, ToWeak};
use std::sync::Mutex;
use std::thread::spawn;

extern crate rtools;

static LOCK: Mutex<bool> = Mutex::new(false);

fn main() {
    MainState::set_lock_check(|| *LOCK.lock().unwrap());

    let num = Own::new(5);

    dbg!(num.weak());

    dbg!(&num);

    dbg!(thread_id());
    dbg!(is_main_thread());
    dbg!(MainState::safe());

    let wee = num.weak();

    spawn(move || {
        dbg!(thread_id());
        dbg!(is_main_thread());
        dbg!(MainState::safe());
        *LOCK.lock().unwrap() = true;
        dbg!(MainState::safe());

        dbg!(wee);
    });

    spawn(move || {
        dbg!(thread_id());
        dbg!(is_main_thread());
        dbg!(MainState::safe());
        dbg!(wee);
    });

    rtools::sleep(1);
}
