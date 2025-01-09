//  ENUMS.rs
//    by Lut99
//
//  Created:
//    26 Dec 2024, 12:56:13
//  Last edited:
//    09 Jan 2025, 01:24:32
//  Auto updated?
//    Yes
//
//  Description:
//!   Shows that the crate works for enums.
//

use std::marker::PhantomData;

use better_derive::Debug;


/***** EXAMPLES *****/
/// Example empty struct as usual.
#[derive(Debug)]
#[allow(unused)]
enum Foo {}

/// Example tuple struct as usual.
#[derive(Debug)]
enum Bar {
    Variant1((), bool, String),
}

/// Example struct struct as usual.
#[derive(Debug)]
enum Baz {
    Variant1 { a: (), b: bool, c: String },
}



struct DontImplementDebug;

/// Special struct with generics that don't have to be debug.
#[derive(Debug)]
enum PhantomEnum<T> {
    Variant1 { _f: PhantomData<T> },
}





/***** ENTRYPOINT *****/
fn main() {
    // NOTE: Can't construct, of course
    // assert_eq!(format!("{:?}", Foo), ???);
    // assert_eq!(format!("{:#?}", Foo), ???);
    assert_eq!(format!("{:?}", Bar::Variant1((), true, "Hello, world!".into())), "Bar::Variant1((), true, \"Hello, world!\")");
    assert_eq!(format!("{:#?}", Bar::Variant1((), true, "Hello, world!".into())), "Bar::Variant1(\n    (),\n    true,\n    \"Hello, world!\",\n)");
    assert_eq!(
        format!("{:?}", Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() }),
        "Baz::Variant1 { a: (), b: true, c: \"Hello, world!\" }"
    );
    assert_eq!(
        format!("{:#?}", Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() }),
        "Baz::Variant1 {\n    a: (),\n    b: true,\n    c: \"Hello, world!\",\n}"
    );

    assert_eq!(
        format!("{:?}", PhantomEnum::<DontImplementDebug>::Variant1 { _f: PhantomData }),
        "PhantomEnum::Variant1 { _f: PhantomData<enums::DontImplementDebug> }"
    );
    assert_eq!(
        format!("{:#?}", PhantomEnum::<DontImplementDebug>::Variant1 { _f: PhantomData }),
        "PhantomEnum::Variant1 {\n    _f: PhantomData<enums::DontImplementDebug>,\n}"
    );
}
