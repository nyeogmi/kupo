// TODO: Get rid of RefCell, it's checked elsewhere and I'm afraid I'll accidentally Copy it
use std::{any::{Any, TypeId}, cell::{Cell}, marker::PhantomData, mem::{MaybeUninit, size_of}};

pub struct RefToUnknown<'a> {
    ptr: *const u8, 
    len: usize, 
    phantom: std::marker::PhantomData<&'a u8>,
}

#[repr(C)]  // needed to guarantee that initialized and type_id are in the same place no matter what T is
pub struct InPlace<T> {
    initialized: Cell<bool>,
    type_id: Cell<TypeId>,
    value: Cell<MaybeUninit<T>>,
}


impl<'a> RefToUnknown<'a> {
    pub fn from(ptr: *const u8, len: usize) -> RefToUnknown<'a> { 
        RefToUnknown { ptr, len,  phantom: PhantomData }
    }

    pub fn cast<T: Any>(&self) -> &'a InPlace<T> {
        assert_eq!(self.len, size_of::<InPlace<T>>());
        let s: &'a InPlace<T> = unsafe { std::mem::transmute(&*self.ptr) };
        assert_eq!(s.type_id.get(), TypeId::of::<T>());
        s
    }

    pub(crate) fn initialize_metadata(&self, type_id: Option<TypeId>) {
        let v: &'a InPlace<()> = unsafe { std::mem::transmute(&*self.ptr) };

        v.initialized.replace(false);
        if let Some(tid) = type_id {
            v.type_id.replace(tid);
        } else {
            struct Crap {}
            v.type_id.replace(TypeId::of::<Crap>()); // make all type asserts fail
        }
    }
}

impl<T> InPlace<T> {
    pub fn initialize(&self, t: T) {
        // println!("checking initialized ({:?})", (&self.initialized as *const bool));
        // println!("initializing {:?}", self as *const InPlace<T>);
        assert!(!self.initialized.get());
        self.value.replace(MaybeUninit::new(t));
        self.initialized.replace(true);
    }
}

impl<T: Copy> InPlace<T> {
    pub fn get(&self) -> T {
        // println!("reading {:?}", self as *const InPlace<T>);
        assert!(self.initialized.get());
        unsafe { self.value.get().assume_init() }
    }
}