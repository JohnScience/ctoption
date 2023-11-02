#![feature(adt_const_params)]

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
