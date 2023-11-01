#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(thread_id_value)]
#![feature(arbitrary_self_types)]
#![feature(core_intrinsics)]

pub mod address;
pub mod as_any;
pub mod from_ref;
pub mod main_lock;
pub mod own;
pub(crate) mod ref_deallocators;
pub mod rglica;
pub mod stats;
pub mod to_own;
pub mod to_rglica;
pub mod total_size;
pub mod utils;
pub mod vec;
pub mod weak;

pub use address::*;
pub use as_any::*;
pub use from_ref::*;
pub use main_lock::*;
pub use own::*;
pub use rglica::*;
pub use stats::*;
pub use to_own::*;
pub use to_rglica::*;
pub use total_size::*;
pub use utils::*;
pub use weak::*;
