global_asm!(include_str!("instruction.S"));

pub fn mret() -> ! {
    unsafe {
        asm!("mret");
    }

    loop {}
}

pub fn sret() -> ! {
    unsafe {
        asm!("sret");
    }

    loop {}
}

pub fn ecall() {
    unsafe {
        asm!("ecall");
    }
}

pub fn sfenve_vma() {
    unsafe {
        asm!("sfence.vma");
    }
}

extern "C" {
    fn __hfence_gvma_all();
}

pub fn hfence_gvma() {
    unsafe {
        __hfence_gvma_all();
    }
}

pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
