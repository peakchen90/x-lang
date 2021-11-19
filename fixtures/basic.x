// comment
fn a(b :num, c :bool) -> num {
    if (c) {
        return b; // comment
    }
    return b + 1;
}

fn main() {
    a(0.5, false);
}
