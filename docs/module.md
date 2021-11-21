# 模块设计

基本设计理念，每个模块（文件）内定义的方法、常量等都是隔离的，需要通过 `import` 关键字导入其他模块暴露出的方法、常量等

## 示例

**file: src/foo.x**
```
pub fn say(n: num) {
    ddd();
    print(n);
}

// 模块内的私有方法，不能在模块外使用
fn ddd() {
}
```

**file: src/main.x**
```
import foo.{say} // 导入 foo 模块的 say 方法

fn main() {
    var a = 3;
    say(n);
}
```

## 深入 `import` 关键字

### 导入其他目录的模块
- `import mod/b.{x}`
- `import ../mod/b.{y}`
- `import /absolute/mod/b.{z}`

### 同时导入多个方法
- `import mod.{x,y,z}`
- `import mod.{*}`: 导入 mod 模块内所有被 `pub` 修饰的方法
- `import mod` : 与 `use mod.{*}` 含义相同

### 重命名
- `import mod.{x,y as foo}`
- `import mod.{*, x as foo}`: 导入 mod 模块内所有被 `pub` 修饰的方法。其中：`x` 被重命名为 `foo`

### 导入内置标准库模块
> 相比自定义模块，导入内置标准库模块需要使用 `<...>` 包裹模块路径，其他用法相同
- `import <std/string>.{x,y,z}`
- `import <std/string>.{*}`
- `import <std/string>`