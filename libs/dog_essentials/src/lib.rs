#![no_std]
#![feature(unsafe_cell_access)]

//this is needed so we can have the panic handler from k_panic_handler;
#[allow(unused_imports)]
#[cfg(feature = "panic_handler")]
use k_panic_handler;

pub use lazy_static;

pub mod format_non_alloc;
pub mod geometry;
pub mod pointer_ops;
pub mod static_cell;
pub mod sync;
