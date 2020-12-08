global_asm!(include_str!("hypervisor.S"));

use crate::guest::Guest;
use crate::memlayout;
use crate::paging;
use crate::plic;
use crate::riscv;
use crate::uart;
use crate::virtio;
use core::fmt::Error;

extern "C" {
    #[link_name = "hypervisor_entrypoint"]
    pub fn entrypoint();

    #[link_name = "trap_to_hypervisor"]
    pub fn trap();
}

#[no_mangle]
pub fn rust_hypervisor_entrypoint() -> ! {
    log::info!("hypervisor started");

    if let Err(e) = init() {
        panic!("Failed to init tiny-hypervisor. {:?}", e)
    }
    log::info!("succeeded in initializing tiny hypervisor");

    // TODO (enhnancement): multiplex here
    let guest_name = "guest01";
    log::info!("a new guest instance: {}", guest_name);
    log::info!("-> create metadata set");
    let mut guest = Guest::new(guest_name);
    log::info!("-> load a tiny kernel image");
    guest.load_from_disk();

    log::info!("switch to guest");
    switch_to_guest(&guest);
}

pub fn init() -> Result<(), Error> {
    // inti memory allocator
    paging::init();

    // init virtio
    virtio::init();

    // hedeleg: delegate some synchoronous exceptions
    riscv::csr::hedeleg::write((1 << 0) | (1 << 3) | (1 << 8) | (1 << 12) | (1 << 13) | (1 << 15));

    // hideleg: delegate all interrupts
    riscv::csr::hideleg::write(
        riscv::csr::hideleg::VSEIP | riscv::csr::hideleg::VSTIP | riscv::csr::hideleg::VSSIP,
    );

    // hvip: clear all interrupts first
    riscv::csr::hvip::write(0);

    // stvec: set handler
    riscv::csr::stvec::set(&(trap as unsafe extern "C" fn()));
    assert_eq!(
        riscv::csr::stvec::read(),
        (trap as unsafe extern "C" fn()) as usize
    );

    // allocate memory region for TrapFrame and set it sscratch
    let trap_frame = paging::alloc();
    riscv::csr::sscratch::write(trap_frame.address().to_usize());
    log::info!("sscratch: {:016x}", riscv::csr::sscratch::read());

    // enable interupts
    enable_interrupt();

    // TODO: hip and sip
    // TODO: hie and sie

    // leave
    Ok(())
}

fn enable_interrupt() {
    // TODO (enhancement): UART0

    // configure PLIC
    plic::enable_interrupt();

    // sie; enable external interrupt
    // TODO (enhancement): timer interrupt
    // TODO (enhancement): software interrupt
    let current_sie = riscv::csr::sie::read();
    riscv::csr::sie::write(current_sie | (riscv::csr::sie::SEIE as usize));

    // sstatus: enable global interrupt
    riscv::csr::sstatus::set_sie(true);
}

pub fn switch_to_guest(target: &Guest) -> ! {
    // hgatp: set page table for guest physical address translation
    riscv::csr::hgatp::set(&target.hgatp);
    riscv::instruction::hfence_gvma();
    assert_eq!(target.hgatp.to_usize(), riscv::csr::hgatp::read());

    // hstatus: handle SPV change the virtualization mode to 0 after sret
    riscv::csr::hstatus::set_spv(riscv::csr::VirtualzationMode::Guest);

    // sstatus: handle SPP to 1 to change the privilege level to S-Mode after sret
    riscv::csr::sstatus::set_spp(riscv::csr::CpuMode::S);

    // sepc: set the addr to jump
    riscv::csr::sepc::set(&target.sepc);

    // jump!
    riscv::instruction::sret();
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32],  // 0 - 255
    pub fregs: [usize; 32], // 256 - 511
    pub pc: usize,          // 512
}

#[no_mangle]
pub extern "C" fn rust_strap_handler(
    sepc: usize,           // a0
    stval: usize,          // a1
    scause: usize,         // a2
    sstatus: usize,        // a3
    frame: *mut TrapFrame, // a4
) -> usize {
    log::debug!("<--------- trap --------->");
    log::debug!("sepc: 0x{:016x}", sepc,);
    log::debug!("stval: 0x{:016x}", stval,);
    log::debug!("scause: 0x{:016x}", scause,);
    log::debug!("sstatus: 0x{:016x}", sstatus,);

    let is_async = scause >> 63 & 1 == 1;
    let cause_code = scause & 0xfff;
    if is_async {
        match cause_code {
            // external interrupt
            9 => {
                if let Some(interrupt) = plic::get_claim() {
                    log::debug!("interrupt id: {}", interrupt);
                    match interrupt {
                        1..=8 => {
                            virtio::handle_interrupt(interrupt);
                        }
                        10 => {
                            uart::handle_interrupt();
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                    plic::complete(interrupt);
                } else {
                    panic!("invalid state")
                }
            }
            // timer interrupt & software interrrupt
            _ => {
                unimplemented!();
            }
        }
    } else {
        match cause_code {
            8 => {
                log::info!("environment call from U-mode / VU-mode at 0x{:016x}", sepc);
                // TODO: better handling
                loop {}
            }
            10 => {
                log::info!("environment call from VS-mode at 0x{:016x}", sepc);
                // TODO: better handling
                loop {}
            }
            21 => {
                log::info!("exception: load guest page fault at 0x{:016x}", sepc);
                // TODO (enhancement): demand paging
                loop {}
            }
            23 => {
                log::info!("exception: store/amo guest-page fault at 0x{:016x}", sepc);
                // TODO: better handling
                loop {}
            }
            _ => {
                unimplemented!();
            }
        }
    }
    sepc
}
