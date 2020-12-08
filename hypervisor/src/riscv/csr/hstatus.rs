define_read!(0x600);
define_write!(0x600);

pub fn set_spv(mode: crate::riscv::csr::VirtualzationMode) {
    let hstatus = read();
    let spv_mask = !(0b1 << 7 as usize);
    write((hstatus & spv_mask) | ((mode as usize) << 7))
}
