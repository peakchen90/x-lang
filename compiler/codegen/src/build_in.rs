///! build-in impl
use crate::helper::never;

pub extern "C" fn system_print_newline() {
    println!();
}

pub extern "C" fn system_print_str(value: &str) {
    print!("{}", value);
}

pub extern "C" fn system_print_num(value: f64) {
    print!("{}", value);
}

pub extern "C" fn system_print_bool(value: u8) {
    print!(
        "{}",
        match value {
            1 => "true",
            0 => "false",
            _ => never(),
        }
    );
}
