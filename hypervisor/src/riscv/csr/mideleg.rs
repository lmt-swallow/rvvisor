define_read!(0x303);
define_write!(0x303);

pub const SEIP: usize = 1 << 9;
pub const STIP: usize = 1 << 5;
pub const SSIP: usize = 1 << 1;
