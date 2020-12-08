// TODO (enhancement):
// if we run more rich guest OS or add more rich features to hypervisor,
// we need to refine this implmentation :-D

use crate::memlayout::{elf_end, DRAM_END, PAGE_SIZE};

// VirtualAddress
/////

#[derive(Debug)]
pub struct VirtualAddress {
    addr: usize,
}

impl VirtualAddress {
    pub fn new(addr: usize) -> VirtualAddress {
        VirtualAddress { addr: addr }
    }

    pub fn to_vpn(&self) -> [usize; 3] {
        [
            (self.addr >> 12) & 0x1ff,
            (self.addr >> 21) & 0x1ff,
            (self.addr >> 30) & 0x1ff,
        ]
    }

    pub fn to_offset(&self) -> usize {
        self.addr & 0x3ff
    }

    pub fn to_usize(&self) -> usize {
        self.addr
    }

    pub fn as_pointer(&self) -> *mut usize {
        self.addr as *mut usize
    }
}

// PhysicalAddress
/////

#[derive(Copy, Clone, Debug)]
pub struct PhysicalAddress {
    addr: usize,
}

impl PhysicalAddress {
    pub fn new(addr: usize) -> PhysicalAddress {
        PhysicalAddress { addr: addr }
    }

    pub fn to_ppn(&self) -> usize {
        self.addr >> 12
    }

    pub fn to_ppn_array(&self) -> [usize; 3] {
        [
            (self.addr >> 12) & 0x1ff,
            (self.addr >> 21) & 0x1ff,
            (self.addr >> 30) & 0x3ff_ffff,
        ]
    }

    pub fn to_usize(&self) -> usize {
        self.addr
    }

    pub fn as_pointer(&self) -> *mut usize {
        self.addr as *mut usize
    }
}

// Page
/////

#[derive(Debug)]
pub struct Page {
    addr: PhysicalAddress,
}

impl Page {
    pub fn from_address(addr: PhysicalAddress) -> Page {
        Page { addr: addr }
    }

    pub fn address(&self) -> PhysicalAddress {
        self.addr
    }

    pub fn clear(&self) {
        unsafe {
            let ptr = self.addr.as_pointer();
            for i in 0..512 {
                ptr.add(i).write(0)
            }
        }
    }
}

// Page Allocator (soooo tiny version)
/////

static mut base_addr: usize = 0;
static mut last_index: usize = 0;
static mut initialized: bool = false;

pub fn init() {
    unsafe {
        base_addr = (elf_end() & !(0xfff as usize)) + 4096;
        last_index = 0;
        initialized = true;
    }
}

pub fn set_alloc_base(addr: usize) {
    unsafe {
        base_addr = addr;
    }
}

pub fn alloc() -> Page {
    // TODO: this unsafe block is evil!
    unsafe {
        if !initialized {
            panic!("page manager was used but not initialized");
        }

        last_index += 1;
        let addr = base_addr + (PAGE_SIZE as usize) * (last_index - 1);
        if addr > DRAM_END {
            panic!("memory exhausted; 0x{:016x}", addr)
        }
        let p = Page::from_address(PhysicalAddress::new(addr));
        p.clear();
        p
    }
}

pub fn alloc_16() -> Page {
    let mut root_page = alloc();
    while root_page.address().to_usize() & (0b11_1111_1111_1111 as usize) > 0 {
        log::debug!(
            "a page 0x{:016x} was allocated, but it does not follow 16KiB boundary. drop.",
            root_page.address().to_usize()
        );
        root_page = alloc();
    }
    alloc();
    alloc();
    alloc();
    root_page
}

pub fn alloc_continuous(num: usize) -> Page {
    if num <= 0 {
        panic!("invalid arg for alloc_contenious: {}", num);
    }

    let first = alloc();
    for _ in 0..(num - 1) {
        let _ = alloc();
    }

    first
}

// Page Table
/////

#[derive(Debug)]
struct PageTableEntry {
    pub ppn: [usize; 3],
    pub flags: u16,
}

pub enum PageTableEntryFlag {
    Valid = 1 << 0,
    Read = 1 << 1,
    Write = 1 << 2,
    Execute = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Access = 1 << 6,
    Dirty = 1 << 7,
    // TODO (enhancement): RSW
}

impl PageTableEntry {
    pub fn from_value(v: usize) -> PageTableEntry {
        let ppn = [(v >> 10) & 0x1ff, (v >> 19) & 0x1ff, (v >> 28) & 0x3ff_ffff];
        PageTableEntry {
            ppn: ppn,
            flags: (v & (0x1ff as usize)) as u16,
        }
    }

    pub unsafe fn from_memory(paddr: PhysicalAddress) -> PageTableEntry {
        let ptr = paddr.as_pointer();
        let entry = *ptr;
        PageTableEntry::from_value(entry)
    }

    pub fn to_usize(&self) -> usize {
        (if (self.ppn[2] >> 25) & 1 > 0 {
            0x3ff << 54
        } else {
            0
        }) | ((self.ppn[2] as usize) << 28)
            | ((self.ppn[1] as usize) << 19)
            | ((self.ppn[0] as usize) << 10)
            | (self.flags as usize)
    }

    pub fn next_page(&self) -> Page {
        Page::from_address(PhysicalAddress::new(
            (self.ppn[2] << 30) | (self.ppn[1] << 21) | (self.ppn[0] << 12),
        ))
    }

    pub fn set_flag(&mut self, flag: PageTableEntryFlag) {
        self.flags |= flag as u16;
    }

    pub fn is_valid(&self) -> bool {
        self.flags & (PageTableEntryFlag::Valid as u16) != 0
    }
}

pub struct PageTable {
    pub page: Page,
}

// TODO (enhancement): this naming is not so good.
// This implementation assumes the paging would be done with Sv39,
// but the word 'page table" is a more general idea.
// We can rename this like `Sv39PageTable` or change the implementation in a more polymorphic way.
impl PageTable {
    fn set_entry(&self, i: usize, entry: PageTableEntry) {
        let ptr = self.page.address().as_pointer() as *mut usize;
        unsafe { ptr.add(i).write(entry.to_usize()) }
    }

    fn get_entry(&self, i: usize) -> PageTableEntry {
        let ptr = self.page.address().as_pointer() as *mut usize;
        unsafe { PageTableEntry::from_value(ptr.add(i).read()) }
    }

    pub fn from_page(page: Page) -> PageTable {
        PageTable { page: page }
    }

    pub fn resolve(&self, vaddr: &VirtualAddress) -> PhysicalAddress {
        self.resolve_intl(vaddr, self, 2)
    }

    fn resolve_intl(
        &self,
        vaddr: &VirtualAddress,
        pt: &PageTable,
        level: usize,
    ) -> PhysicalAddress {
        let vpn = vaddr.to_vpn();

        let entry = pt.get_entry(vpn[level]);
        if !entry.is_valid() {
            panic!("failed to resolve vaddr: 0x{:016x}", vaddr.addr)
        }

        if level == 0 {
            let addr_base = entry.next_page().address().to_usize();
            PhysicalAddress::new(addr_base | vaddr.to_offset())
        } else {
            let next_page = entry.next_page();
            let new_pt = PageTable::from_page(next_page);
            self.resolve_intl(vaddr, &new_pt, level - 1)
        }
    }

    pub fn map(&self, vaddr: VirtualAddress, dest: &Page, perm: u16) {
        self.map_intl(vaddr, dest, self, perm, 2)
    }

    fn map_intl(
        &self,
        vaddr: VirtualAddress,
        dest: &Page,
        pt: &PageTable,
        perm: u16,
        level: usize,
    ) {
        let vpn = vaddr.to_vpn();

        if level == 0 {
            // register `dest`  addr
            let new_entry = PageTableEntry::from_value(
                ((dest.address().to_usize() as i64 >> 2) as usize)
                    | (PageTableEntryFlag::Valid as usize)
                    | (PageTableEntryFlag::Dirty as usize)
                    | (PageTableEntryFlag::Access as usize)
                    | (perm as usize),
            );
            pt.set_entry(vpn[0], new_entry);
        } else {
            // walk the page table
            let entry = pt.get_entry(vpn[level]);
            if !entry.is_valid() {
                // if no entry found, create new page and assign it.
                let new_page = alloc();
                let new_entry = PageTableEntry::from_value(
                    ((new_page.address().to_usize() as i64 >> 2) as usize)
                        | (PageTableEntryFlag::Valid as usize),
                );
                pt.set_entry(vpn[level], new_entry);
                let new_pt = PageTable::from_page(new_page);
                self.map_intl(vaddr, dest, &new_pt, perm, level - 1);
            } else {
                let next_page = entry.next_page();
                let new_pt = PageTable::from_page(next_page);
                self.map_intl(vaddr, dest, &new_pt, perm, level - 1);
            };
        }
    }
}
