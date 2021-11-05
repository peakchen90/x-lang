# xlang

基于 LLVM 与 rust 实现一门名叫 **x** 的玩具语言，仅用于学习

## 教程

- [用LLVM开发新语言](https://llvm-tutorial-cn.readthedocs.io/en/latest/index.html)
- [A Tour to LLVM IR（上）](https://zhuanlan.zhihu.com/p/66793637)
- [A Tour to LLVM IR（下）](https://zhuanlan.zhihu.com/p/66909226)

## 资源
- https://crates.io/crates/llvm-sys
- https://crates.io/crates/llvm-ir
- https://github.com/TheDan64/inkwell
- https://llvm.org/

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