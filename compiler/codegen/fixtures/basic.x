// comment
fn a(b :num, c :bool) -> num {
    if (c) {
        return b; // comment
    }
    return b + 1;
}

a(0.5, false);
