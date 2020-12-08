.option norvc

.section .text.entrypoint
.global m_entrypoint

m_entrypoint:
    la a0, _trapframe
    csrw mscratch, a0
    # load stack addr
    la     sp, _m_stack_end
    # jump to rust code
    tail       rust_m_entrypoint

.global _trapframe
_trapframe:
    .skip 1024 * 1024 
