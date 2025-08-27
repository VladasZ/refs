#![allow(incomplete_features)]
#![allow(internal_features)]
#![feature(specialization)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(thread_id_value)]
#![feature(core_intrinsics)]
#![feature(const_type_name)]

pub mod address;
pub mod as_any;
pub mod editor;
mod erased;
pub mod from_ref;
pub mod into_own;
pub mod main_lock;
pub mod own;
pub(crate) mod ref_counter;
pub mod rglica;
pub mod stats;
pub mod to_rglica;
pub mod utils;
pub mod vec;
pub mod weak;

pub use address::*;
pub use as_any::*;
pub use erased::*;
pub use from_ref::*;
pub use main_lock::*;
pub use own::*;
pub use rglica::*;
pub use stats::*;
pub use to_rglica::*;
pub use utils::*;
pub use weak::*;
