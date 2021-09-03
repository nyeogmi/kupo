use std::{marker::PhantomData, mem::size_of};

use moogle::{SharedAnyToOne, IdLike};

struct ConcreteToOne<'a, T, RealK: IdLike, RealV: IdLike>(&'a T, PhantomData<(*const RealK, *const RealV)>)
where T: SharedAnyToOne<'a, RealK, RealV>;

type K<'a> = &'a [u8];
type V<'a> = &'a [u8];
type VMut<'a> = &'a mut [u8];

// TODO: Have a type for an "iterator variant" -- it's any iterator that DynToOne can return, including reversed versions.
// Allow that type to be allocated on the stack, then provide methods to advance it.
// Ideally it wouldn't have to be a variant -- we could eliminate conditionals by providing several separate types. 
// That might be a good optimization for later!
// TODO: Encapsulate all the size check logic behind a type called Alloc and a partner type DynAlloc. 
// Then provide a cfg(feature) to replace Alloc with a pointery version instead of a slice-y version.

trait DynToOne<'a> {
    /*
    type Iter: 'a+DoubleEndedIterator<Item=(K<'a>, V<'a>)>;
    type Keys: 'a+DoubleEndedIterator<Item=K<'a>>;
    type Values: 'a+DoubleEndedIterator<Item=V<'a>>;
    */

    const KSIZE: usize;
    const VSIZE: usize;

    fn key_size() -> usize { Self::KSIZE }
    fn value_size() -> usize { Self::VSIZE }

    fn get(&self, k: K<'a>, out: VMut<'a>) -> bool;
    fn contains_key(&self, k: K<'a>) -> bool;
    fn len(&self) -> usize;

    fn contains(&self, k: K<'a>, v: V<'a>) -> bool;

    /*
    fn iter(&self) -> Self::Iter;
    fn keys(&self) -> Self::Keys;
    fn values(&self) -> Self::Values;
    */

    fn insert(&self, k: K<'a>, v: V<'a>) -> bool;
    fn expunge(&self, k: K<'a>) -> bool;

    fn remove(&self, k: K<'a>, v: V<'a>, out: VMut<'a>) -> bool;
}

// TODO: Unchecked stuff
impl<'a, T: SharedAnyToOne<'a, RealK, RealV>, RealK: IdLike, RealV: IdLike> DynToOne<'a> for ConcreteToOne<'a, T, RealK, RealV> {
    /*
    type Iter = impl 'a+DoubleEndedIterator<Item=(K<'a>, V<'a>)>;
    type Keys = impl 'a+DoubleEndedIterator<Item=K<'a>>;
    type Values = impl 'a+DoubleEndedIterator<Item=V<'a>>;
    */

    const KSIZE: usize = size_of::<RealK>();
    const VSIZE: usize = size_of::<RealV>();

    fn get(&self, k: K<'a>, out: VMut<'a>) -> bool {
        assert_eq!(k.len(), Self::KSIZE);
        assert_eq!(out.len(), Self::VSIZE);
        let k_mono: &RealK = unsafe { std::mem::transmute(&k[0]) };
        let out_mono: &mut RealV = unsafe { std::mem::transmute(&mut out[0]) };

        match self.0.get(*k_mono) {
            Some(v) => { *out_mono = v; return true; }
            None => { return false; }
        }
    }

    fn contains_key(&self, k: K<'a>) -> bool {
        assert_eq!(k.len(), Self::KSIZE);
        let k_mono: &RealK = unsafe { std::mem::transmute(&k[0]) };

        self.0.contains_key(*k_mono)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn contains(&self, k: K<'a>, v: V<'a>) -> bool {
        assert_eq!(k.len(), Self::KSIZE);
        assert_eq!(v.len(), Self::VSIZE);
        let k_mono: &RealK = unsafe { std::mem::transmute(&k[0]) };
        let v_mono: &RealV = unsafe { std::mem::transmute(&v[0]) };

        self.0.contains(*k_mono, *v_mono)
    }

    /*
    fn iter(&self) -> Self::Iter {
        todo!()
    }

    fn keys(&self) -> Self::Keys {
        todo!()
    }

    fn values(&self) -> Self::Values {
        todo!()
    }
    */

    fn insert(&self, k: K<'a>, v: V<'a>) -> bool {
        assert_eq!(k.len(), Self::KSIZE);
        assert_eq!(v.len(), Self::VSIZE);
        let k_mono: &RealK = unsafe { std::mem::transmute(&k[0]) };
        let v_mono: &RealV = unsafe { std::mem::transmute(&v[0]) };

        self.0.insert(*k_mono, *v_mono).is_none()  // true if new, false if old
    }

    fn expunge(&self, k: K<'a>) -> bool {
        assert_eq!(k.len(), Self::KSIZE);
        let k_mono: &RealK = unsafe { std::mem::transmute(&k[0]) };

        self.0.expunge(*k_mono).is_some()  // true if something was expunged
    }

    fn remove(&self, k: K<'a>, v: V<'a>, out: VMut<'a>) -> bool {
        assert_eq!(k.len(), Self::KSIZE);
        assert_eq!(v.len(), Self::VSIZE);
        assert_eq!(out.len(), Self::VSIZE);
        let k_mono: &RealK = unsafe { std::mem::transmute(&k[0]) };
        let v_mono: &RealV = unsafe { std::mem::transmute(&v[0]) };
        let out_mono: &mut RealV = unsafe { std::mem::transmute(&mut out[0]) };

        match self.0.remove(*k_mono, *v_mono) {
            Some(v) => { *out_mono = v; return true; }
            None => { return false; }
        }
    }

}