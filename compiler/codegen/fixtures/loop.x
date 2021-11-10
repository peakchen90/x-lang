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

        print(); // TODO: 这里有问题，不应该被执行
    }
}

b();