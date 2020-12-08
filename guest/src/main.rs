#![no_main]
#![cfg_attr(not(test), no_std)]
#![feature(panic_info_message, global_asm, llvm_asm, asm)]

// extenal crates
extern crate log;

// modules
#[macro_use]
pub mod uart;
pub mod boot;
mod debug;
pub mod kernel;
pub mod memlayout;