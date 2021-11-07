# x-lang

基于 LLVM 与 rust 实现一门名叫 **x** 的玩具语言，仅用于学习

## 教程

- [LLVM Tutorial](https://releases.llvm.org/13.0.0/docs/tutorial/index.html)
- [LLVM Language Reference Manual](https://releases.llvm.org/13.0.0/docs/LangRef.html)
- [llvm.org](https://llvm.org/)
- [Getting Started with LLVM Core Libraries（中文版）](https://getting-started-with-llvm-core-libraries-zh-cn.readthedocs.io/zh_CN/latest/index.html)
- [用 LLVM 开发新语言](https://llvm-tutorial-cn.readthedocs.io/en/latest/index.html) (已过时)
- [LLVM 中文文档](https://llvm.liuxfe.com/)
- [LLVM IR入门指南](https://github.com/Evian-Zhang/llvm-ir-tutorial)

## 资源

- https://crates.io/crates/llvm-sys
- https://crates.io/crates/llvm-ir
- https://github.com/TheDan64/inkwell

## 语法

```
// 注释
fn add(a: num, b: num) -> num {
    return a + b;
}

var n: num = 3;
var result = add(n, 4.2);

print(result); // 调用系统内置方法
```