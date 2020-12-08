use crate::memlayout;
use core::fmt::{Error, Write};

pub struct Uart {
	addr_base: *mut u8,
}

impl Write for Uart {
	fn write_str(&mut self, out: &str) -> Result<(), Error> {
		for c in out.bytes() {
			self.put(c);
		}
		Ok(())
	}
}

impl Uart {
	pub fn new(uart_base: usize) -> Self {
		let ptr = uart_base as *mut u8;
		Uart { addr_base: ptr }
	}

	fn thr(&mut self) -> *mut u8 {
		unsafe { self.addr_base.offset(0) }
	}

	fn rbr(&mut self) -> *mut u8 {
		unsafe { self.addr_base.offset(0) }
	}

	fn ier(&mut self) -> *mut u8 {
		unsafe { self.addr_base.offset(1) }
	}

	fn fcr(&mut self) -> *mut u8 {
		unsafe { self.addr_base.offset(2) }
	}

	fn lcr(&mut self) -> *mut u8 {
		unsafe { self.addr_base.offset(3) }
	}

	fn lsr(&mut self) -> *mut u8 {
		unsafe { self.addr_base.offset(5) }
	}

	pub fn init(&mut self) {
		unsafe {
			// enable interrupts
			self.ier().write_volatile(1 << 0);

			// enable FIFO
			self.fcr().write_volatile(1 << 0);

			// set WLS to 8 bits
			self.lcr().write_volatile((1 << 0) | (1 << 1));
		}
	}

	pub fn put(&mut self, c: u8) {
		unsafe {
			// spin until bit 5 of LSR holds
			while self.lsr().read_volatile() & (1 << 5) == 0 {}

			// add `c` to the FIFO
			self.thr().write_volatile(c);
		}
	}

	pub fn get(&mut self) -> Option<u8> {
		unsafe {
			// read LSR first in order to check whether read FIFO has any data or not
			if self.lsr().read_volatile() & (1 << 0) == 0 {
				None
			} else {
				Some(self.rbr().offset(0).read_volatile())
			}
		}
	}
}

#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
		use core::fmt::Write;
		let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
	});
}

#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

pub fn handle_interrupt() {
	let mut uart = Uart::new(memlayout::UART_BASE);
	if let Some(c) = uart.get() {
		// TODO (enhancement): pass it buffer or somewhere else

		// echo back
		match c {
			8 => {
				println!("{} {}", 8 as char, 8 as char);
			}
			10 | 13 => {
				println!();
			}
			_ => {
				print!("{}", c as char);
			}
		}
	}
}
