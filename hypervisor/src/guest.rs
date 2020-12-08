use crate::memlayout;
use crate::paging;
use crate::riscv;
use crate::virtio;
use core::fmt::Error;
use elf_rs::Elf;

pub struct Guest {
    pub name: &'static str,
    pub hgatp: riscv::csr::hgatp::Setting,
    pub sepc: usize,
    // TODO: other CSRs & registers
}

impl Guest {
    pub fn new(name: &'static str) -> Guest {
        // hgatp
        let root_pt = prepare_gpat_pt().unwrap();
        let hgatp = riscv::csr::hgatp::Setting::new(
            riscv::csr::hgatp::Mode::Sv39x4,
            0,
            root_pt.page.address().to_ppn(),
        );

        Guest {
            name: name,
            hgatp: hgatp,
            sepc: memlayout::GUEST_DRAM_START,
        }
    }

    pub fn load_from_disk(&mut self) {
        let load_size = 1024 * 1024 * 2;
        let buf_page = paging::alloc_continuous(load_size / memlayout::PAGE_SIZE as usize);
        let buf_addr = buf_page.address().to_usize() as *mut u8;
        unsafe {
            let sector_max: u64 = load_size as u64 / virtio::SECTOR_SIZE as u64;
            if let Some(_queue) = virtio::QUEUE {
                for sector in 0..sector_max {
                    (*_queue).read(
                        sector as u64,
                        buf_addr.offset(sector as isize * virtio::SECTOR_SIZE as isize)
                            as *const (),
                    );
                    if sector % 10 == 9 {
                        log::debug!("progress: {} / {}", sector + 1, sector_max)
                    }
                }
            }
            log::debug!("an ELF was copied into a buffer")
        }

        let gpat_pt = paging::PageTable::from_page(paging::Page::from_address(
            paging::PhysicalAddress::new(self.hgatp.ppn << 12),
        ));
        unsafe {
            let buf: &mut [u8] = core::slice::from_raw_parts_mut(buf_addr, load_size as usize);

            // TODO (enhancement): care about page permissions
            let elf = Elf::from_bytes(buf);
            match elf {
                Ok(Elf::Elf64(e)) => {
                    // change entrypoint
                    self.sepc = e.header().entry_point() as usize;
                    log::info!("-> entrypoint: 0x{:016x}", self.sepc);
                    // copy each section (page to page)
                    for s in e.section_header_iter() {
                        if s.sh.addr() > 0 && s.sh.sh_type() == elf_rs::SectionType::SHT_PROGBITS {
                            log::info!(
                                "-> section found: name={}, address:0x{:016x}, offset=0x{:016x}",
                                s.section_name(),
                                s.sh.addr(),
                                s.sh.size()
                            );
                            let start_page_head = s.sh.addr() >> 12 << 12;
                            let end_page_head = (s.sh.addr() + s.sh.size()) >> 12 << 12;
                            let last_idx =
                                (end_page_head - start_page_head) / (memlayout::PAGE_SIZE as u64);
                            let mut seek = (s.sh.addr() & 0xfff) as usize;

                            for i in 0..=last_idx {
                                let dest_base_vaddr = paging::VirtualAddress::new(
                                    (start_page_head + i * (memlayout::PAGE_SIZE as u64)) as usize,
                                );
                                let dest_addr = (gpat_pt.resolve(&dest_base_vaddr).to_usize()
                                    as *mut u8)
                                    .add(seek % (memlayout::PAGE_SIZE) as usize);
                                let src_addr = buf_addr
                                    .offset(s.sh.offset() as isize)
                                    .add(seek - (s.sh.addr() & 0xfff) as usize);
                                let copy_size = core::cmp::min(
                                    (i + 1) as usize * memlayout::PAGE_SIZE as usize,
                                    ((s.sh.addr() + s.sh.size()) as usize)
                                        - (start_page_head as usize),
                                ) as usize
                                    - seek;
                                seek += copy_size;
                                core::ptr::copy(src_addr, dest_addr, copy_size);
                                log::debug!(
                                    "{:016x}: {:0x} {:0x} {:0x} {:0x}",
                                    dest_base_vaddr.to_usize(),
                                    dest_addr.read_volatile(),
                                    dest_addr.add(1).read_volatile(),
                                    dest_addr.add(2).read_volatile(),
                                    dest_addr.add(3).read_volatile()
                                );
                            }
                        }
                    }
                    log::info!("-> the ELF was extracted into the guest memory");
                }
                Ok(Elf::Elf32(_)) => {
                    panic!("-> 32bit ELF is not supported");
                }
                Err(e) => {
                    panic!("-> failed to parse ELF file. {:?}", e);
                }
            }
        }
    }
}

// This function return newly allocated page table for Guest Physical Address Translation.
fn prepare_gpat_pt() -> Result<paging::PageTable, Error> {
    // NOTE (from the RISC-V specification):
    // As explained in Section 5.5.1, for the paged virtual-memory schemes (Sv32x4, Sv39x4, and Sv48x4),
    // the root page table is 16 KiB and must be aligned to a 16-KiB boundary. In these modes, the lowest
    // two bits of the physical page number (PPN) in hgatp always read as zeros. An implementation
    // that supports only the defined paged virtual-memory schemes and/or Bare may hardwire PPN[1:0]
    // to zero

    // get a 16KiB-aligned & 16KiB page
    // TODO (enhancement): this is a native implementation. we need to implement a more clean allocator in `paging` crate.
    let root_page = paging::alloc_16();
    log::info!(
        "a page 0x{:016x} was allocated for a guest page address translation page table",
        root_page.address().to_usize()
    );
    let root_pt = paging::PageTable::from_page(root_page);

    // create an identity map for UART MMIO
    let vaddr = memlayout::GUEST_UART_BASE;
    let page = paging::Page::from_address(paging::PhysicalAddress::new(vaddr));
    root_pt.map(
        paging::VirtualAddress::new(vaddr),
        &page,
        (paging::PageTableEntryFlag::Read as u16)
            | (paging::PageTableEntryFlag::Write as u16)
            | (paging::PageTableEntryFlag::Execute as u16)
            | (paging::PageTableEntryFlag::User as u16), // required!
    );

    // allocating new pages and map GUEST_DRAM_START ~ GUEST_DRAM_END into those pages for guest kernel
    let map_page_num = (memlayout::GUEST_DRAM_END - memlayout::GUEST_DRAM_START)
        / (memlayout::PAGE_SIZE as usize)
        + 1;
    for i in 0..map_page_num {
        let vaddr = memlayout::GUEST_DRAM_START + i * (memlayout::PAGE_SIZE as usize);
        let page = paging::alloc();
        root_pt.map(
            paging::VirtualAddress::new(vaddr),
            &page,
            (paging::PageTableEntryFlag::Read as u16)
                | (paging::PageTableEntryFlag::Write as u16)
                | (paging::PageTableEntryFlag::Execute as u16)
                | (paging::PageTableEntryFlag::User as u16), // required!
        )
    }

    Ok(root_pt)
}
