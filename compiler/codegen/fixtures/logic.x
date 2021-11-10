fn a(a: bool) {
    var b = 0;
    if a {
        b = 1
    } else if true {
        b = 2

        if a {
            b = 21
            return;
        } else {
            b = 22
        }
    } else if false {
        b = 3
    } else {
        b = 4
    }
}

a(true);

fn b(a: bool) -> num {
    if a {
        return 1;
    } else if a {
        return 2;
    } else {
        return 3;
    }
}

b(true);