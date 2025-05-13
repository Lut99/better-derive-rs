//  CUSTOM.rs
//    by Lut99
//
//  Created:
//    06 Feb 2025, 11:51:12
//  Last edited:
//    13 Feb 2025, 15:31:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Showcases the use of defining custom trait bounds.
//

use std::cmp::Ordering;
use std::hash::{DefaultHasher, Hasher};
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





/***** EXAMPLE STRUCTS *****/
/// First half of the co-dependent struct.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct Foo<T> {
    foo: Wrapper<T>,
    bar: Bar<T>,
}

/// Second half of the co-dependent struct.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[better_derive(bound = (Wrapper<T>))]
struct Bar<T> {
    foos: Vec<Foo<T>>,
}

/// Some common ancestor.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct Wrapper<T>(PhantomData<T>);





/***** ENTRYPOINT *****/
fn main() {
    let foo1 = Foo { foo: Wrapper(PhantomData::<&str>), bar: Bar { foos: vec![] } };
    let foo2 = Foo { foo: Wrapper(PhantomData::<&str>), bar: Bar { foos: vec![foo1.clone()] } };

    // Check the traits are implemented
    assert!(foo1.clone() == foo1);
    assert!(format!("{foo1:?}") == "Foo { foo: Wrapper(PhantomData<&str>), bar: Bar { foos: [] } }");
    assert!(foo1 == foo1);
    assert_eq!(foo1.partial_cmp(&foo2), Some(Ordering::Less));
    assert!(hash(&foo1) == hash(&foo1));
    assert!(foo1 != foo2);
    #[cfg(feature = "serde")]
    assert_eq!(serde_json::to_string(&foo1).unwrap(), "{\"foo\":null,\"bar\":{\"foos\":[]}}");
}
