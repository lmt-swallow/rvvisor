# rvvisor

**NOTE: This project is still work in progress!**

**rvvisor** is a tiny hypervisor written in Rust, which partially supports RISC-V Hypervisor Extension v0.6.1 included in [Volume II: RISC-V Privileged Architectures V1.12-draft](https://riscv.org/technical/specifications/)). 

## Requirements

This project relies on the following tools.

- [riscv/riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain)
- [QEMU with RISC-V Hypervisor Extension Emulation](https://github.com/kvm-riscv/qemu)

To run rvvisor, you need to install them and configure your `PATH` first. 

## Usage

Here's a list of the possible usecases:

- Run rvvisor with an example kernel
- (in the future) Run our hypervisor with your own kernel

### Run rvvisor with an example kernel

You can run the simple guest kernel, whose implementation is in `./guest` directory, as follows.

```sh
rustup target add riscv64gc-unknown-none-elf || true

# build hypervisor
cd hypervisor
CC=riscv64-unknown-linux-gnu-gcc cargo build
cd ..

# build guest
cd guest
CC=riscv64-unknown-linux-gnu-gcc cargo build
cd ..

# run hypervisor with guest
cd hypervisor
cargo run -- -drive file=../guest/target/riscv64gc-unknown-none-elf/debug/riscv-virt-guest,if=none,format=raw,id=x0 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
```

### Run rvvisor with your own kernel

Currently, due to the lack of features, famous kernels like [xv6-riscv](https://github.com/mit-pdos/xv6-riscv) or Linux do NOT work upon rrvisor.

### NOTE: Debug rvvisor with GDB

You can debug rvvisor with gdb like this:

```sh
# in a shell ...
$ cargo run -- -S -gdb tcp::9999 # + additional opts

# in another shell ...
$ riscv64-unknown-linux-gnu-gdb target/riscv64gc-unknown-none-elf/debug/rvvisor
...
(gdb) target remote localhost:9999
(gdb) continue
```

## Features (to be supported)

rvvisor currently supports the following features:

- :construction: Run a single VM upon rvvisor
    - [x] load ELF image into the memory space of a VM
    - [x] jump to the kernel image loaded to a VM image whiel enabling guest physical address translation by `hgatp`
    - [x] run a tiny kernel which does not require any external hardwares like disk devices
    - [ ] handle read/write requests for CSRs from a guest
    - [ ] handle SBI calls
- [ ] Run multiple VMs upon rvvisor
    - [ ] switch CPU contexts among guests
    - [ ] schedule the guest in a fancy way
- [ ] Support multi-core host environment
- [ ] Support device virtualization
    - [ ] block device
    - [ ] network device
    - [ ] input device
    - [ ] display device