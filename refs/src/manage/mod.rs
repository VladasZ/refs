use std::collections::HashMap;

mod data_manager;
mod exists_managed;
mod managed;
mod resource_loader;

pub use data_manager::DataManager;
pub use exists_managed::ExistsManaged;
pub use resource_loader::ResourceLoader;

pub type DataStorage<T> = HashMap<String, crate::Own<T>>;

pub trait Managed: 'static + ResourceLoader + DataManager<Self> {}
