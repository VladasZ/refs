use std::thread::spawn;

use refs::{
    current_thread_id, dump_ref_stats, enable_ref_stats_counter, is_main_thread, set_current_thread_as_main,
    Own, ToWeak,
};

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

    dbg!(current_thread_id());
    dbg!(is_main_thread());

    let num_weak = num.weak();

    spawn(move || {
        dbg!(current_thread_id());
        dbg!(is_main_thread());

        set_current_thread_as_main();

        dbg!(num_weak);
    });

    dump_ref_stats();
}
