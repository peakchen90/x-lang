fn a() {
    var i = 1;
    loop {
        if i == 2 {
            break;
        }
        i = i + 1
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