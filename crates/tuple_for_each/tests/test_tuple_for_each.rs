use tuple_for_each::*;

trait Base {
    type Item;
    fn foo(&self) -> Self::Item;
    fn bar(&self);
    fn bar_mut(&mut self) {}
}

struct A;
struct B;
struct C;

impl Base for A {
    type Item = u32;
    fn foo(&self) -> Self::Item {
        1
    }
    fn bar(&self) {
        println!("I'am A")
    }
}

impl Base for B {
    type Item = f32;
    fn foo(&self) -> Self::Item {
        2.333
    }
    fn bar(&self) {
        println!("I'am B")
    }
}

impl Base for C {
    type Item = &'static str;
    fn foo(&self) -> Self::Item {
        "hello"
    }
    fn bar(&self) {
        println!("I'am C")
    }
}

#[derive(TupleForEach)]
struct Pair(A, B);

#[derive(TupleForEach)]
struct Tuple(A, B, C);

#[test]
fn test_for_each() {
    let t = Pair(A, B);
    assert_eq!(t.len(), 2);

    let mut i = 0;
    pair_for_each!(x in t {
        println!("for_each {}: {}", i, x.foo());
        x.bar();
        i += 1;
    });
    assert_eq!(i, 2);
}

#[test]
fn test_for_each_mut() {
    let mut t = Tuple(A, B, C);
    assert_eq!(t.len(), 3);

    let mut i = 0;
    tuple_for_each!(x in mut t {
        println!("for_each_mut {}: {}", i, x.foo());
        x.bar_mut();
        i += 1;
    });
    assert_eq!(i, 3);
}

#[test]
fn test_enumerate() {
    let t = Tuple(A, B, C);
    assert_eq!(t.len(), 3);

    let mut real_idx = 0;
    tuple_enumerate!((i, x) in t {
        println!("enumerate {}: {}", i, x.foo());
        x.bar();
        assert_eq!(i, real_idx);
        real_idx += 1;
    });
    assert_eq!(real_idx, 3);
}

#[test]
fn test_enumerate_mut() {
    let mut t = Pair(A, B);
    assert_eq!(t.len(), 2);

    let mut real_idx = 0;
    pair_enumerate!((i, x) in mut t {
        println!("enumerate_mut {}: {}", i, x.foo());
        x.bar_mut();
        assert_eq!(i, real_idx);
        real_idx += 1;
    });
    assert_eq!(real_idx, 2);
}
