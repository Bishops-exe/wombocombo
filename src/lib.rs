extern crate self as wombocombo;

use std::cmp::Ordering;
use std::fmt::Alignment;

pub use wombocombo_macros::*;

#[doc(hidden)]
pub mod __private {
    pub use itertools::iproduct;

    // `#[macro_export]` hoists the macro to the crate root, so re-export it here
    // to make the `$crate::__private::generate_all_combinations!` path resolve.
    pub use crate::{define_values_iter, generate_all_combinations};

    #[macro_export]
    macro_rules! generate_all_combinations {
        (@fields $name:ident, $($struct_key:ident => $struct_key_type:ty),* $(,)?) => {
            $crate::__private::generate_all_combinations!(@with_constructor |($($struct_key,)*)| {
                $name {
                    $($struct_key,)*
                }
            }, $($struct_key => $struct_key_type),*)
        };

        (@tuple $name:ident, $($struct_key:ident => $struct_key_type:ty),* $(,)?) => {
            $crate::__private::generate_all_combinations!(@with_constructor |($($struct_key,)*)| {
                $name($($struct_key,)*)
            }, $($struct_key => $struct_key_type),*)
        };

        (@enum $enum_type:ident, $($enum_key:ident),* $(,)?)=> {
            [$($enum_type::$enum_key,)*]
        };

        (@with_constructor $constructor:expr, $($struct_key:ident => $struct_key_type:ty),* $(,)?) => {
            // Using itertools::iproduct! perfectly preserves ExactSizeIterator and size_hint properties.
            $crate::__private::iproduct!($(<$struct_key_type as $crate::AllValuesIter>::all_values_iter()),*).map($constructor)
        }
    }

    #[macro_export]
    macro_rules! define_values_iter {
        ($typ:ty, $iter:expr) => {
            impl AllValuesIter for $typ {
                $crate::__private::define_values_iter!(@fn $iter);
            }
        };
        (@fn $iter:expr) => {
            fn all_values_iter() -> impl Iterator<Item = Self> + Clone {
                $iter.into_iter()
            }
        };
    }
}

pub trait AllValuesIter: Sized + Clone {
    // `Clone` is required so the iterators can be fed to `itertools::iproduct!`,
    // whose cartesian product needs to re-iterate the inner iterators.
    fn all_values_iter() -> impl Iterator<Item = Self> + Clone;
}

pub trait AllValues: Sized {
    fn all_values() -> Vec<Self>;
}

impl<T> AllValues for T
where
    T: AllValuesIter,
{
    fn all_values() -> Vec<Self> {
        Self::all_values_iter().collect::<Vec<_>>()
    }
}

define_values_iter!(bool, [true, false]);
define_values_iter!(
    Ordering,
    [Ordering::Less, Ordering::Greater, Ordering::Equal]
);
define_values_iter!(
    Alignment,
    [Alignment::Left, Alignment::Right, Alignment::Center]
);

impl<T> AllValuesIter for Option<T>
where
    T: AllValuesIter + Clone,
{
    define_values_iter!(@fn [None].into_iter().chain(T::all_values_iter().map(Some)));
}

impl<S, E> AllValuesIter for Result<S, E>
where
    S: AllValuesIter,
    E: AllValuesIter,
{
    define_values_iter!(
        @fn
        S::all_values_iter().map(Ok).chain(E::all_values_iter().map(Err))
    );
}

#[cfg(test)]
mod tests {
    use wombocombo::{AllValues, AllValuesIter};

    macro_rules! assert_is {
        ($values:expr, $($value:expr),+) => {
            let mut count = 0;
            $(
                assert!($values.contains(&$value));
                count += 1;
            )+
            assert_eq!(count, $values.len());
        };
    }

    #[test]
    fn generates_all_combinations() {
        #[derive(AllValues, Hash, PartialEq, Eq, Debug, Clone)]
        struct Test {
            bool: bool,
        }

        let values = Test::all_values();

        assert_is!(values, Test { bool: false }, Test { bool: true });
    }

    #[test]
    fn generates_correct_size_hint() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        enum Value {
            Value1,
            Value2,
            Value3,
        }

        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Test {
            bool: bool,
            value: Value,
        }

        let values = Test::all_values_iter().size_hint();

        assert_eq!(values, (6, Some(6)));
    }

    #[test]
    fn derives_enum_all_values() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        enum Color {
            Red,
            Green,
            Blue,
        }

        let values = Color::all_values();

        assert_is!(values, Color::Red, Color::Green, Color::Blue);
        assert_eq!(Color::all_values_iter().size_hint(), (3, Some(3)));
    }

    #[test]
    fn derives_single_variant_enum() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        enum Unit {
            Only,
        }

        assert_is!(Unit::all_values(), Unit::Only);
        assert_eq!(Unit::all_values_iter().size_hint(), (1, Some(1)));
    }

    #[test]
    fn generates_full_cartesian_product() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        enum Value {
            A,
            B,
        }

        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Test {
            flag: bool,
            value: Value,
        }

        let values = Test::all_values();

        assert_is!(
            values,
            Test {
                flag: false,
                value: Value::A
            },
            Test {
                flag: false,
                value: Value::B
            },
            Test {
                flag: true,
                value: Value::A
            },
            Test {
                flag: true,
                value: Value::B
            }
        );
    }

    #[test]
    fn three_field_product_has_exact_size() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Test {
            a: bool,
            b: bool,
            c: bool,
        }

        assert_eq!(Test::all_values().len(), 8);
        assert_eq!(Test::all_values_iter().size_hint(), (8, Some(8)));
    }

    #[test]
    fn nested_struct_combinations() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Inner {
            a: bool,
        }

        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Outer {
            inner: Inner,
            b: bool,
        }

        let values = Outer::all_values();

        assert_is!(
            values,
            Outer {
                inner: Inner { a: false },
                b: false
            },
            Outer {
                inner: Inner { a: false },
                b: true
            },
            Outer {
                inner: Inner { a: true },
                b: false
            },
            Outer {
                inner: Inner { a: true },
                b: true
            }
        );
    }

    #[test]
    fn struct_with_option_field() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Test {
            maybe: Option<bool>,
        }

        let values = Test::all_values();

        assert_is!(
            values,
            Test { maybe: None },
            Test { maybe: Some(false) },
            Test { maybe: Some(true) }
        );
    }

    #[test]
    fn all_values_iter_is_clonable_and_repeatable() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Test {
            a: bool,
            b: bool,
        }

        let iter = Test::all_values_iter();
        let first: Vec<_> = iter.clone().collect();
        let second: Vec<_> = iter.collect();

        assert_eq!(first, second);
        assert_eq!(first.len(), 4);
    }

    #[test]
    fn builtin_bool_values() {
        assert_is!(bool::all_values(), false, true);
    }

    #[test]
    fn builtin_ordering_values() {
        use std::cmp::Ordering;

        assert_is!(
            Ordering::all_values(),
            Ordering::Less,
            Ordering::Equal,
            Ordering::Greater
        );
    }

    #[test]
    fn builtin_alignment_values() {
        use std::fmt::Alignment;

        assert_is!(
            Alignment::all_values(),
            Alignment::Left,
            Alignment::Right,
            Alignment::Center
        );
    }

    #[test]
    fn option_values() {
        assert_is!(Option::<bool>::all_values(), None, Some(false), Some(true));
    }

    #[test]
    fn result_values() {
        use std::cmp::Ordering;

        assert_is!(
            Result::<bool, Ordering>::all_values(),
            Ok(false),
            Ok(true),
            Err(Ordering::Less),
            Err(Ordering::Equal),
            Err(Ordering::Greater)
        );
    }

    #[test]
    fn supports_tuples() {
        #[derive(AllValues, Clone, Hash, PartialEq, Eq, Debug)]
        struct Test(bool);

        assert_is!(Test::all_values(), Test(true), Test(false));
    }
}
