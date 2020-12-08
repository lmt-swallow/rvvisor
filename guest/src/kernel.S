.option norvc

.section .text.kernel
.global trap_to_kernel

.align 4
trap_to_kernel:
    # TODO: save registers    

    # jump to a handler written in Rust
    call   rust_trap_handler

    # TODO: restore registers
    # TODO: mret again

.bss
.global _stack_start, _stack_end
_stack_start:
    # allocate 1 MB
    .skip 1024 * 1024
_stack_end:
    .skip 1024
