#![allow(incomplete_features)]
#![allow(internal_features)]
#![feature(specialization)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(thread_id_value)]
#![feature(core_intrinsics)]
#![feature(const_type_name)]
#![feature(arbitrary_self_types)]

mod address;
mod as_any;
mod erased;
mod from_ref;
mod into_own;
mod own;
mod ref_counter;
mod rglica;
mod stats;
mod to_rglica;
mod utils;
mod weak;

pub use address::*;
pub use as_any::*;
pub use erased::*;
pub use from_ref::*;
pub use own::*;
pub use rglica::*;
pub use stats::*;
pub use to_rglica::*;
pub use utils::*;
pub use weak::*;

pub mod editor;
pub mod main_lock;
pub mod manage;
pub mod vec;
mod tests;
