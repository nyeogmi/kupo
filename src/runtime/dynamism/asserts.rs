// TODO: Get rid of RefCell, it's checked elsewhere and I'm afraid I'll accidentally Copy it
use std::{any::{Any, TypeId}, cell::{Ref, RefCell, RefMut}, mem::{MaybeUninit, size_of, swap, transmute}};

pub struct RefToUnknown<'a>(&'a [u8]);
pub struct MutToUnknown<'a>(&'a mut [u8]);

#[repr(C)]  // needed to guarantee that initialized and type_id are in the same place no matter what T is
pub struct InPlace<T> {
    initialized: bool, type_id: TypeId,
    value: MaybeUninit<RefCell<T>>,
}

impl<'a> RefToUnknown<'a> {
    pub fn from(slice: &'a [u8]) -> RefToUnknown<'a> { RefToUnknown(slice) }

    pub fn cast<T: Any>(self) -> &'a InPlace<T> {
        let s: &'a [u8] = self.0;
        assert_eq!(s.len(), size_of::<InPlace<T>>());
        let s: &'a InPlace<T> = unsafe { transmute(&s[0]) };
        assert_eq!(s.type_id, TypeId::of::<T>());
        s
    }
}

impl<'a> MutToUnknown<'a> {
    pub fn from(slice: &'a mut [u8]) -> MutToUnknown<'a> { MutToUnknown(slice) }

    pub fn cast<T: Any>(self) -> &'a mut InPlace<T> {
        let s: &'a mut [u8] = self.0;
        assert_eq!(s.len(), size_of::<InPlace<T>>());
        let s: &'a mut InPlace<T> = unsafe { transmute(&mut s[0]) };
        assert_eq!(s.type_id, TypeId::of::<T>());
        s
    }

    pub(crate) fn initialize_asserts(&mut self, type_id: Option<TypeId>) {
        let s: &mut [u8] = self.0;
        let s: &mut InPlace<()> = unsafe { transmute(&mut s[0]) };
        // println!("initializing at {:?} ({:?})", s as *mut InPlace<()>, (&s.initialized as *const bool));
        s.initialized = false;
        if let Some(tid) = type_id {
            s.type_id = tid;
        } else {
            // make all type asserts fail
            struct Crap {}
            s.type_id = TypeId::of::<Crap>();
        }
    }

    pub(crate) fn downgrade(self) -> RefToUnknown<'a> {
        RefToUnknown(self.0)
    }
}

impl<T> InPlace<T> {
    pub fn initialize(&mut self, t: T) {
        // println!("checking initialized ({:?})", (&self.initialized as *const bool));
        // println!("initializing {:?}", self as *const InPlace<T>);
        assert!(!self.initialized);
        self.value = MaybeUninit::new(RefCell::new(t));
        self.initialized = true;
    }

    pub fn get(&self) -> Ref<'_, T> {
        // println!("reading {:?}", self as *const InPlace<T>);
        assert!(self.initialized);
        unsafe { self.value.assume_init_ref().borrow() }
    }

    pub fn get_mut(&mut self) -> RefMut<'_, T> {
        // println!("getting mut for {:?}", self as *const InPlace<T>);
        assert!(self.initialized);
        unsafe { self.value.assume_init_mut().borrow_mut() }
    }

    pub fn extract(&mut self) -> T {
        // println!("extracting from {:?}", self as *const InPlace<T>);
        assert!(self.initialized);
        unsafe { self.value.assume_init_mut().borrow_mut(); } // ensure no refs

        let mut mu2 = MaybeUninit::uninit();

        swap(&mut self.value, &mut mu2);
        self.initialized = false;

        unsafe { mu2.assume_init().into_inner() }
    }
}