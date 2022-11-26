use refs::{
    dump_ref_stats, enable_ref_stats_counter, is_main_thread, set_current_thread_as_main,
    thread_id, Own, ToWeak,
};
use std::thread::spawn;

extern crate rtools;

struct Drep {
    _int: i32,
}

impl Drop for Drep {
    fn drop(&mut self) {
        dbg!("Drep!");
    }
}

fn main() {
    enable_ref_stats_counter(true);

    let drep = Own::new(Drep { _int: 5 });

    drop(drep);

    dbg!("m?");

    let num = Own::new(5);

    dbg!(num.weak());

    dbg!(&num);

    dbg!(thread_id());
    dbg!(is_main_thread());

    let wee = num.weak();

    spawn(move || {
        dbg!(thread_id());
        dbg!(is_main_thread());

        set_current_thread_as_main();

        dbg!(wee);
    });

    dump_ref_stats();

    rtools::sleep(1);
}
