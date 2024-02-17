#![cfg(test)]
use vents::Event;

use crate::{
    editor::{EditedCallback, Editor},
    set_current_thread_as_main, Own,
};

#[derive(Default)]
struct Data {
    number: i32,
}

#[derive(Default)]
struct DataHolder {
    data:          Data,
    number_edited: Event<i32>,
}

impl DataHolder {
    fn data(&mut self) -> Editor<DataHolder, Data> {
        Editor::new(self, &self.data)
    }
}

impl EditedCallback for DataHolder {
    fn edited(&self) {
        self.number_edited.trigger(self.data.number);
    }
}

#[test]
fn test_editor() {
    set_current_thread_as_main();

    let mut data = DataHolder::default();

    let test = Own::new(0);
    let mut test = test.weak();

    assert_eq!(*test, 0);

    data.number_edited.val(move |a| {
        *test += a;
    });

    data.data().number = 10;
    assert_eq!(*test, 10);

    data.data().number = 40;
    assert_eq!(*test, 50);
}
