
> rust grammar used in peaCore.

- The default executable file is src/main.rs.

    Other executables can be placed in src/bin/.
    
    see [rust package layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)

- #[repr(transparent)]: 保证内存分布和其单一成员相同
  类似的还有#[repr(C)], #[repr(u8)]

- [#macro_use]: use the macro declared in the following crate.

- [#link_section]: 

    指定了输出对象文件中函数或静态项的内容将被放置到的节点位置。

    ```rust
    #[no_mangle]
    #[link_section = ".example_section"]
    pub static VAR1: u32 = 1;
    ```
- [build scripts: build.rs](https://course.rs/cargo/reference/build-script/intro.html)

- #[feature(linkage)]: 规定了链接方式 (e.g. link=weak)

- RefCell: 

    From ChatGPT

    在 Rust 中，RefCell 是一种用于提供运行时借用检查的数据类型。它允许你在运行时动态地检查借用规则，以避免在编译时就阻止某些合法的操作。

    RefCell 是一个类似于 Mutex 或 RwLock 的类型，它允许在多个地方同时持有对同一个数据的可变或不可变引用。但是，与 Mutex 或 RwLock 不同，RefCell 的借用检查是在运行时进行的，而不是在编译时进行的。这意味着，使用 RefCell 时，你需要自己确保在任何时候都不会同时持有多个可变引用，否则会在运行时产生 panic。

    RefCell 主要有两个方法：borrow 和 borrow_mut。borrow 方法用于获取一个不可变引用，borrow_mut 方法用于获取一个可变引用。这些方法会在运行时检查是否违反了借用规则，如果检查失败，则会产生 panic。

    ```rust
    use std::cell::RefCell;

    fn main() {
        let x = RefCell::new(42);

        // Get an immutable reference to the value
        let r1 = x.borrow();
        println!("r1 = {}", *r1);

        // Get a mutable reference to the value
        let mut r2 = x.borrow_mut();
        *r2 += 1;
        println!("r2 = {}", *r2);

        // Attempt to get another mutable reference, which will panic
        // because r2 is still in scope
        // let mut r3 = x.borrow_mut();
    }

    ```


- Rc\<T>: requires copy trait

    ```rust
    use std::cell::Cell;
    fn main() {
        let c = Cell::new("asdf");
        let one = c.get();
        c.set("qwer");
        let two = c.get();
        println!("{},{}", one, two);
    }
    // ouput: asdf qwer
    ```


- `&` versus `ref`

    [rust forum](https://users.rust-lang.org/t/ref-keyword-versus/18818/3)

- #[repr(align(4))]: function align (inorder to be written into mtvec/stvec)