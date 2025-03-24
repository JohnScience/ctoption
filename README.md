# `ctoption`

[![Crates.io](https://img.shields.io/crates/v/ctoption)](https://crates.io/crates/ctoption)
[![Downloads](https://img.shields.io/crates/d/ctoption.svg)](https://crates.io/crates/ctoption)
[![Documentation](https://docs.rs/ctoption/badge.svg)](https://docs.rs/ctoption)
[![License](https://img.shields.io/crates/l/ctoption)](https://crates.io/crates/ctoption)
[![Dependency Status](https://deps.rs/repo/github/JohnScience/ctoption/status.svg)](https://deps.rs/repo/github/JohnScience/ctoption)

A compile-time option type for Rust with the discriminant elevated to compile time.

## Add to your project

```console
cargo add ctoption
```

## Usage

### Creating and using CTSome and CTNone

```rust
use ctoption::prelude::*;

// enforcing constant evaluation
const _: () = {
    let some = CTSome::new(42);
    assert!(some.is_some());
    assert!(some.into_inner() == 42);

    let none = CTNone::<u32>::new();
    assert!(!none.is_some());
    // at the moment of writing, const drop is unstable
    none.forget();
};
```

### Inserting values into a CTNone and extracting them from a CTSome

```rust
use ctoption::prelude::*;

// enforcing constant evaluation
const _: () = {
    let mut none: CTNone<u32> = CTNone::new();
    assert!(!none.is_some());
    // What insert does is: 
    // 1. accepts an owned value
    // 2. consumes the CTNone instance
    // 3. returns a CTSome instance
    let some: CTSome<u32> = none.insert(42);
    assert!(some.is_some());
    assert!(some.into_inner() == 42);
};
```

## Limitations

### `const` drop

At the moment of writing, [`const_destruct`](https://github.com/rust-lang/rust/issues/133214) feature of Rust is unstable. This means that `CTSome` and `CTNone` cannot be dropped in a `const` context.

However,

* `CTSome` can be disposed of by calling `CTSome::into_inner`,
* `CTNone` can be disposed of by calling `CTNone::forget`.

### `.map`-like methods

At the moment of writing, methods like [`std::option::Option::map`](https://doc.rust-lang.org/std/option/enum.Option.html#method.map) cannot be implemented for `CTSome` and `CTNone` because they require a closure to be passed to them. Closures cannot be evaluated in `const` contexts. And the attempts to provide a [sealed trait](https://rust-lang.github.io/api-guidelines/future-proofing.html) with an implementation for a function pointer failed because implementing a trait that would allow using it in a `const` context is impossible at the time of writing.

### Type narrowing

At the moment of writing, matching on `IS_SOME_VAL` for `CTOption<T, IS_SOME_VAL: bool` does not narrow the type to `CTSome<T>` or `CTNone<T>`.

However, you can manually narrow the type using the unsafe methods
`CTOption::assume_some` and `CTOption::assume_none`:

```rust
use ctoption::prelude::*;

const fn add_inner_or_none<const IS_SOME_VAL: bool>(a: u32, b: CTOption<u32, IS_SOME_VAL>) -> u32 {
   if b.is_some() {
     let some: CTSome<u32> = unsafe { b.assume_some() };
     a + some.into_inner()
   } else {
     let none: CTNone<u32> = unsafe { b.assume_none() };
     none.forget();
     a
   }
}

const _: () = {
  let some = CTSome::new(2);
  assert!(add_inner_or_none(2, some) == 4);
  let none = CTNone::<u32>::new();
  assert!(add_inner_or_none(2, none) == 2);
};
```

Additionally, since Rust's compiler can't infer after checking `IS_SOME_VAL` in const context that, for example, `CTOption<T, IS_SOME_VAL>` is the same as `CTSome<T>`, you may be in a situation where

1. you have `CTSome<T>`,
2. the check in const context inferred that `IS_SOME_VAL` is `true`,
3. you need to return `CTOption<T, IS_SOME_VAL>`

and be confused why returning `CTSome<T>` doesn't work.

To resolve this, you can use, `CTSome::assume_const_generic_val` or `CTNone::assume_const_generic_val`.

## License

Licensed under either of [Apache License, Version 2.0] or [MIT license] at your option.

[Apache License, Version 2.0]: https://www.apache.org/licenses/LICENSE-2.0
[MIT license]: https://opensource.org/licenses/MIT
