# 字符串

基于 `array` 可变长度数组实现，字符编码使用 [UTF-16](https://datatracker.ietf.org/doc/html/rfc2781) 编码

使用字符串直接量表示时，实际会实例化一个 `String` 类，例如这种：`new String([104, 101, 108, 108, 111])`

## 示例

```
fn main() {
    var s1 = "hello";
    var s2: str = " world"
    print(s1 + s2); // "hello world"
}
```

## 编码存储

> UTF-16 编码规则：https://datatracker.ietf.org/doc/html/rfc2781#section-2.1
> <br/>
> UTF-16 解码规则：https://datatracker.ietf.org/doc/html/rfc2781#section-2.2


使用 `"hello"` 字符串举例：

首先会使用 UTF-16 编码字符串得到一个字符码点集合: `[104, 101, 108, 108, 111]`,
然后直接基于可变长数组（类型为16位无符号整数，仅内部用）存储到内存中

### 标准库
`String`
- `at(index: num) -> String`
- `substr(start: num, end: num) -> String`