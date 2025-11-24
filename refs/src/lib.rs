#![allow(incomplete_features)]
#![allow(internal_features)]
#![feature(specialization)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(thread_id_value)]
#![feature(core_intrinsics)]
#![feature(const_type_name)]
#![feature(arbitrary_self_types)]

mod as_any;
mod erased;
mod from_ref;
mod into_own;
mod own;
mod raw_pointer;
mod ref_counter;
mod rglica;
mod to_rglica;
mod utils;
mod weak;

pub use as_any::*;
pub use erased::*;
pub use from_ref::*;
pub use own::*;
pub use raw_pointer::*;
pub use rglica::*;
pub use to_rglica::*;
pub use utils::*;
pub use weak::*;

pub mod editor;
pub mod main_lock;
pub mod manage;
mod tests;
pub mod vec;

#[cfg(feature = "stats")]
pub mod stats;
