# 变量与可变性

用 `let` 关键字声明变量。

在 Rust 中，变量分为两种：可变变量和不可变变量。
你可以在变量名称之前增加 `mut` 关键字，表明这个变量是可变的。

当一个值和一个**不可变变量**绑定后，就不能再修改这个变量的值；
当一个值和一个*可变变量*绑定后，可以在后续修改这个变量的值。

Rust 中可以声明未初始化的变量，但必须显式标注类型，且在初始化之前禁止使用这个变量。

```rust
// 这是一个不可变变量
let a = 0;

a = 1; // 为不可变变量赋值会引发编译错误

// 这是一个可变变量
let mut b = 0;

b = 2; // 可以修改变量b的值

// 可以声明一个未初始化的变量
let c: i32;

// println!("{}", c);
// ^ 但使用未初始化的变量会出错

c = 3;
println!("{}", c); // 这样就没问题了！
```

> 译注：声明一个不可变变量之后，Rust 会保证这个变量的值在整个程序运行期间都不会发生改变。
> 这一点一开始或许有点烦人，不过却能为检查程序的行为提供了便利，在 Debug 时，你只需跟踪
> 那些可变变量，能够减少一部分心智负担。

## 参见

- 变量与可变性 [中文版](https://kaisery.github.io/trpl-zh-cn/ch03-01-variables-and-mutability.html) [英文原版](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
- 变量绑定 [中文版](https://rustwiki.org/zh-CN/rust-by-example/variable_bindings.html) [英文原版](https://doc.rust-lang.org/rust-by-example/variable_bindings.html)
