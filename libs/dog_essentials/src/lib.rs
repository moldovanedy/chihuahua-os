#![no_std]
#![feature(unsafe_cell_access)]

pub mod format_non_alloc;
pub mod pointer_ops;
pub mod static_cell;
pub mod sync;

fn consume_panic() {
    k_corelib::use_panic();
}
