define_read!(0x680);
define_write!(0x680);

#[derive(Copy, Clone)]
pub enum Mode {
    Bare = 0,
    Sv39x4 = 8,
    Sv48x4 = 9,
    Sv57x4 = 10,
}

pub fn set(s: &Setting) {
    write(s.to_usize());
}

pub struct Setting {
    pub mode: Mode,
    pub vmid: u16,
    pub ppn: usize,
}

impl Setting {
    pub fn new(mode: Mode, vmid: u16, ppn: usize) -> Setting {
        Setting {
            mode: mode,
            vmid: vmid,
            ppn: ppn,
        }
    }

    #[cfg(target_pointer_width = "64")]
    pub fn to_usize(&self) -> usize {
        let mut v: usize = 0;
        v |= (self.mode as usize) << 60;
        v |= (self.vmid as usize) << 44;
        v |= self.ppn;
        v
    }
}
