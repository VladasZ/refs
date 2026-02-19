use std::{
    mem::transmute,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::Result;
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

use crate::{
    Own, Weak,
    manage::{DataStorage, Managed},
};

pub trait DataManager<T: Managed> {
    fn root_path() -> &'static Path;
    fn set_root_path(path: impl Into<PathBuf>);

    fn storage() -> RwLockReadGuard<'static, DataStorage<T>>;
    fn storage_mut() -> RwLockWriteGuard<'static, DataStorage<T>>;

    fn full_path(name: &str) -> PathBuf {
        Self::root_path().join(name)
    }

    fn free_with_name(name: impl ToString) {
        Self::storage_mut().remove(&name.to_string());
    }

    fn free(self: Weak<Self>) {
        if self.is_null() {
            return;
        }
        let mut storage = Self::storage_mut();
        let key = storage
            .iter()
            .find(|(_, val)| val.addr() == self.addr())
            .expect("Failed to find managed object to free.")
            .0
            .clone();
        storage.remove(&key);
    }

    fn store_with_name<E>(name: &str, create: impl FnOnce() -> Result<T, E>) -> Result<Weak<T>, E> {
        if let Some(entry) = Self::storage().get(name) {
            return Ok(entry.weak());
        }

        let entry = Own::new(create()?);

        let weak = entry.weak();

        Self::storage_mut().insert(name.to_owned(), entry);

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

        if let Some(existing) = Self::storage().get(&name) {
            return existing.weak();
        }

        let new = Own::new(T::load_path(&Self::full_path(&name)));
        let weak = new.weak();

        Self::storage_mut().insert(name, new);

        weak
    }

    fn load(data: &[u8], name: impl ToString) -> Weak<T> {
        let name = name.to_string();

        if let Some(existing) = Self::storage().get(&name) {
            return existing.weak();
        }

        let new = Own::new(T::load_data(data, &name));
        let weak = new.weak();

        Self::storage_mut().insert(name, new);

        weak
    }

    #[allow(async_fn_in_trait)]
    async fn download(name: impl ToString, url: &str) -> Result<Weak<T>> {
        let name = name.to_string();

        let data = reqwest::get(url).await?.bytes().await?;

        Ok(Self::load(&data, &name))
    }
}
