#![no_std]
#![feature(unsafe_cell_access)]

//this is needed so we can have the panic handler from k_corelib;
#[allow(unused_imports)]
use k_corelib;

pub mod format_non_alloc;
pub mod pointer_ops;
pub mod static_cell;
pub mod sync;
