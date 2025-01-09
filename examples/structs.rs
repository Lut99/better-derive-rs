//  SIMPLE.rs
//    by Lut99
//
//  Created:
//    26 Dec 2024, 12:12:07
//  Last edited:
//    09 Jan 2025, 01:24:37
//  Auto updated?
//    Yes
//
//  Description:
//!   Shows that the crate works for structs.
//

use std::marker::PhantomData;

use better_derive::Debug;


/***** EXAMPLES *****/
/// Example unit struct as usual.
#[derive(Debug)]
struct Foo;

/// Example tuple struct as usual.
#[derive(Debug)]
struct Bar((), bool, String);

/// Example struct struct as usual.
#[derive(Debug)]
struct Baz {
    a: (),
    b: bool,
    c: String,
}



struct DontImplementDebug;

/// Special struct with generics that don't have to be debug.
#[derive(Debug)]
struct PhantomStruct<T> {
    _f: PhantomData<T>,
}





/***** ENTRYPOINT *****/
fn main() {
    assert_eq!(format!("{:?}", Foo), "Foo");
    assert_eq!(format!("{:#?}", Foo), "Foo");
    assert_eq!(format!("{:?}", Bar((), true, "Hello, world!".into())), "Bar((), true, \"Hello, world!\")");
    assert_eq!(format!("{:#?}", Bar((), true, "Hello, world!".into())), "Bar(\n    (),\n    true,\n    \"Hello, world!\",\n)");
    assert_eq!(format!("{:?}", Baz { a: (), b: true, c: "Hello, world!".into() }), "Baz { a: (), b: true, c: \"Hello, world!\" }");
    assert_eq!(format!("{:#?}", Baz { a: (), b: true, c: "Hello, world!".into() }), "Baz {\n    a: (),\n    b: true,\n    c: \"Hello, world!\",\n}");

    assert_eq!(
        format!("{:?}", PhantomStruct::<DontImplementDebug> { _f: PhantomData }),
        "PhantomStruct { _f: PhantomData<structs::DontImplementDebug> }"
    );
    assert_eq!(
        format!("{:#?}", PhantomStruct::<DontImplementDebug> { _f: PhantomData }),
        "PhantomStruct {\n    _f: PhantomData<structs::DontImplementDebug>,\n}"
    );
}
