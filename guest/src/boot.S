.option norvc

.section .text.entrypoint
.global entrypoint

entrypoint:
    # load stack addr
    la     sp, _stack_end
    # jump to rust code
    tail       rust_entrypoint