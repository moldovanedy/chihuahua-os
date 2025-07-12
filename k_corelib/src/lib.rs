#![no_std]
#![feature(abi_x86_interrupt)]

#[allow(dead_code)]
#[allow(unused_imports)]
use k_panic_handler;

pub mod interrupts;
pub mod log;
pub mod platform_initializer;
pub mod ports;
pub mod renderer;
pub mod mem_manager;

mod arch;
mod k_drivers;
