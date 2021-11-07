use libc::c_char;

pub fn to_char_ptr(str: &str) -> *const c_char {
    str.as_ptr() as *const c_char
}
