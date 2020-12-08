pub trait Target {
    fn convert_to_fn_address(&self) -> usize;
}

impl Target for usize {
    fn convert_to_fn_address(&self) -> usize {
        return *self;
    }
}

impl Target for fn() {
    fn convert_to_fn_address(&self) -> usize {
        return (*self) as *const () as usize;
    }
}

impl Target for unsafe extern "C" fn() {
    fn convert_to_fn_address(&self) -> usize {
        return (*self) as *const () as usize;
    }
}
