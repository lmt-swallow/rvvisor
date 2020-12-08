.option norvc
.altmacro

.section .text.hypervisor
.global hypervisor_entrypoint

.macro load_gp i, base
	ld	x\i, ((\i)*8)(\base)
.endm

.macro save_gp i, base
	sd	x\i, ((\i)*8)(\base)
.endm

.align 4
hypervisor_entrypoint:
    # TODO: copy registers    

    # jump to a handler written in Rust
    tail   rust_hypervisor_entrypoint

.align 4
trap_to_hypervisor:
    csrrw	t6, sscratch, t6

    # save GPRs    
    .set 	i, 0
	.rept	31
		save_gp	%i, t6
		.set	i, i+1
	.endr

    mv		t5, t6
	csrr	t6, sscratch
	save_gp 31, t5  

    # # save FPRs
	# .set 	i, 0
	# .rept	32
	# 	save_fp	%i, t5
	# 	.set	i, i+1
	# .endr    

    # save sscratch
    csrw	sscratch, t5

	csrr	a0, sepc
	csrr	a1, stval
	csrr	a2, scause
	csrr	a3, sstatus
	csrr	a4, sscratch
	la		sp, _intr_stack_end

    # -------
    
    # jump to a handler written in Rust
    call rust_strap_handler   

    # -------
    
    # after getting back from rust_strap_handler ...
	csrw	sepc, a0
	csrr	t6, sscratch

    # # restore FPRs
	# .set	i, 0
	# .rept	32
	# 	load_fp %i, t6
	# 	.set i, i+1
	# .endr

    # restore GPRs
	.set	i, 1
	.rept	31
		load_gp %i, t6
		.set	i, i+1
	.endr

	sret


.bss   
.global _hs_stack_start, _hs_stack_end
_hs_stack_start:
    .skip 1024 * 1024 * 16 #  16 MB
_hs_stack_end:
    .skip 1024    

.global _intr_stack_start, _intr_stack_end
_intr_stack_start:
    .skip 1024 * 1024  #  1 MB
_intr_stack_end:
    .skip 1024        