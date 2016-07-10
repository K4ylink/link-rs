# Link
Link, an experimental safe anti-ownership relation between Rust objects.

## The Idea
Normally, rust advocates a ownership model, in which a strict high-level to
low-level relationship between objects is decided at design time.

However, sometimes you may want to do it another way, in which you can defer
the ownership relation decision to a later stage.

Using `Link`, you are able to do so directly.

However, there are three points to remember before you start to use `Link`:

1. You'll need to always refer those objects as Rc\<RefCell\<T\>\>,
in this way, safety is guaranteed.

2. You HAVE TO disconnect all `Link`s  before those objects went out of
scope. You can, however, disconnect from either side.

3. In the mutable borrow case, you can't trip around a `Link`. That will fail
because in Rust you can't borrow a thing mutable twice.

## How to use

Add this to your Cargo.toml:
```toml
[dependencies]
field-offset = "0.1"
link = "0.1"
```

To use it:

Declare the structs like:
```Rust
struct A {
    pub data: u32,
    pub link1: Link<A, B>,
}
struct B {
    pub data: String,
    pub link2: Link<B, A>,
}
```
import the `offset_of!` macro from the "field-offset" crate:

```Rust
#[macro_use]
extern crate field_offset as offset;

let mut a = Rc::new(RefCell::new(A {
            data: 42,
            link1: Link::new(),
        }));

let mut b = Rc::new(RefCell::new(B {
               data: "hello".to_owned(),
               link2: Link::new(),
           }));
Link::connect(&mut a, offset_of!{A => link1}, &mut b, offset_of!{B => link2});

println!("{}", a.borrow().link1.remote_owner().unwrap().data) // hello
println!("{}", b.borrow().link2.remote_owner().unwrap().data) // 42

```
You can also use `remote_owner_mut()` to borrow the objects on the other side
 of the link mutably.

## References
* The `field-offset` crate is used along with Link.
* The idea is also inspired by the `lia` language.