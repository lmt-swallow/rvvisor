#[macro_use]
mod macros;

#[derive(PartialEq)]
pub enum CpuMode {
    M = 0b11,
    S = 0b01,
    U = 0b00,
}

pub enum VirtualzationMode {
    Host = 0,
    Guest = 1,
}

pub mod medeleg;
pub mod mepc;
pub mod mideleg;
pub mod mie;
pub mod misa;
pub mod mstatus;
pub mod mtvec;

pub mod satp;
pub mod sepc;
pub mod sie;
pub mod sscratch;
pub mod sstatus;
pub mod stvec;

pub mod hcontext;
pub mod hedeleg;
pub mod hgatp;
pub mod hgeie;
pub mod hgeip;
pub mod hideleg;
pub mod hie;
pub mod hip;
pub mod hstatus;
pub mod htval;
pub mod hvip;

pub mod vsepc;
