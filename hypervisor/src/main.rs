#![no_main]
#![cfg_attr(not(test), no_std)]
#![feature(panic_info_message, global_asm, llvm_asm, asm)]

// extenal crates
extern crate elf_rs;
extern crate log;

// modules
#[macro_use]
pub mod uart;
#[macro_use]
pub mod riscv;
pub mod boot;
pub mod memlayout;
pub mod paging;
pub mod plic;

pub mod mkernel;

pub mod guest;
pub mod hypervisor;

pub mod debug;
pub mod util;

pub mod virtio;
