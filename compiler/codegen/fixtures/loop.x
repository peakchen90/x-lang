fn a() {
    loop {
        break;
    }
}

a();

fn b() {
    a: loop {
        b: loop {
           break a;
        }
    }
}

b();