// The test can be ran with `cargo test --test const_genericity --features=adt_const_params,const_precise_live_drops`

#![cfg(all(feature = "adt_const_params", feature = "const_precise_live_drops"))]
// this is used to disable the warning for adt_const_params feature
#![allow(incomplete_features)]
#![feature(adt_const_params, const_precise_live_drops)]

use ctoption::workarounds::Option;

#[derive(Debug, PartialEq)]
enum Evaluation {
    Constant(u32),
    Dynamic,
}

const fn my_const_generic_fn<const PARAM: Option<u32>>() -> Evaluation {
    let opt = PARAM.into_core();
    match opt {
        Some(v) => Evaluation::Constant(v),
        None => Evaluation::Dynamic,
    }
}

fn main() {
    assert_eq!(
        my_const_generic_fn::<{ Option::Some(42) }>(),
        Evaluation::Constant(42)
    );
    assert_eq!(
        my_const_generic_fn::<{ Option::None }>(),
        Evaluation::Dynamic
    );
}
