use std::ffi::CStr;
use libc::c_char;

#[no_mangle]
pub extern "C" fn compile(input: *const c_char) {
    let str = unsafe {CStr::from_ptr(input)};
    let source = str.to_str().unwrap();
    x_lang_codegen::compile(source, false);
}


// #[no_mangle]
// pub extern "C" fn dx() -> *const c_char {
//     return "12232".as_ptr() as  *const c_char;
// }

// #[no_mangle]
// pub extern "C" fn dx(a: *const c_char) -> *const c_char {
//     let str = unsafe {CStr::from_ptr(a)};
//     let str = str.to_str().unwrap();
//     let mut new_str = String::from("HELLO: ");
//     new_str.push_str(str);
//     return new_str.as_ptr() as  *const c_char;
// }

// #[no_mangle]
// pub extern "C" fn dx(a: *const c_char) -> u32 {
//     let str = unsafe {CStr::from_ptr(a)};
//     let str = str.to_str().unwrap();
//     match str.as_bytes() {
//         b"abc" => 1,
//         _ => 2
//     }
// }

// #[no_mangle]
// pub extern "C" fn dx(a: *mut c_char) -> *const c_char {
//     let str = unsafe {CString::from_raw(a)};
//     let str = str.to_str().unwrap();
//     let mut new_str = String::from("HELLO: ");
//     new_str.push_str(str);
//     let res = CString::new(new_str.as_bytes()).unwrap();
//     res.into_raw()
// }

