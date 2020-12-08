define_read!(0x341);
define_write!(0x341);

pub fn set<T: crate::util::jump::Target>(t: T) {
    write(t.convert_to_fn_address());
}
