# 类

## 示例
```
class Point {
    pub name: str
    x: num
    y: num

    pub Point(x, y) {
        this.x = x;
        this.y = y;
    }

    pub getX() {
        return this.x;
    }

    pub getY() {
        return this.y;
    }
}

fn main() {
    var point = new Point(2, 4);
    point.name = "haha";
    point.getX(); // 2
}
```

**说明：**
- 通过 `class` 关键字定义类，类里面可以定义属性或方法
- 默认类里的属性或方法都是内部私有的，如果需要外部可访问，必须通过 `pub` 关键字修饰
- 与类名同名的方法（示例中的 `Point(x: num, y: num)` 方法）为类的构造函数，通过 `new` 实例化一个类时会调用
构造方法，构造方法必须通过 `pub` 修饰，并且不能定义返回类型，因为返回类本身的实例

## 继承
使用 `extends` 关键字继承
```
class Circle extends Point {
    radius: num

    pub Circle(radius: num, x: num, y: num) {
        super(x, y);
        this.radius = radius;
    }

    pub draw() {
        var x = this.getX();
        var y = this.getY();
        var radius = this.radius;
        // draw ...
    }
}
```