//  ENUMS.rs
//    by Lut99
//
//  Created:
//    26 Dec 2024, 12:56:13
//  Last edited:
//    13 Feb 2025, 15:29:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Shows that the crate works for enums.
//

use std::cmp::Ordering;
use std::hash::{DefaultHasher, Hasher as _};
use std::marker::PhantomData;

#[cfg(feature = "serde")]
use better_derive::Serialize;
use better_derive::{Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd};


/***** HELPER FUNCTIONS *****/
#[inline]
fn hash<T: std::hash::Hash>(obj: T) -> u64 {
    let mut state = DefaultHasher::default();
    obj.hash(&mut state);
    state.finish()
}





/***** EXAMPLES *****/
/// Example empty struct as usual.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[allow(unused)]
enum Foo {}

/// Example tuple struct as usual.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
enum Bar {
    Variant1((), bool, String),
    #[cfg(feature = "serde")]
    Variant2(bool),
}

/// Example struct struct as usual.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
enum Baz {
    Variant1 { a: (), b: bool, c: String },
}



struct DontImplementAnything;

/// Special struct with generics that don't have to be debug.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
    // assert_eq!(Foo.cmp(&Foo), Ordering::Equal);
    assert_eq!(Bar::Variant1((), true, "Hello, world!".into()).cmp(&Bar::Variant1((), true, "Goodbye, world!".into())), Ordering::Greater);
    assert_eq!(
        Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() }.cmp(&Baz::Variant1 { a: (), b: true, c: "Howdy, world!".into() }),
        Ordering::Less
    );
    assert_eq!(
        PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData }.cmp(&PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData }),
        Ordering::Equal
    );



    // NOTE: Can't construct of course
    // assert_eq!(hash(Foo), 0);
    assert_eq!(hash(Bar::Variant1((), true, "Hello, world!".into())), 17152124978856657821);
    assert_eq!(hash(Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() }), 17152124978856657821);
    assert_eq!(hash(PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData }), 13646096770106105413);



    #[cfg(feature = "serde")]
    {
        // NOTE: Can't construct of course
        // assert_eq!(serde_json::to_string(&Foo).unwrap(), "null");
        assert_eq!(serde_json::to_string(&Bar::Variant1((), true, "Hello, world!".into())).unwrap(), "{\"Variant1\":[null,true,\"Hello, world!\"]}");
        assert_eq!(serde_json::to_string(&Bar::Variant2(true)).unwrap(), "{\"Variant2\":true}");
        assert_eq!(
            serde_json::to_string(&Baz::Variant1 { a: (), b: true, c: "Hello, world!".into() }).unwrap(),
            "{\"Variant1\":{\"a\":null,\"b\":true,\"c\":\"Hello, world!\"}}"
        );
        assert_eq!(
            serde_json::to_string(&PhantomEnum::Variant1::<DontImplementAnything> { _f: PhantomData }).unwrap(),
            "{\"Variant1\":{\"_f\":null}}"
        );
    }
}
