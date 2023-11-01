#![feature(generic_const_exprs)]

mod builder {
    use ctoption::prelude::*;

    pub(super) struct Builder<
        const B0: bool,
        const B1: bool,
        const B2: bool,
        const B3: bool,
        const B4: bool,
    > {
        field0: CTOption<i32, B0>,
        field1: CTOption<i32, B1>,
        field2: CTOption<i32, B2>,
        field3: CTOption<i32, B3>,
        field4: CTOption<i32, B4>,
    }

    impl Builder<false, false, false, false, false> {
        pub(super) fn new() -> Self {
            Self {
                field0: CTNone::new(),
                field1: CTNone::new(),
                field2: CTNone::new(),
                field3: CTNone::new(),
                field4: CTNone::new(),
            }
        }
    }

    impl<const B1: bool, const B2: bool, const B3: bool, const B4: bool>
        Builder<false, B1, B2, B3, B4>
    {
        pub(super) fn set_field0(self, val: i32) -> Builder<true, B1, B2, B3, B4> {
            Builder {
                field0: CTSome::new(val),
                field1: self.field1,
                field2: self.field2,
                field3: self.field3,
                field4: self.field4,
            }
        }
    }

    impl<const B0: bool, const B2: bool, const B3: bool, const B4: bool>
        Builder<B0, false, B2, B3, B4>
    {
        pub(super) fn set_field1(self, val: i32) -> Builder<B0, true, B2, B3, B4> {
            Builder {
                field0: self.field0,
                field1: CTSome::new(val),
                field2: self.field2,
                field3: self.field3,
                field4: self.field4,
            }
        }
    }

    impl<const B0: bool, const B1: bool, const B3: bool, const B4: bool>
        Builder<B0, B1, false, B3, B4>
    {
        pub(super) fn set_field2(self, val: i32) -> Builder<B0, B1, true, B3, B4> {
            Builder {
                field0: self.field0,
                field1: self.field1,
                field2: CTSome::new(val),
                field3: self.field3,
                field4: self.field4,
            }
        }
    }

    impl<const B0: bool, const B1: bool, const B2: bool, const B4: bool>
        Builder<B0, B1, B2, false, B4>
    {
        pub(super) fn set_field3(self, val: i32) -> Builder<B0, B1, B2, true, B4> {
            Builder {
                field0: self.field0,
                field1: self.field1,
                field2: self.field2,
                field3: CTSome::new(val),
                field4: self.field4,
            }
        }
    }

    impl<const B0: bool, const B1: bool, const B2: bool, const B3: bool>
        Builder<B0, B1, B2, B3, false>
    {
        pub(super) fn set_field4(self, val: i32) -> Builder<B0, B1, B2, B3, true> {
            Builder {
                field0: self.field0,
                field1: self.field1,
                field2: self.field2,
                field3: self.field3,
                field4: CTSome::new(val),
            }
        }
    }

    impl<const B0: bool, const B1: bool, const B2: bool, const B3: bool, const B4: bool>
        Builder<B0, B1, B2, B3, B4>
    {
        const LEN: usize =
            { (B0 as usize) + (B1 as usize) + (B2 as usize) + (B3 as usize) + (B4 as usize) };

        pub(super) fn build(self) -> [i32; Self::LEN] {
            let mut arr = [0; Self::LEN];

            let mut i = 0;

            let Self {
                field0,
                field1,
                field2,
                field3,
                field4,
            } = self;

            if B0 {
                arr[i] = unsafe { field0.assume_some() }.into_inner();
                i += 1;
            }

            if B1 {
                arr[i] = unsafe { field1.assume_some() }.into_inner();
                i += 1;
            }

            if B2 {
                arr[i] = unsafe { field2.assume_some() }.into_inner();
                i += 1;
            }

            if B3 {
                arr[i] = unsafe { field3.assume_some() }.into_inner();
                i += 1;
            }

            if B4 {
                arr[i] = unsafe { field4.assume_some() }.into_inner();
                // i += 1;
            }

            arr
        }
    }
}

use builder::Builder;

fn main() {
    let b = Builder::new();
    let v = b
        .set_field0(1)
        .set_field1(2)
        .set_field2(3)
        .set_field3(3)
        .set_field4(5)
        .build();
    assert!(v == [1, 2, 3, 3, 5]);
    assert!(core::mem::size_of_val(&v) == 5 * core::mem::size_of::<i32>());

    let b = Builder::new();
    let v = b
        .set_field0(1)
        .set_field1(2)
        .set_field2(3)
        .set_field3(3)
        .build();
    assert!(v == [1, 2, 3, 3]);
    assert!(core::mem::size_of_val(&v) == 4 * core::mem::size_of::<i32>());

    // Note: this is a stepping stone to generic const arrays and strings.
}
