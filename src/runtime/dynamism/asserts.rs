// TODO: Get rid of RefCell, it's checked elsewhere and I'm afraid I'll accidentally Copy it
use std::{any::{Any, TypeId}, borrow::BorrowMut, cell::{Cell, Ref, RefCell, RefMut, UnsafeCell}, marker::PhantomData, mem::{MaybeUninit, size_of}};

pub struct RefToUnknown<'a> {
    ptr: *const u8, 
    len: usize, 
    phantom: std::marker::PhantomData<&'a u8>,
}

#[repr(C)]  // needed to guarantee that initialized and type_id are in the same place no matter what T is
pub struct InPlaceCell<T> {
    initialized: Cell<bool>,
    type_id: Cell<TypeId>,
    value: Cell<MaybeUninit<T>>,
}

#[repr(C)]  // needed to guarantee that initialized and type_id are in the same place no matter what T is
pub struct InPlaceRefCell<T> {
    initialized: Cell<bool>,
    type_id: Cell<TypeId>,
    value: UnsafeCell<MaybeUninit<RefCell<T>>>,
}


impl<'a> RefToUnknown<'a> {
    pub fn from(ptr: *const u8, len: usize) -> RefToUnknown<'a> { 
        RefToUnknown { ptr, len,  phantom: PhantomData }
    }

    pub fn cast_copy<T: Any+Copy>(&self) -> &'a InPlaceCell<T> {
        assert_eq!(self.len, size_of::<InPlaceCell<T>>());
        let s: &'a InPlaceCell<T> = unsafe { std::mem::transmute(&*self.ptr) };
        assert_eq!(s.type_id.get(), TypeId::of::<Cell<T>>());
        s
    }

    pub fn cast_nocopy<T: Any>(&self) -> &'a InPlaceRefCell<T> {
        assert_eq!(self.len, size_of::<InPlaceRefCell<T>>());
        let s: &'a InPlaceRefCell<T> = unsafe { std::mem::transmute(&*self.ptr) };
        assert_eq!(s.type_id.get(), TypeId::of::<RefCell<T>>());
        s
    }

    pub(crate) fn initialize_metadata(&self, type_id: Option<TypeId>) {
        let v: &'a InPlaceCell<()> = unsafe { std::mem::transmute(&*self.ptr) };

        v.initialized.replace(false);
        if let Some(tid) = type_id {
            v.type_id.replace(tid);
        } else {
            struct Crap {}
            v.type_id.replace(TypeId::of::<Crap>()); // make all type asserts fail
        }
    }
}

impl<T> InPlaceCell<T> {
    pub fn initialize(&self, t: T) {
        // println!("checking initialized ({:?})", (&self.initialized as *const bool));
        // println!("initializing {:?}", self as *const InPlace<T>);
        assert!(!self.initialized.get());
        self.value.replace(MaybeUninit::new(t));
        self.initialized.replace(true);
    }
}

impl<T: Copy> InPlaceCell<T> {
    pub fn get(&self) -> T {
        // println!("reading {:?}", self as *const InPlace<T>);
        assert!(self.initialized.get());
        unsafe { self.value.get().assume_init() }
    }
}

impl<T> InPlaceRefCell<T> {
    pub fn initialize(&self, t: T) {
        // println!("checking initialized ({:?})", (&self.initialized as *const bool));
        // println!("initializing {:?}", self as *const InPlace<T>);
        assert!(!self.initialized.get());
        let ptr = self.value.get();
        unsafe { *ptr = MaybeUninit::new(RefCell::new(t)); }
        self.initialized.replace(true);
    }

    pub fn get(&self) -> Ref<'_, T> {
        // println!("reading {:?}", self as *const InPlace<T>);
        assert!(self.initialized.get());
        unsafe { (*self.value.get()).assume_init_ref().borrow() }
    }

    pub fn get_mut(&self) -> RefMut<'_, T> {
        // println!("reading {:?}", self as *const InPlace<T>);
        assert!(self.initialized.get());
        unsafe { (*self.value.get()).assume_init_ref().borrow_mut() }
    }
}