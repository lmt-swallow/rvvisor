OUTPUT_ARCH("riscv")

ENTRY(entrypoint)

SECTIONS
{
    . = 0x80000000; 
    .text.entrypoint : 
    {        
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
    }  
}