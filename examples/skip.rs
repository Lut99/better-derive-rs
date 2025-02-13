//  SKIP.rs
//    by Lut99
//
//  Created:
//    05 Feb 2025, 15:43:34
//  Last edited:
//    13 Feb 2025, 15:30:38
//  Auto updated?
//    Yes
//
//  Description:
//!   Showcases the usage of the `#[debug(skip)]`-field.
//

#![allow(unused)]

use std::cmp::Ordering;
use std::hash::{DefaultHasher, Hasher as _};

use better_derive::{Debug, Hash, PartialEq, PartialOrd};


#[inline]
fn hash<T: std::hash::Hash>(obj: T) -> u64 {
    let mut state = DefaultHasher::default();
    obj.hash(&mut state);
    state.finish()
}



// If you use `cargo expand --example skip`, you can see the bounds on `u32` aren't even derived!
#[derive(Debug, Hash, PartialEq, PartialOrd)]
pub struct Foo {
    // This is interesting!
    bar: String,
    // This isn't, really.
    #[debug(skip)]
    #[hash(skip)]
    #[partial_eq(skip)]
    #[partial_ord(skip)]
    baz: u32,
    // Note: you can also do this!
    #[better_derive(skip)]
    quz: u32,
}



fn main() {
    let foo1 = Foo { bar: "Hello, world!".into(), baz: 42, quz: 420 };
    let foo2 = Foo { bar: "Hello, world!".into(), baz: 43, quz: 421 };
    assert_eq!(format!("{:?}", foo1), "Foo { bar: \"Hello, world!\" }");
    assert!(foo1 == foo2);
    assert_eq!(foo1.partial_cmp(&foo2), Some(Ordering::Equal));
    assert!(hash(foo1) == hash(foo2));
}
