define_read!(0x241);
define_write!(0x241);

pub fn set<T: crate::util::jump::Target>(t: &T) {
    write(t.convert_to_fn_address());
}
