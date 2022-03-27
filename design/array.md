# 数组

相同类型可变长数组

使用数组直接量表示时，实际会实例化一个 `Array` 类，例如这种：`new Array([1, 2, 3])`

## 示例
```
fn main() {
    var a = [1, 2, 3];
    var b: bool[] = [false, true];
}
```

## 标准库
`Array`
- `push(val: T)`
- `pop(val: T)`
- `get(index: num) -> T`