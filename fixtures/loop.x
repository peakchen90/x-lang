fn a() {
    var i = 1;
    loop {
        if i == 2 {
            break;
        }
        i = i + 1
    }
}

fn b() {
    a: loop {
        b: loop {
           break a;
        }

        print(1);
    }
}

fn c() {
    a: loop {
        b: loop {
           return;
        }
        print(1);
    }
}

fn d() {
    var i = 1;
    a: loop {
        print(i);
        loop {
            print(true, i);
            if (i == 10) {
                break;
            }
            i = i + 1;
        }
        if (i == 20) {
            break;
        }
        i = i + 1;
    }
}

fn main() {
    a();
    b();
    c();
    // d();
}