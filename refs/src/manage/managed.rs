#[macro_export]
macro_rules! managed {
    ($($refs_path:tt)::+, $type:ident) => {
        static _MANAGED_ROOT_PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
        static _STORAGE: $($refs_path)::+::Mutex<$($refs_path)::+::manage::DataStorage<$type>> =
            $($refs_path)::+::Mutex::new(std::collections::BTreeMap::new());

        impl $($refs_path)::+::manage::Managed for $type {}

        impl $($refs_path)::+::manage::DataManager<$type> for $type {
            fn root_path() -> &'static std::path::Path {
                static DEFAULT_PATH: std::path::PathBuf = std::path::PathBuf::new();

                if let Some(path) = _MANAGED_ROOT_PATH.get() {
                    return path;
                } else {
                    $($refs_path)::+::__internal_deps::warn!("Managed root path for type {} is not set.", stringify!($type));
                    return &DEFAULT_PATH
                }
            }

            fn set_root_path(path: impl Into<std::path::PathBuf>) {
                let path = path.into();
                _MANAGED_ROOT_PATH.set(path.to_path_buf()).expect(&format!(
                    "Managed root path for type {} was already set set.",
                    stringify!($type)
                ))
            }

            fn storage() -> $($refs_path)::+::MutexGuard<'static, $($refs_path)::+::manage::DataStorage<$type>> {
                _STORAGE.lock()
            }
        }
    };

    ($type:ident) => {
        managed!(refs, $type);
    };
}
