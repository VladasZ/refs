use refs::{is_main_thread, thread_id, Own, ToWeak};
use std::thread::spawn;

extern crate rtools;

fn main() {
    let num = Own::new(5);

    dbg!(num.weak());

    dbg!(&num);

    dbg!(thread_id());
    dbg!(is_main_thread());

    let wee = num.weak();

    spawn(move || {
        dbg!(thread_id());
        dbg!(is_main_thread());

        dbg!(wee);
    });

    rtools::sleep(1);
}
