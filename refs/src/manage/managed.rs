#[macro_export]
macro_rules! managed {
    ($type:ident) => {
        static _MANAGED_ROOT_PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
        static _STORAGE: refs::main_lock::MainLock<refs::manage::DataStorage<$type>> =
            refs::main_lock::MainLock::new();

        impl refs::manage::Managed for $type {}

        impl refs::manage::DataManager<$type> for $type {
            fn root_path() -> &'static std::path::Path {
                _MANAGED_ROOT_PATH.get().expect(&format!(
                    "Managed root path for type {} is not set.",
                    stringify!($type)
                ))
            }

            fn set_root_path(path: impl Into<std::path::PathBuf>) {
                let path = path.into();
                _MANAGED_ROOT_PATH.set(path.to_path_buf()).expect(&format!(
                    "Managed root path for type {} was already set set.",
                    stringify!($type)
                ))
            }

            fn storage() -> &'static mut refs::manage::DataStorage<$type> {
                _STORAGE.get_mut()
            }
        }
    };
}
