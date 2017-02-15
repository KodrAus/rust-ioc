use super::*;

use std::mem;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap as StdHashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

type HashMap<K, V> = StdHashMap<K, V, BuildHasherDefault<FnvHasher>>;

// TODO: This leaks. We need to find a way to drop values
// efficiently when the scope is dropped.
// Maybe we could use unboxed closures, so we have:
// `HashMap<TypeId, (*mut Any, Fn(*mut Any) -> ())>`.
// Where the `Fn(*mut Any) -> ()` will convert the pointer into a `Rc<T>`
// and drop it.
struct TypeMap {
    refs: HashMap<TypeId, *mut Any>,
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

    unsafe fn get<T>(&self) -> Rc<T>
        where T: 'static
    {
        let ptr = self.refs.get(&Self::key::<T>()).unwrap();
        let rc_ptr = *ptr as *mut T;

        let rc = Rc::from_raw(rc_ptr);
        let rc_clone = rc.clone();

        Rc::into_raw(rc);

        rc_clone
    }

    fn insert<T>(&mut self, t: T)
        where T: 'static
    {
        let rc_ptr = Rc::into_raw(Rc::new(t));
        let ptr = rc_ptr as *mut Any;

        self.refs.insert(Self::key::<T>(), ptr);
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

// NOTE: the 'brw here probably isn't doing much, since the T
// to resolve needs to live for 'scope anyway
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
