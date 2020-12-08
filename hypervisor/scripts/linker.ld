OUTPUT_ARCH("riscv")

ENTRY(m_entrypoint)

SECTIONS
{
    . = 0x80000000; 
    .text.entrypoint : 
    {        
        PROVIDE(_elf_start = .);
        *(.text.entrypoint);
    }

    .text :
    {
        *(.text) *(.text.*);
    }

    .rodata :
    {
        *(.rdata .rodata. .rodata.*);
    }

    . = ALIGN(4096);
    .data :
    {
        *(.data .data.*);
    }

    _bss_start = .;
    .bss :
    {
        *(.bss .bss.*);
        PROVIDE(_elf_end = .);    
    }  
}