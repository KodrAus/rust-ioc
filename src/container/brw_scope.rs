use super::*;

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap as StdHashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

type HashMap<K, V> = StdHashMap<K, V, BuildHasherDefault<FnvHasher>>;
type DropHandle = Box<Fn(*mut Any) -> ()>;

struct TypeMap {
    refs: HashMap<TypeId, (*mut Any, DropHandle)>,
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

    /// Get a shared reference to a dependency.
    /// 
    /// This will increment the reference count.
    /// It will panic if the dependency doesn't already exist so
    /// call `exists` first, and `insert` if it's not found.
    unsafe fn get<T>(&self) -> Rc<T>
        where T: 'static
    {
        let &(ptr, _) = self.refs.get(&Self::key::<T>()).unwrap();

        let rc = Rc::from_raw(ptr as *mut T);
        let rc_clone = rc.clone();

        // forget this Rc again (don't decrement count)
        Rc::into_raw(rc);

        rc_clone
    }

    /// Insert a dependency into the map.
    fn insert<T>(&mut self, t: T)
        where T: 'static
    {
        let ptr = Rc::into_raw(Rc::new(t));

        // a function to drop this Rc
        let drop = Box::new(|ptr| unsafe {
            Rc::from_raw(ptr as *mut T);
        });

        // add the dependency, dropping any previous value
        match self.refs.insert(Self::key::<T>(), (ptr, drop)) {
            Some((ptr, drop)) => drop(ptr),
            _ => ()
        }
    }
}

impl Drop for TypeMap {
    fn drop(&mut self) {
        for (_, (ptr, drop)) in self.refs.drain() {
            drop(ptr);
        }
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
    unsafe fn get<T>(&self) -> Rc<T>
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

impl ScopedContainer for Scoped {
    fn get_or_add<T, D>(&self) -> Rc<T>
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
