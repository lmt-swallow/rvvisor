define_read!(0x603);
define_write!(0x603);

pub const VSEIP: usize = 1 << 10;
pub const VSTIP: usize = 1 << 6;
pub const VSSIP: usize = 1 << 2;
