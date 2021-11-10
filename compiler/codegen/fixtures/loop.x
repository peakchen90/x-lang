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

        print(1);
    }
}

b();

fn c() {
    a: loop {
        b: loop {
           return;
        }
        print(1);
    }
}

c();