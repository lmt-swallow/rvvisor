define_read!(0x300);
define_write!(0x300);

pub fn set_mpp(mode: crate::riscv::csr::CpuMode) {
    let mstatus = read();
    let mpp_mask = !(0b11 << 11 as usize);
    write((mstatus & mpp_mask) | ((mode as usize) << 11))
}

pub fn set_mpv(mode: crate::riscv::csr::VirtualzationMode) {
    let mstatus = read();
    let mpv_mask = !(0b1 << 39 as usize);
    write((mstatus & mpv_mask) | ((mode as usize) << 39))
}
