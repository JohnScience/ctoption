#![no_std]
#![cfg_attr(feature = "const_trait_impl", feature(const_trait_impl))]
#![cfg_attr(feature = "core_intrinsics", feature(core_intrinsics))]
#![cfg_attr(
    feature = "adt_const_params",
    allow(incomplete_features),
    feature(adt_const_params)
)]
#![cfg_attr(
    feature = "const_precise_live_drops",
    feature(const_precise_live_drops)
)]

use core::mem::{ManuallyDrop, MaybeUninit};

/// A [`prelude`] for this crate, which is meant to be used as `use ctoption::prelude::*;`.
/// 
/// [`prelude`]: https://doc.rust-lang.org/std/prelude/index.html#other-preludes
pub mod prelude;

/// A compile-time alternative to [`Option`]. Unlike [`Option`],
/// this type is guaranteed to have the same size and alignmemt as `T`.
///
/// This type is especially useful for software targetting memory-constrained
/// environments where the size of types matters. However, its interface
/// can be just as ergonomic as that of [`Option`].
///
/// One important application of this type is a category of [Type State] variations
/// of the [Builder Pattern]. One example implementation of this pattern
/// will be provided after a few more beginner-friendly examples.
///
/// # Examples
///
/// ## Create and get the inner value back
///
/// ```
/// use ctoption::prelude::*;
///
/// // enforcing constant evaluation
/// const _: () = {
///     let some = CTSome::new(42);
///     assert!(some.into_inner() == 42);
/// };
/// ```
///
/// This crate offers a [`prelude`] module with the most essential items.
///
/// One of these items is the [`CTSome`] type alias, which is a convenience
/// type inspired by [`Some`] variant of the [`Option`] type.
///
/// `CTSome::new` and `CTSome::into_inner` are the two inverse associated
/// functions. One wraps a value in a `CTSome` instance, while the other
/// turns the [`CTSome`] instance into the inner value.
///
/// ## Create empty, insert the value, and get the value back
///
/// ```
/// use ctoption::prelude::*;
///
/// const _: () = {
///     let none = CTNone::new();
///     let some = none.insert(42);
///     assert!(some.into_inner() == 42);
/// };
/// ```
///
/// or, if slightly [unsugared],
///
/// ```
/// use ctoption::prelude::*;
///
/// const _: () = {
///     let none: CTOption<i32,IS_NONE> = CTNone::<i32>::new();
///     let some: CTOption<i32,IS_SOME> = {
///         let v: CTSome<i32> = none.insert(42);
///         v
///     };
///     assert!(some.into_inner() == 42);
/// };
/// ```
///
/// Similarly to [`CTSome`], [`CTNone`] is a type alias inspired by an
/// enum variant of [`Option`] type - [`None`].
///
/// The fact that [`CTSome`] and [`CTNone`] are type aliases and not
/// enum variants has deep impact on the API. Compare the code above
/// to the nearly equivalent code using [`Option`] type:
///
/// ```
/// // unwrap is unstable in const context
/// // const _: () = {
///     let mut enum_instance = None;
///     enum_instance = Some(42);
///     assert!(enum_instance.unwrap() == 42);
/// // };
/// ```
///
/// or, if unsugared similarly,
///
/// ```
/// // const _: () = {
///     let mut enum_instance: Option<i32> = None::<i32>;
///     enum_instance = Some::<i32>(42);
///     assert!(enum_instance.unwrap() == 42);
/// // };
/// ```
///
/// Unlike [`Some`] and [`None`] enum variants of [`Option`] type, [`CTSome`]
/// and [`CTNone`] correspond to (distinct) types themselves and do not
/// share a common overarching [contravariant] type (except for `!`) for every
/// parameterization. Therefore, turning an instance of one `CT*` type into an
/// instance of another `CT*` type requires juggling owned values. This is the
/// reason why the `CTNone::insert` associated function accepts an owned value
/// of [`CTNone`] and returns an owned value of [`CTSome`] - a mutable reference
/// wouldn't suffice.
///
/// However, both [`CTSome`] and [`CTNone`] are merely type aliases which allow
/// to conveniently refer to different partial parametrizations of the same
/// generic type - [`CTOption`].
///
/// ## Matching `CTOption` instances in generic code
///
/// It would be awesome if we could write something like this:
///
/// ```compile_fail
/// const fn get_ty_alias_name<T, const IS_SOME_VAL: bool>(opt: &CTOption<T, IS_SOME_VAL>) -> &'static str {
///     match opt {
///         CTSome(x) => {
///             "CTSome"
///         }
///         CTNone => {
///             "CTNone"
///         }
///     }
/// }
/// ```
///
/// However, this code won't work because match expression doesn't allow to match the value against
/// different types. If you try to do this, you'll get
/// [`error[E0532]: expected tuple struct or tuple variant, found type alias CTSome.`][E0532]
///
/// Then what about matching against the value of `IS_SOME_VAL`? Let's say we want to write a cleanup function.
///
/// ```compile_fail
/// const fn do_one_thing() {}
/// const fn do_another_thing() {}
///
/// const fn extra_cleanup<T, const IS_SOME_VAL: bool>(opt: CTOption<T, IS_SOME_VAL>) {
///     match IS_SOME_VAL {
///         true => {
///             do_one_thing()
///         }
///         false => {
///             do_another_thing()
///         }
///     }
/// }
/// ```
///
/// Then you'll encounter another problem:
/// [`error[E0493]: destructor of CTOption<T, IS_SOME_VAL> cannot be evaluated at compile-time`][E0493]. This
/// happens because [`CTOption`] has a custom implementation of [`Drop`] trait and custom implementations
/// cannot be evaluated at compile-time.
///
/// In theory, this could eventually be solved by [`core::marker::Destruct`] trait, but it is not the case
/// at the moment of writing this.
///
/// [Type State]: http://cliffle.com/blog/rust-typestate/
/// [Builder Pattern]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
/// [contravariant]: https://en.m.wikipedia.org/wiki/Covariance_and_contravariance_(computer_science)
/// [unsugared]: https://en.wikipedia.org/wiki/Syntactic_sugar
/// [E0532]: https://doc.rust-lang.org/error_codes/E0532.html
/// [E0493]: https://doc.rust-lang.org/error_codes/E0493.html
#[repr(transparent)]
pub struct CTOption<T, const IS_SOME_VAL: bool>(MaybeUninit<T>);

/// A convenience constant for `true`, which is used to indicate that
/// the [`CTOption`] instance is a [`CTSome`].
pub const IS_SOME: bool = true;

/// A convenience constant for `false`, which is used to indicate that
/// the [`CTOption`] instance is a [`CTNone`].
pub const IS_NONE: bool = false;

// the literals are used in the constants due to the bug of rust-analyzer:
// https://github.com/rust-lang/rust-analyzer/issues/15821

/// A convenience type alias for [`CTOption`] with `true` as the second
/// type parameter. This type alias is inspired by the [`Some`] variant
/// of the [`Option`] type.
pub type CTSome<T> = CTOption<T, true>;

/// A convenience type alias for [`CTOption`] with `false` as the second
/// type parameter. This type alias is inspired by the [`None`] variant
/// of the [`Option`] type.
pub type CTNone<T> = CTOption<T, false>;

#[doc(hidden)]
pub unsafe trait OptionalConstGeneric {
    type Inner;
    const IS_SOME_VAL: bool;
}

#[cfg(feature = "adt_const_params")]
pub mod workarounds {
    use core::marker::ConstParamTy;

    #[derive(Eq, PartialEq, ConstParamTy)]
    pub enum Option<T> {
        Some(T),
        None,
    }

    impl<T> Option<T> {
        #[cfg(feature = "const_precise_live_drops")]
        pub const fn into_core(self) -> core::option::Option<T> {
            match self {
                Self::Some(x) => core::option::Option::Some(x),
                Self::None => core::option::Option::None,
            }
        }
    }
}

unsafe impl<T, const IS_SOME_VAL: bool> OptionalConstGeneric for CTOption<T, IS_SOME_VAL> {
    type Inner = T;
    const IS_SOME_VAL: bool = IS_SOME_VAL;
}

#[cfg(feature = "core_intrinsics")]
pub mod opt_const_generic {
    use super::{CTSome, OptionalConstGeneric};
    use core::mem::MaybeUninit;

    pub const fn to_option<T: OptionalConstGeneric>(opt: T) -> Option<T::Inner> {
        let storage: MaybeUninit<T::Inner> = unsafe { core::intrinsics::transmute_unchecked(opt) };
        match T::IS_SOME_VAL {
            true => {
                let opt = unsafe { CTSome::<T::Inner>::from_maybe_uninit(storage) };
                Some(opt.into_inner())
            }
            false => None,
        }
    }
}

impl<T> CTSome<T> {
    pub const fn new(val: T) -> Self {
        Self(MaybeUninit::new(val))
    }

    pub const fn into_inner(self) -> T {
        union CTSomeUnion<T> {
            md_ctsome: ManuallyDrop<CTSome<T>>,
            md_inner: ManuallyDrop<T>,
        }

        let md_ctsome = ManuallyDrop::new(self);

        let u = CTSomeUnion { md_ctsome };

        let md_inner = unsafe { u.md_inner };
        ManuallyDrop::into_inner(md_inner)
    }

    pub const unsafe fn assume_const_generic_val<const IS_SOME_VAL: bool>(
        self,
    ) -> CTOption<T, IS_SOME_VAL> {
        union CTOptionVariantUnion<U, const NESTED_IS_SOME_VAL: bool> {
            md_ctsome: ManuallyDrop<CTSome<U>>,
            md_ctopt: ManuallyDrop<CTOption<U, NESTED_IS_SOME_VAL>>,
        }

        let md_ctsome = ManuallyDrop::new(self);
        let u = CTOptionVariantUnion { md_ctsome };
        let md_ctopt = unsafe { u.md_ctopt };
        ManuallyDrop::into_inner(md_ctopt)
    }
}

impl<T> CTNone<T> {
    pub const fn new() -> Self {
        Self(MaybeUninit::uninit())
    }

    pub const fn insert(mut self, val: T) -> CTSome<T> {
        union CTOptionVariantUnion<T> {
            md_ctsome: ManuallyDrop<CTSome<T>>,
            md_ctnone: ManuallyDrop<CTNone<T>>,
        }

        self.0 = MaybeUninit::new(val);
        let md_ctnone = ManuallyDrop::new(self);
        let u = CTOptionVariantUnion { md_ctnone };
        let md_ctsome = unsafe { u.md_ctsome };
        ManuallyDrop::into_inner(md_ctsome)
    }
}

impl<T, const IS_SOME_VAL: bool> CTOption<T, IS_SOME_VAL> {
    pub const unsafe fn from_maybe_uninit(val: MaybeUninit<T>) -> Self {
        Self(val)
    }

    pub const fn is_some(&self) -> bool {
        IS_SOME_VAL
    }

    pub const unsafe fn assume_some(self) -> CTSome<T> {
        union CTOptionVariantUnion<U, const NESTED_IS_SOME_VAL: bool> {
            md_ctsome: ManuallyDrop<CTSome<U>>,
            md_ctopt: ManuallyDrop<CTOption<U, NESTED_IS_SOME_VAL>>,
        }

        let md_ctopt = ManuallyDrop::new(self);
        let u = CTOptionVariantUnion { md_ctopt };
        let md_ctsome = unsafe { u.md_ctsome };
        ManuallyDrop::into_inner(md_ctsome)
    }

    pub const unsafe fn assume_none(self) -> CTNone<T> {
        union CTOptionVariantUnion<U, const NESTED_IS_SOME_VAL: bool> {
            md_ctnone: ManuallyDrop<CTNone<U>>,
            md_ctopt: ManuallyDrop<CTOption<U, NESTED_IS_SOME_VAL>>,
        }

        let md_ctopt = ManuallyDrop::new(self);
        let u = CTOptionVariantUnion { md_ctopt };
        let md_ctnone = unsafe { u.md_ctnone };
        ManuallyDrop::into_inner(md_ctnone)
    }
}

#[cfg(not(feature = "const_trait_impl"))]
impl<T, const IS_SOME_VAL: bool> Drop for CTOption<T, IS_SOME_VAL> {
    fn drop(&mut self) {
        if IS_SOME_VAL {
            unsafe { self.0.assume_init_drop() }
        }
    }
}

#[cfg(feature = "const_trait_impl")]
macro_rules! provide_items_guarded_by_const_trait_impl {
    () => {
        // The items are in a macro because they use a new syntax, which is not
        // a valid Rust syntax at the moment of writing this.
        pub const fn const_drop<T: ~const core::marker::Destruct>(val: T) {
            core::mem::forget(val);
        }

        impl<T, const IS_SOME_VAL: bool> const Drop for CTOption<T, IS_SOME_VAL> {
            fn drop(&mut self) {
                if IS_SOME_VAL {
                    unsafe { self.0.assume_init_drop() }
                }
            }
        }
    };
}

#[cfg(feature = "const_trait_impl")]
provide_items_guarded_by_const_trait_impl!();

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    const fn into_inner_works() {
        let opt = CTSome::new(3);
        let v = opt.into_inner();
        if v != 3 {
            panic!("v != 3");
        }
    }

    #[test]
    const fn test_insert() {
        let none = CTNone::<i32>::new();
        let some = none.insert(42);
        assert!(some.into_inner() == 42);
    }
}
