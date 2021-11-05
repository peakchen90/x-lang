# xlang

基于 LLVM 与 rust 实现一门名叫 **x** 的语言，仅用于学习练手

## 教程

- [用LLVM开发新语言](https://llvm-tutorial-cn.readthedocs.io/en/latest/index.html)

## 语法

```
// 注释
fn add(a, b) {
    return a + b;
}

var num = 3;
var result = add(num, 4.2);

print(result); // 调用系统内置方法
```