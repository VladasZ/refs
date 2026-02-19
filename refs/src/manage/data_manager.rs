use std::{
    mem::transmute,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::Result;
use parking_lot::MutexGuard;

use crate::{
    Own, Weak,
    manage::{DataStorage, Managed},
};

pub trait DataManager<T: Managed> {
    fn root_path() -> &'static Path;
    fn set_root_path(path: impl Into<PathBuf>);

    fn storage() -> MutexGuard<'static, DataStorage<T>>;

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
        let mut storage = Self::storage();
        let key = storage
            .iter()
            .find(|(_, val)| val.addr() == self.addr())
            .expect("Failed to find managed object to free.")
            .0
            .clone();
        storage.remove(&key);
    }

    fn store_with_name<E>(name: &str, create: impl FnOnce() -> Result<T, E>) -> Result<Weak<T>, E> {
        let mut storage = Self::storage();

        if let Some(entry) = storage.get(name) {
            return Ok(entry.weak());
        }

        let entry = Own::new(create()?);

        let weak = entry.weak();

        storage.insert(name.to_owned(), entry);

        Ok(weak)
    }

    unsafe fn get_static(self: Weak<Self>) -> &'static T {
        let storage = Self::storage();

        let rf = storage
            .iter()
            .find(|(_, val)| val.addr() == self.addr())
            .expect("Failed to get_static managed")
            .1;

        unsafe { transmute(rf.deref()) }
    }

    fn get_existing(name: impl ToString) -> Option<Weak<T>> {
        Self::storage().get(&name.to_string()).map(Own::weak)
    }

    fn get(name: impl ToString) -> Weak<T> {
        let name = name.to_string();
        let mut storage = Self::storage();
        let val = storage
            .entry(name.clone())
            .or_insert_with(|| Own::new(T::load_path(&Self::full_path(&name))));
        val.weak()
    }

    fn load(data: &[u8], name: impl ToString) -> Weak<T> {
        let name = name.to_string();
        let mut storage = Self::storage();
        let val = storage
            .entry(name.clone())
            .or_insert_with(|| Own::new(T::load_data(data, name)));
        val.weak()
    }

    #[allow(async_fn_in_trait)]
    async fn download(name: impl ToString, url: &str) -> Result<Weak<T>> {
        let name = name.to_string();

        let data = reqwest::get(url).await?.bytes().await?;

        Ok(Self::load(&data, &name))
    }
}
