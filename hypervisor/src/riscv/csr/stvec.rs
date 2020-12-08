define_read!(0x105);
define_write!(0x105);

pub fn set<T: crate::util::jump::Target>(t: &T) {
    write(t.convert_to_fn_address());
}
