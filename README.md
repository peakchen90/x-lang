# x-lang

基于 LLVM 与 rust 实现一门名叫 **x** 的玩具语言，仅用于学习

## 基本语法

```
// 行注释
fn add(a: num, b: num) -> num {
    return a + b; // 返回语句
}

var n: num = 3;
var result = add(n, 4.2);

// 调用系统内置的打印方法，只接收一个数字类型参数，无返回值
print(result);
```

## 怎样跑起来
对于 **MacOS (x86-64)** 用户，可以直接下载编译好的二进制文件执行（[下载链接](https://github.com/peakchen90/x-lang/releases/tag/v0.0.1) ,下载 `x-lang` 二进制文件）。

对于其他系统用户，需要将本项目 clone 到本地编译：
- 需要依赖 LLVM 环境（[见说明](./compiler/codegen/README.md)）
- 需要 rust/cargo 环境
- 执行 `cargo build --release` 构建

下载或编译完成后，执行命令 `x-lang example.x` 编译并运行 x 语言（目前就实现了通过 JIT 方式运行），`example.x` 为待编译文件路径。
可以通过 `x-lang example.x --debug` 运行输出编译后端 LLVM-IR 码

## 语法注意事项
- x 是一门**强类型**语言，支持数字(`num`)、字符串(`str`)、布尔类型(`bool`)（目前仅支持数字类型）
- 通过 `var` 关键字声明变量，变量声明的类型**可以省略**，系统会自动推断类型
- 通过 `fn` 关键字声明函数，函数必须在最外层作用域定义，函数的参数类型及返回类型必须明确标识，**不可省略**。函数和变量都必须**先定义后使用**
- 函数调用时**必须**与函数定义的参数匹配，否则会调用失败
- 块级作用域隔离（还未实现，不支持访问/修改全局变量）
- 代码语句后**必须**以分号结尾，函数声明可以省略后面的分号

## 目前支持的能力
- 调用系统内置方法 `print(n)` 控制台打印信息
- 仅实现了数字(`num`) 类型，可以实现数字的加减乘除运算
- 变量/函数声明及访问/调用，变量的重新赋值操作


## 参数教程

- [LLVM Tutorial](https://releases.llvm.org/13.0.0/docs/tutorial/index.html)
- [LLVM Language Reference Manual](https://releases.llvm.org/13.0.0/docs/LangRef.html)
- [llvm.org](https://llvm.org/)
- [Getting Started with LLVM Core Libraries（中文版）](https://getting-started-with-llvm-core-libraries-zh-cn.readthedocs.io/zh_CN/latest/index.html)
- [用 LLVM 开发新语言](https://llvm-tutorial-cn.readthedocs.io/en/latest/index.html) (已过时)
- [LLVM 中文文档](https://llvm.liuxfe.com/)
- [LLVM IR入门指南](https://github.com/Evian-Zhang/llvm-ir-tutorial)

## 一些资源

- https://crates.io/crates/llvm-sys
- https://crates.io/crates/llvm-ir
- https://github.com/TheDan64/inkwell
