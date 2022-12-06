use core::mem::{ManuallyDrop, MaybeUninit};

#[repr(transparent)]
pub struct CTOption<T, const IS_SOME_VAL: bool>(MaybeUninit<T>);

pub const IS_SOME: bool = true;

pub const IS_NONE: bool = false;

pub type CTSome<T> = CTOption<T, IS_SOME>;

pub type CTNone<T> = CTOption<T, IS_NONE>;

union CTOptionVariantUnion<T> {
    pub md_ctsome: ManuallyDrop<CTSome<T>>,
    pub md_ctnone: ManuallyDrop<CTNone<T>>,
}

union CTSomeUnion<T> {
    pub md_ctsome: ManuallyDrop<CTSome<T>>,
    pub md_inner: ManuallyDrop<T>,
}

impl<T> CTSome<T> {
    pub const fn new(val: T) -> Self {
        Self(MaybeUninit::new(val))
    }

    pub const fn into_inner(self) -> T {
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
}

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
