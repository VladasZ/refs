#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(const_trait_impl)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_for)]
#![feature(ptr_metadata)]
#![feature(const_default_impls)]
#![feature(thread_id_value)]

pub mod own;
pub(crate) mod ref_counters;
pub mod rglica;
pub mod strong;
pub mod to_rglica;
pub mod to_weak;
pub mod utils;
pub mod weak;

pub use own::*;
pub(crate) use ref_counters::*;
pub use rglica::*;
pub use strong::*;
pub use to_rglica::*;
pub use to_weak::*;
pub use utils::*;
pub use weak::*;
