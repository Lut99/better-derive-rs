//  ENUMS.rs
//    by Lut99
//
//  Created:
//    26 Dec 2024, 12:56:13
//  Last edited:
//    04 Feb 2025, 16:31:45
//  Auto updated?
//    Yes
//
//  Description:
//!   Shows that the crate works for enums.
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
/// Example empty struct as usual.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[allow(unused)]
enum Foo {}

/// Example tuple struct as usual.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Bar {
    Variant1((), bool, String),
}

/// Example struct struct as usual.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Baz {
    Variant1 { a: (), b: bool, c: String },
}



struct DontImplementAnything;

/// Special struct with generics that don't have to be debug.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum PhantomEnum<T> {
    Variant1 { _f: PhantomData<T> },
}





/***** ENTRYPOINT *****/
fn main() {
    // NOTE: Can't construct of course
    // assert_eq!(Foo.clone(), Foo);
    assert_eq!(Bar::Variant1((), true, "Hello, world!".into()).clone(), Bar::Variant1((), true, "Hello, world!".into()));
    assert_eq!(Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() }.clone(), Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() });
    assert_eq!(PhantomEnum::<DontImplementAnything>::Variant1 { _f: PhantomData }.clone(), PhantomEnum::<DontImplementAnything>::Variant1 {
        _f: PhantomData,
    });



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
        format!("{:?}", PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData }),
        "PhantomEnum::Variant1 { _f: PhantomData<enums::DontImplementAnything> }"
    );
    assert_eq!(
        format!("{:#?}", PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData }),
        "PhantomEnum::Variant1 {\n    _f: PhantomData<enums::DontImplementAnything>,\n}"
    );



    // NOTE: Can't construct of course
    // assert!(Foo == Foo);
    assert!(Bar::Variant1((), true, "Hello, world!".into()) == Bar::Variant1((), true, "Hello, world!".into()));
    assert!(Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() } == Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() });
    assert!(PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData } == PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData });



    // NOTE: Can't construct of course
    // assert_eq!(hash(Foo), 0);
    assert_eq!(hash(Bar::Variant1((), true, "Hello, world!".into())), 17152124978856657821);
    assert_eq!(hash(Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() }), 17152124978856657821);
    assert_eq!(hash(PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData }), 13646096770106105413);
}
