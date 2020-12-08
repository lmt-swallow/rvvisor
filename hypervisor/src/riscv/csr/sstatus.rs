define_read!(0x100);
define_write!(0x100);

pub fn set_spp(mode: crate::riscv::csr::CpuMode) {
    if mode == crate::riscv::csr::CpuMode::M {
        log::debug!("set_spp was called riscv::csr::CpuMode::M")
    }

    let sstatus = read();
    let spp_mask = !(0b1 << 7 as usize);
    write((sstatus & spp_mask) | (((mode as usize) & 1) << 7))
}

pub fn set_sie(enabled: bool) {
    let sstatus = read();
    let sie_mask = 1 << 1 as usize;
    write((sstatus & sie_mask) | (if enabled { 1 << 1 } else { 0 }));
}
