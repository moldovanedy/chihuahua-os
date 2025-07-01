#![no_std]
#![feature(abi_x86_interrupt)]

#[allow(dead_code)]

#[allow(unused_imports)]
use k_panic_handler;

pub mod log;
pub mod platform_initializer;
pub mod ports;
pub mod renderer;
pub mod interrupts;

mod arch;
mod k_drivers;
