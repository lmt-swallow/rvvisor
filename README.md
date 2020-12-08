# riscv-virt-example

This repository includes a tiny implementation that uses RISC-V Hypervisor Extension (v0.6.1).

## Requirements

TODO

- [riscv/riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)
- [QEMU with RISC-V Hypervisor Extension Emulation](https://github.com/kvm-riscv/qemu)

## How to run

TODO

```sh
rustup target add riscv64gc-unknown-none-elf || true

# build hypervisor
cd hypervisor
CC=riscv64-unknown-linux-gnu-gcc cargo run
cd..

# build guest
cd guest
CC=riscv64-unknown-linux-gnu-gcc cargo run
cd..

# run hypervisor with guest
cd hypervisor
cargo run -- -drive file=../guest/target/riscv64gc-unknown-none-elf/debug/riscv-virt-guest,if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
```

## How to debug

```sh
# in a shell ...
$ cargo run -- -S -gdb tcp::9999

# in another shell ...
$ riscv64-unknown-linux-gnu-gdb target/riscv64gc-unknown-none-elf/debug/riscv-virt-example
...
(gdb) target remote localhost:9999
```

## How to test

TODO

```sh
rustup target add riscv64gc-unknown-linux-gnu || true
cargo test --target=riscv64gc-unknown-linux-gnu
```

## Notes

- This implementation assumes `-smp=1`.