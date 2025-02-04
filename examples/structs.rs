//  SIMPLE.rs
//    by Lut99
//
//  Created:
//    26 Dec 2024, 12:12:07
//  Last edited:
//    04 Feb 2025, 16:30:20
//  Auto updated?
//    Yes
//
//  Description:
//!   Shows that the crate works for structs.
//

use std::hash::{DefaultHasher, Hasher as _};
use std::marker::PhantomData;

use better_derive::{Clone, Debug, Eq, Hash, PartialEq};


/***** HELPER FUNCTIONS *****/
#[inline]
fn hash<T: std::hash::Hash>(obj: T) -> u64 {
    let mut state = DefaultHasher::default();
    obj.hash(&mut state);
    state.finish()
}





/***** EXAMPLES *****/
/// Example unit struct as usual.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Foo;

/// Example tuple struct as usual.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Bar((), bool, String);

/// Example struct struct as usual.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Baz {
    a: (),
    b: bool,
    c: String,
}



struct DontImplementAnything;

/// Special struct with generics that don't have to be debug.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PhantomStruct<T> {
    _f: PhantomData<T>,
}





/***** ENTRYPOINT *****/
fn main() {
    assert_eq!(Foo.clone(), Foo);
    assert_eq!(Bar((), true, "Hello, world!".into()).clone(), Bar((), true, "Hello, world!".into()));
    assert_eq!(Baz { a: (), b: true, c: "Hello, world!".into() }.clone(), Baz { a: (), b: true, c: "Hello, world!".into() });
    assert_eq!(PhantomStruct::<DontImplementAnything> { _f: PhantomData }.clone(), PhantomStruct::<DontImplementAnything> { _f: PhantomData });



    assert_eq!(format!("{:?}", Foo), "Foo");
    assert_eq!(format!("{:#?}", Foo), "Foo");
    assert_eq!(format!("{:?}", Bar((), true, "Hello, world!".into())), "Bar((), true, \"Hello, world!\")");
    assert_eq!(format!("{:#?}", Bar((), true, "Hello, world!".into())), "Bar(\n    (),\n    true,\n    \"Hello, world!\",\n)");
    assert_eq!(format!("{:?}", Baz { a: (), b: true, c: "Hello, world!".into() }), "Baz { a: (), b: true, c: \"Hello, world!\" }");
    assert_eq!(format!("{:#?}", Baz { a: (), b: true, c: "Hello, world!".into() }), "Baz {\n    a: (),\n    b: true,\n    c: \"Hello, world!\",\n}");

    assert_eq!(
        format!("{:?}", PhantomStruct::<DontImplementAnything> { _f: PhantomData }),
        "PhantomStruct { _f: PhantomData<structs::DontImplementAnything> }"
    );
    assert_eq!(
        format!("{:#?}", PhantomStruct::<DontImplementAnything> { _f: PhantomData }),
        "PhantomStruct {\n    _f: PhantomData<structs::DontImplementAnything>,\n}"
    );



    assert!(Foo == Foo);
    assert!(Bar((), true, "Hello, world!".into()) == Bar((), true, "Hello, world!".into()));
    assert!(Baz { a: (), b: true, c: "Hello, world!".into() } == Baz { a: (), b: true, c: "Hello, world!".into() });
    assert!(PhantomStruct::<DontImplementAnything> { _f: PhantomData } == PhantomStruct::<DontImplementAnything> { _f: PhantomData });



    assert_eq!(hash(Foo), 15130871412783076140);
    assert_eq!(hash(Bar((), true, "Hello, world!".into())), 13134715174715772495);
    assert_eq!(hash(Baz { a: (), b: true, c: "Hello, world!".into() }), 13134715174715772495);
    assert_eq!(hash(PhantomStruct::<DontImplementAnything> { _f: PhantomData }), 15130871412783076140);
}
