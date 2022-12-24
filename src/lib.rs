use core::mem::{ManuallyDrop, MaybeUninit};

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
/// [Type State]: http://cliffle.com/blog/rust-typestate/
/// [Builder Pattern]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
/// [contravariant]: https://en.m.wikipedia.org/wiki/Covariance_and_contravariance_(computer_science)
/// [unsugared]: https://en.wikipedia.org/wiki/Syntactic_sugar
/// [E0532]: https://doc.rust-lang.org/error_codes/E0532.html
/// [E0493]: https://doc.rust-lang.org/error_codes/E0493.html
#[repr(transparent)]
pub struct CTOption<T, const IS_SOME_VAL: bool>(MaybeUninit<T>);

pub const IS_SOME: bool = true;

pub const IS_NONE: bool = false;

pub type CTSome<T> = CTOption<T, IS_SOME>;

pub type CTNone<T> = CTOption<T, IS_NONE>;

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
    pub const fn is_some(&self) -> bool {
        IS_SOME_VAL
    }

    pub const unsafe fn assume_some(self) -> CTSome<T> {
        union CTOptionVariantUnion<U, const NESTED_IS_SOME_VAL: bool> {
            md_ctsome: ManuallyDrop<CTSome<U>>,
            md_ctopt: ManuallyDrop<CTOption<U,NESTED_IS_SOME_VAL>>,
        }

        let md_ctopt = ManuallyDrop::new(self);
        let u = CTOptionVariantUnion { md_ctopt };
        let md_ctsome = unsafe { u.md_ctsome };
        ManuallyDrop::into_inner(md_ctsome)
    }
}

// const fn do_one_thing() {}
// const fn do_another_thing() {}

// const fn extra_cleanup<T, const IS_SOME_VAL: bool>(opt: CTOption<T, IS_SOME_VAL>) {
//     match IS_SOME_VAL {
//         true => {
//             do_one_thing()
//         }
//         false => {
//             do_another_thing()
//         }
//     }
// }

impl<T, const IS_SOME_VAL: bool> Drop for CTOption<T, IS_SOME_VAL> {
    fn drop(&mut self) {
        if IS_SOME_VAL {
            unsafe { self.0.assume_init_drop() }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CTSome;

    #[test]
    fn into_inner_works() {
        let opt = CTSome::new(3);
        let v = opt.into_inner();
        assert_eq!(v, 3);
    }
}
