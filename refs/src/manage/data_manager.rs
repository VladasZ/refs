use std::path::{Path, PathBuf};

use crate::{
    Own, Weak,
    manage::{DataStorage, Managed},
};

pub trait DataManager<T: Managed> {
    fn root_path() -> &'static Path;
    fn set_root_path(path: impl Into<PathBuf>);

    fn storage() -> &'static mut DataStorage<T>;

    fn full_path(name: &str) -> PathBuf {
        Self::root_path().join(name)
    }

    fn free_with_name(name: impl ToString) {
        Self::storage().remove(&name.to_string());
    }

    fn free(self: Weak<Self>) {
        if self.is_null() {
            return;
        }
        let storage = Self::storage();
        let key = storage
            .iter()
            .find(|(_, val)| val.addr() == self.addr())
            .expect("Failed to find object to free.")
            .0
            .clone();
        storage.remove(&key);
    }

    fn add_with_name(name: &str, create: impl FnOnce() -> T) -> Weak<T> {
        Self::storage()
            .entry(name.to_string())
            .or_insert_with(|| Own::new(create()))
            .weak()
    }

    fn get_static(self: Weak<Self>) -> &'static T {
        Self::storage()
            .iter()
            .find(|(_, val)| val.addr() == self.addr())
            .expect("Failed to get_static managed")
            .1
    }

    fn get_existing(name: impl ToString) -> Option<Weak<T>> {
        Self::storage().get(&name.to_string()).map(Own::weak)
    }

    fn get(name: impl ToString) -> Weak<T> {
        let name = name.to_string();
        let storage = Self::storage();
        let val = storage
            .entry(name.clone())
            .or_insert_with(|| Own::new(T::load_path(&Self::full_path(&name))));
        val.weak()
    }

    fn load(data: &[u8], name: impl ToString) -> Weak<T> {
        let name = name.to_string();
        let storage = Self::storage();
        let val = storage
            .entry(name.clone())
            .or_insert_with(|| Own::new(T::load_data(data, name)));
        val.weak()
    }
}
