define_read!(0x180);
define_write!(0x180);

#[derive(Copy, Clone)]
pub enum Mode {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9,
}

pub fn set(s: Setting) {
    write(s.to_usize());
}

pub struct Setting {
    mode: Mode,
    asid: u16,
    ppn: usize,
}

impl Setting {
    pub fn new(mode: Mode, asid: u16, ppn: usize) -> Setting {
        Setting {
            mode: mode,
            asid: asid,
            ppn: ppn,
        }
    }

    #[cfg(target_pointer_width = "64")]
    pub fn to_usize(&self) -> usize {
        let mut v: usize = 0;
        v |= (self.mode as usize) << 60;
        v |= (self.asid as usize) << 44;
        v |= self.ppn;
        v
    }
}
