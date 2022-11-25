use refs::{dump_ref_stats, enable_ref_stats_counter, is_main_thread, thread_id, Own, ToWeak};
use std::thread::spawn;

extern crate rtools;

fn main() {
    enable_ref_stats_counter(true);

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

    dump_ref_stats();

    rtools::sleep(1);
}
