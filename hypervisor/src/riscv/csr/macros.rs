macro_rules! define_read {
    ($csr_number:expr) => {
        #[inline]
        pub fn read() -> usize {
            unsafe {
                let r: usize;
                llvm_asm!("csrrs $0, $1, x0" : "=r"(r) : "i"($csr_number) :: "volatile");
                r
            }
        }
    };
}

macro_rules! define_write {
    ($csr_number:expr) => {
        pub fn write(v: usize) {
            unsafe {
                llvm_asm!("csrrw x0, $1, $0" :: "r"(v), "i"($csr_number) :: "volatile");
            }
        }
    };
}
