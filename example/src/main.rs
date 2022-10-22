use refs::{Own, ToWeak};

fn main() {
    let num = Own::new(5);

    dbg!(num.weak());

    dbg!(num);
}
