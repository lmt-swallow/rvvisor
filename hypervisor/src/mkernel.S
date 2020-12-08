.option norvc
.altmacro

.section .text.mkernel
.global trap_to_m

.macro load_gp i, base
	ld	x\i, ((\i)*8)(\base)
.endm

.macro save_gp i, base
	sd	x\i, ((\i)*8)(\base)
.endm

.align 4
trap_to_mkernel:
    csrrw	t6, mscratch, t6

    # save GPRs    
    .set 	i, 0
	.rept	31
		save_gp	%i, t6
		.set	i, i+1
	.endr

    mv		t5, t6
	csrr	t6, mscratch
	save_gp 31, t5  

    # save sscratch
    csrw	mscratch, t5

	csrr	a0, mepc
	csrr	a1, mtval
	csrr	a2, mcause
	csrr	a3, mstatus
	csrr	a4, mscratch
	la		sp, _intr_stack_end

    # -------

    call   rust_mtrap_handler

    # -------

    # after getting back from rust_mtrap_handler ...
	csrw	mepc, a0
	csrr	t6, mscratch

    # restore GPRs
	.set	i, 1
	.rept	31
		load_gp %i, t6
		.set	i, i+1
	.endr

	mret

.bss
.global _m_stack_start, _m_stack_end
_m_stack_start:
    # allocate 16 MB
    .skip 1024 * 1024 * 16
_m_stack_end:
    .skip 1024

.global _mintr_stack_start, _mintr_stack_end
_mintr_stack_start:
    .skip 1024 * 1024 
_mintr_stack_end:
    .skip 1024        