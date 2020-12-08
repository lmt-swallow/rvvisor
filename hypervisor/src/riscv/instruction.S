.option nor

.section .text.instruction
.global __hfence_gvma_all

__hfence_gvma_all:
	.word 0x62000073
	ret