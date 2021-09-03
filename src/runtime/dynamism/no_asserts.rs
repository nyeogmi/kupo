use std::{any::Any, mem::{MaybeUninit, swap, transmute}};

pub struct RefToUnknown<'a>(&'a [u8]);
pub struct MutToUnknown<'a>(&'a mut [u8]);

pub struct InPlace<T> {
    value: MaybeUninit<T>,
}

impl<'a> RefToUnknown<'a> {
    pub fn from(slice: &'a [u8]) -> RefToUnknown<'a> { RefToUnknown(slice) }

    pub fn cast<T: Any>(self) -> &'a InPlace<T> {
        let s: &'a [u8] = self.0;
        let s: &'a InPlace<T> = unsafe { transmute(&s[0]) };
        s
    }
}

impl<'a> MutToUnknown<'a> {
    pub fn from(slice: &'a mut [u8]) -> MutToUnknown<'a> { MutToUnknown(slice) }

    pub fn cast<T: Any>(self) -> &'a mut InPlace<T> {
        let s: &'a mut [u8] = self.0;
        let s: &'a mut InPlace<T> = unsafe { transmute(&mut s[0]) };
        s
    }

    pub(crate) fn downgrade(self) -> RefToUnknown<'a> {
        RefToUnknown(self.0)
    }
}

impl<T> InPlace<T> {
    pub fn initialize(&mut self, t: T) {
        self.value = MaybeUninit::new(t);
    }

    pub fn get(&self) -> &T {
        unsafe { self.value.assume_init_ref() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { self.value.assume_init_mut() }
    }

    pub fn extract(&mut self) -> T {
        let mut mu2 = MaybeUninit::uninit();

        swap(&mut self.value, &mut mu2);

        unsafe { mu2.assume_init() }
    }
}