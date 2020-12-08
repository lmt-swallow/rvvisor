define_read!(0x141);
define_write!(0x141);

pub fn set<T: crate::util::jump::Target>(t: &T) {
    write(t.convert_to_fn_address());
}
