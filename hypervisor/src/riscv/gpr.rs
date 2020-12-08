// TODO (enhancement): this code is terrible.
// however, due to the restriction of llvm_asm macro and asm macro, it's hard to rewrite this cleanly :-(

extern crate num_enum;

use core::convert::TryFrom;
use core::marker::Copy;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(usize)]
#[derive(Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Register {
	Zero = 0,
	Ra,
	Sp,
	Gp,
	Tp,
	T0,
	T1,
	T2,
	S0,
	S1,
	A0,
	A1,
	A2,
	A3,
	A4,
	A5,
	A6,
	A7,
	S2,
	S3,
	S4,
	S5,
	S6,
	S7,
	S8,
	S9,
	S10,
	S11,
	T3,
	T4,
	T5,
	T6,
}

impl core::fmt::Display for Register {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let s = match *self {
			Register::Zero => "x0",
			Register::Ra => "ra",
			Register::Sp => "sp",
			Register::Gp => "gp",
			Register::Tp => "tp",
			Register::T0 => "t0",
			Register::T1 => "t1",
			Register::T2 => "t2",
			Register::S0 => "s0",
			Register::S1 => "s1",
			Register::A0 => "a0",
			Register::A1 => "a1",
			Register::A2 => "a2",
			Register::A3 => "a3",
			Register::A4 => "a4",
			Register::A5 => "a5",
			Register::A6 => "a6",
			Register::A7 => "a7",
			Register::S2 => "s2",
			Register::S3 => "s3",
			Register::S4 => "s4",
			Register::S5 => "s5",
			Register::S6 => "s6",
			Register::S7 => "s8",
			Register::S8 => "s8",
			Register::S9 => "s9",
			Register::S10 => "s10",
			Register::S11 => "s11",
			Register::T3 => "t3",
			Register::T4 => "t4",
			Register::T5 => "t5",
			Register::T6 => "t6",
		};
		f.pad(s)
	}
}

impl Register {
	fn read(&self) -> usize {
		unsafe {
			let rval;
			match *self {
				Register::Zero => asm!("addi {}, zero, 0", out(reg) rval),
				Register::Ra => asm!("addi {}, ra, 0", out(reg) rval),
				Register::Sp => asm!("addi {}, sp, 0", out(reg) rval),
				Register::Gp => asm!("addi {}, gp, 0", out(reg) rval),
				Register::Tp => asm!("addi {}, tp, 0", out(reg) rval),
				Register::T0 => asm!("addi {}, t0, 0", out(reg) rval),
				Register::T1 => asm!("addi {}, t1, 0", out(reg) rval),
				Register::T2 => asm!("addi {}, t2, 0", out(reg) rval),
				Register::S0 => asm!("addi {}, s0, 0", out(reg) rval),
				Register::S1 => asm!("addi {}, s1, 0", out(reg) rval),
				Register::A0 => asm!("addi {}, a0, 0", out(reg) rval),
				Register::A1 => asm!("addi {}, a1, 0", out(reg) rval),
				Register::A2 => asm!("addi {}, a2, 0", out(reg) rval),
				Register::A3 => asm!("addi {}, a3, 0", out(reg) rval),
				Register::A4 => asm!("addi {}, a4, 0", out(reg) rval),
				Register::A5 => asm!("addi {}, a5, 0", out(reg) rval),
				Register::A6 => asm!("addi {}, a6, 0", out(reg) rval),
				Register::A7 => asm!("addi {}, a7, 0", out(reg) rval),
				Register::S2 => asm!("addi {}, s2, 0", out(reg) rval),
				Register::S3 => asm!("addi {}, s3, 0", out(reg) rval),
				Register::S4 => asm!("addi {}, s4, 0", out(reg) rval),
				Register::S5 => asm!("addi {}, s5, 0", out(reg) rval),
				Register::S6 => asm!("addi {}, s6, 0", out(reg) rval),
				Register::S7 => asm!("addi {}, s7, 0", out(reg) rval),
				Register::S8 => asm!("addi {}, s8, 0", out(reg) rval),
				Register::S9 => asm!("addi {}, s9, 0", out(reg) rval),
				Register::S10 => asm!("addi {}, s10, 0", out(reg) rval),
				Register::S11 => asm!("addi {}, s11, 0", out(reg) rval),
				Register::T3 => asm!("addi {}, t3, 0", out(reg) rval),
				Register::T4 => asm!("addi {}, t4, 0", out(reg) rval),
				Register::T5 => asm!("addi {}, t5, 0", out(reg) rval),
				Register::T6 => asm!("addi {}, t6, 0", out(reg) rval),
			};
			rval
		}
	}
}

pub fn dump() {
	for i in 0..32 {
		let reg = Register::try_from(i).unwrap();
		print!("{:<3} = 0x{:016x} ", reg, reg.read());
		if i % 4 == 3 {
			println!();
		} else {
			print!("| ")
		}
	}
}
