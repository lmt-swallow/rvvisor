use crate::memlayout;
use crate::paging;
use crate::riscv;
use core::mem::size_of;

pub const SECTOR_SIZE: usize = 512;

const VIRTIO_DESC_F_NEXT: u16 = 1 << 0;
const VIRTIO_DESC_F_WRITE: u16 = 1 << 1;

const VIRTIO_MAGIC: u32 = 0x74_72_69_76;
const VIRTIO_VENDOR: u32 = 0x55_4d_45_51;

pub const VIRTIO_BLK_F_RO: u32 = 1 << 5;
pub const VIRTIO_BLK_F_SCSI: u32 = 1 << 7;
pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 1 << 11;
pub const VIRTIO_BLK_F_MQ: u32 = 1 << 12;
pub const VIRTIO_F_ANY_LAYOUT: u32 = 1 << 27;
pub const VIRTIO_RING_F_INDIRECT_DESC: u32 = 1 << 28;
pub const VIRTIO_RING_F_EVENT_IDX: u32 = 1 << 29;

const VIRTIO_RING_SIZE: usize = 8;

#[repr(usize)]
#[derive(Copy, Clone)]
enum Offset {
	MagicValue = 0x000,
	Version = 0x004,
	DeviceId = 0x008,
	VendorId = 0x00c,
	HostFeatures = 0x010,
	HostFeaturesSel = 0x014,
	GuestFeatures = 0x020,
	GuestFeaturesSel = 0x024,
	GuestPageSize = 0x028,
	QueueSel = 0x030,
	QueueNumMax = 0x034,
	QueueNum = 0x038,
	QueueAlign = 0x03c,
	QueuePfn = 0x040,
	QueueNotify = 0x050,
	InterruptStatus = 0x060,
	InterruptAck = 0x064,
	Status = 0x070,
	Config = 0x100,
}

impl Offset {
	fn apply(&self, base: &*mut u32) -> *mut u32 {
		unsafe { base.offset((*self as isize) / 4) }
	}
}

enum StatusFlag {
	Acknowledge = 1,
	Driver = 2,
	Failed = 128,
	FeaturesOk = 8,
	DriverOk = 4,
	DeviceNeedsReset = 64,
}

#[repr(C)]
pub struct Descriptor {
	addr: u64,
	len: u32,
	flags: u16,
	next: u16,
}

#[repr(C)]
pub struct Available {
	flags: u16,
	idx: u16,
	ring: [u16; VIRTIO_RING_SIZE],
	event: u16,
}

#[repr(C)]
pub struct UsedElem {
	id: u32,
	len: u32,
}

#[repr(C)]
pub struct Used {
	flags: u16,
	idx: u16,
	ring: [UsedElem; VIRTIO_RING_SIZE],
	event: u16,
}

#[repr(C)]
pub struct VInfo {
	pub data: *mut u8,
	pub status: u8,
}

#[repr(C)]
pub struct BlkOuthdr {
	pub typ: u32,
	pub reserved: u32,
	pub sector: u64,
}

#[repr(C)]
pub struct Queue {
	pub desc: [Descriptor; VIRTIO_RING_SIZE],
	pub avail: Available,
	pub padding0: [u8; (memlayout::PAGE_SIZE as usize)
		- size_of::<Descriptor>() * VIRTIO_RING_SIZE
		- size_of::<Available>()],
	pub used: Used,

	pub used_idx: u16,
	pub vinfo: [VInfo; VIRTIO_RING_SIZE],
	pub header: [BlkOuthdr; VIRTIO_RING_SIZE],

	pub notify_slot: [bool; VIRTIO_RING_SIZE],
	pub device_base_addr: *mut u32,
}

pub static mut QUEUE: Option<*mut Queue> = None;
static mut INITIALIZED: bool = false;

impl Queue {
	pub fn from_page(p: paging::Page) -> *mut Queue {
		unsafe {
			let queue = p.address().to_usize() as *mut Queue;
			(*queue).used_idx = 0;
			queue
		}
	}

	fn request(&mut self, sector: u64, buf_addr: *const (), is_write: bool) -> usize {
		unsafe {
			// TODO: select unused descriptors
			let idx = [0, 1, 2];

			self.header[idx[0] as usize].typ = if is_write { 1 } else { 0 };
			self.header[idx[0] as usize].reserved = 0;
			self.header[idx[0] as usize].sector = sector;

			self.desc[idx[0]].addr = &self.header[idx[0] as usize] as *const BlkOuthdr as u64;
			self.desc[idx[0]].len = size_of::<BlkOuthdr>() as u32;
			self.desc[idx[0]].flags = VIRTIO_DESC_F_NEXT;
			self.desc[idx[0]].next = idx[1] as u16;
			self.desc[idx[1]].addr = buf_addr as u64;
			self.desc[idx[1]].len = SECTOR_SIZE as u32;
			self.desc[idx[1]].flags =
				VIRTIO_DESC_F_NEXT | (if !is_write { VIRTIO_DESC_F_WRITE } else { 0 });
			self.desc[idx[1]].next = idx[2] as u16;

			self.vinfo[idx[0] as usize].status = 0xff;
			self.desc[idx[2]].addr = &self.vinfo[idx[0] as usize].status as *const u8 as u64;
			self.desc[idx[2]].len = 1;
			self.desc[idx[2]].flags = VIRTIO_DESC_F_WRITE;
			self.desc[idx[2]].next = 0 as u16;

			self.avail.ring[self.avail.idx as usize % VIRTIO_RING_SIZE] = idx[0] as u16;
			self.avail.idx += 1;
			self.notify_slot[idx[0]] = false;
			Offset::QueueNotify
				.apply(&self.device_base_addr)
				.write_volatile(0);
			idx[0]
		}
	}

	pub fn read(&mut self, sector: u64, buf_addr: *const ()) {
		let idx = self.request(sector, buf_addr, false);
		log::debug!("request was sent. watching id: {}", idx);

		// TODO (enhancement): this spin lock is too heavy; we can do better
		unsafe { while !(&self.notify_slot[idx] as *const bool).read_volatile() {} }

		log::debug!("request was handled: {}", idx);
	}

	pub fn write(&mut self, sector: u64, buf_addr: *const ()) {
		let idx = self.request(sector, buf_addr, true);
		log::debug!("request was sent. watching id: {}", idx);

		// TODO (enhancement): this spin lock is too heavy; we can do better
		unsafe { while !(&self.notify_slot[idx] as *const bool).read_volatile() {} }

		log::debug!("request was handled: {}", idx);
	}

	pub fn mark_finished(&mut self, id: usize) {
		self.notify_slot[id] = true;
	}
}

pub fn init() {
	// log::info!("virtio0 addr: 0x{:016x}", memlayout::VIRTIO0_BASE);
	let base = memlayout::VIRTIO0_BASE as *mut u32;
	assert_device_status(&base);
	assert_device_type(&base, 2);
	log::info!("a block device found");

	let queue_page = paging::alloc_continuous(2);
	log::info!(
		"-> allocated query object: 0x{:016x}",
		queue_page.address().to_usize()
	);

	let queue = Queue::from_page(queue_page);
	unsafe {
		// TODO (enhancement): support multi core
		(*queue).device_base_addr = base;
		QUEUE = Some(queue);
		INITIALIZED = true;
	}
	init_block_device(&base, queue);
}

fn init_block_device(base: &*mut u32, queue: *mut Queue) {
	let mut status: u32 = 0;

	unsafe {
		let status_addr = Offset::Status.apply(base);

		// start to config
		status |= StatusFlag::Acknowledge as u32;
		status_addr.write_volatile(status);

		status |= StatusFlag::Driver as u32;
		status_addr.write_volatile(status);

		// set features
		let mut features: u32 = Offset::HostFeatures.apply(base).read_volatile();
		features &= !(VIRTIO_BLK_F_RO as u32);
		features &= !(VIRTIO_BLK_F_SCSI as u32);
		features &= !(VIRTIO_BLK_F_CONFIG_WCE as u32);
		features &= !(VIRTIO_BLK_F_MQ as u32);
		features &= !(VIRTIO_F_ANY_LAYOUT as u32);
		features &= !(VIRTIO_RING_F_EVENT_IDX as u32);
		features &= !(VIRTIO_RING_F_INDIRECT_DESC as u32);
		Offset::HostFeatures.apply(base).write_volatile(features);

		// finish feature configuration
		status |= StatusFlag::FeaturesOk as u32;
		status_addr.write_volatile(status);

		// finish configuration
		status |= StatusFlag::DriverOk as u32;
		status_addr.write_volatile(status);

		// tell our page size to virtio
		Offset::GuestPageSize
			.apply(base)
			.write_volatile(memlayout::PAGE_SIZE as u32);

		// set first queue selector
		Offset::QueueSel.apply(base).write_volatile(0);

		// set our queue num
		let queue_num = VIRTIO_RING_SIZE as u32;
		let queue_max = Offset::QueueNumMax.apply(base).read_volatile();
		if queue_num > queue_max {
			panic!("virtio disk has invalid queue max setting");
		}
		Offset::QueueNum.apply(base).write_volatile(queue_num);

		// set our queue addr
		Offset::QueuePfn
			.apply(base)
			.write_volatile(((queue as usize) >> 12) as u32);
	}
}

fn assert_device_status(base: &*mut u32) {
	unsafe {
		let magic = base.read_volatile();
		if magic != VIRTIO_MAGIC {
			panic!("invalid magic number found: {}", magic);
		}
		let version = base.offset(1).read_volatile();
		if version != 1 {
			panic!("invalid version: {}", version);
		}

		let vendor_id = base.offset(3).read_volatile();
		if vendor_id != VIRTIO_VENDOR {
			panic!("invalid vendor id: {}", vendor_id);
		}
	}
}

fn assert_device_type(base: &*mut u32, t: u32) {
	unsafe {
		let device_id = base.offset(2).read_volatile();
		if device_id != t {
			panic!("invalid device id: {} (expected: {})", device_id, t);
		}
	}
}

pub fn handle_interrupt(interrupt: u32) {
	let device_id = interrupt as usize - 1;
	if device_id == 0 {
		unsafe {
			// TODO (enhancement): notify related contes here
			if let Some(_queue) = QUEUE {
				while ((*_queue).used_idx as usize % VIRTIO_RING_SIZE)
					!= ((*_queue).used.idx as usize % VIRTIO_RING_SIZE)
				{
					let used_elem =
						&(*_queue).used.ring[(*_queue).used_idx as usize % VIRTIO_RING_SIZE];
					let finished_id = used_elem.id;
					log::debug!("used_elem: id={}, len={}", used_elem.id, used_elem.len);
					(*_queue).mark_finished(finished_id as usize);
					(*_queue).used_idx += 1;
				}
			} else {
				panic!("virtio queue uninitialized")
			}
		}
	} else {
		panic!("invalid interrupt from virtio0: {}", interrupt);
	}
}
