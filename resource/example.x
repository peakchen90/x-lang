fn add(a: num, b: num) -> num {
    return a + b; // 返回语句
}

// main 函数，程序的入口
fn main() {
    var n: num = 3;
    var str = "result: "
    var result = add(n, 4.2);

    // 调用系统内置的打印方法
    print(str, result);
}
