// NOTE: This depends on a tweaked `TypeId` that doesn't require `T: 'static`

use super::*;

use std::mem;
use std::any::{Any, TypeId};
use std::intrinsics;
use std::cell::RefCell;
use std::collections::HashMap as StdHashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

type HashMap<K, V> = StdHashMap<K, V, BuildHasherDefault<FnvHasher>>;

struct TypeMap {
    refs: HashMap<TypeId, Rc<Box<Any>>>,
}

impl TypeMap {
    pub fn new() -> Self {
        TypeMap { refs: HashMap::default() }
    }

    fn key<T>() -> TypeId
        where T: 'static
    {
        TypeId::of::<T>()
    }

    fn exists<T>(&self) -> bool
        where T: 'static
    {
        self.refs.get(&Self::key::<T>()).is_some()
    }

    unsafe fn get<T>(&self) -> Rc<Box<T>>
        where T: 'static
    {
        let rc = self.refs.get(&Self::key::<T>()).unwrap().clone();
        
        mem::transmute(rc)
    }

    fn insert<T>(&mut self, t: T)
        where T: 'static
    {
        let rc: Rc<Box<Any>> = Rc::new(Box::new(t));
        self.refs.insert(Self::key::<T>(), rc);
    }
}

/// A basic implementation of a scoped container.
pub struct Scoped {
    map: RefCell<TypeMap>,
}

impl Scoped {
    pub fn new() -> Self {
        Scoped { map: RefCell::new(TypeMap::new()) }
    }

    #[inline]
    fn exists<T>(&self) -> bool
        where T: 'static
    {
        self.map.borrow().exists::<T>()
    }

    #[inline]
    unsafe fn get<T>(&self) -> Rc<Box<T>>
        where T: 'static
    {
        self.map.borrow().get::<T>()
    }

    #[inline]
    fn add<T>(&self, t: T)
        where T: 'static
    {
        self.map.borrow_mut().insert::<T>(t);
    }
}

impl Container for Scoped {}

// NOTE: the 'brw here probably isn't doing much, since the T
// to resolve needs to live for 'scope anyway
impl ScopedContainer for Scoped {
    fn get_or_add<T, D>(&self) -> Rc<Box<T>>
        where T: Resolvable<Self, Dependency = D> + 'static,
              D: ResolvableFromContainer<Self>
    {
        if !self.exists::<T>() {
            let d = D::resolve_from_container(self);
            let t = T::resolve(d);

            self.add(t);
        }

        unsafe { self.get() }
    }
}
