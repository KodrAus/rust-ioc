// NOTE: This depends on a tweaked `TypeId` that doesn't require `T: 'static`

use super::*;

use std::intrinsics;
use std::cell::RefCell;
use std::collections::HashMap as StdHashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

type HashMap<K, V> = StdHashMap<K, V, BuildHasherDefault<FnvHasher>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TypeId(u64);

impl TypeId {
    fn of<T: ?Sized>() -> Self {
        TypeId(unsafe { intrinsics::type_id::<T>() })
    }
}

pub trait Any {}
impl<T: ?Sized> Any for T {}

struct TypeMap<'scope> {
    refs: HashMap<TypeId, Box<Any + 'scope>>,
}

impl<'scope> TypeMap<'scope> {
    pub fn new() -> Self {
        TypeMap { refs: HashMap::default() }
    }

    fn key<T>() -> TypeId {
        TypeId::of::<T>()
    }

    fn exists<T>(&self) -> bool {
        self.refs.get(&Self::key::<T>()).is_some()
    }

    unsafe fn get<T>(&self) -> *const T {
        let ptr = self.refs.get(&Self::key::<T>()).unwrap();

        &**ptr as *const Any as *const T
    }

    fn insert<T>(&mut self, t: T)
        where T: 'scope
    {
        self.refs.insert(Self::key::<T>(), Box::new(t));
    }
}

/// A basic implementation of a scoped container.
pub struct Scoped<'scope> {
    map: RefCell<TypeMap<'scope>>,
}

impl<'scope> Scoped<'scope> {
    pub fn new() -> Self {
        Scoped { map: RefCell::new(TypeMap::new()) }
    }

    #[inline]
    fn exists<T>(&self) -> bool {
        self.map.borrow().exists::<T>()
    }

    #[inline]
    unsafe fn get<T>(&self) -> *const T {
        self.map.borrow().get::<T>()
    }

    #[inline]
    fn add<T>(&self, t: T)
        where T: 'scope
    {
        self.map.borrow_mut().insert::<T>(t);
    }
}

impl<'scope> Container for Scoped<'scope> {}

impl<'scope> ScopedContainer for Scoped<'scope> {
    fn get_or_add<T, D>(&self) -> T
        where T: Resolvable<Self, Dependency = D> + Clone + 'static,
              D: ResolvableFromContainer<Self>
    {
        if !self.exists::<T>() {
            let d = D::resolve_from_container(self);
            let t = T::resolve(d);

            self.add(t);
        }

        unsafe { (*self.get::<T>()).clone() }
    }
}

impl<'scope> BrwScopedContainer<'scope> for Scoped<'scope> {
    fn brw_or_add<'brw, T, D>(&self) -> &'brw T
        where 'scope: 'brw,
              T: Resolvable<Self, Dependency = D> + 'scope,
              D: ResolvableFromContainer<Self>
    {
        if !self.exists::<T>() {
            let d = D::resolve_from_container(self);
            let t = T::resolve(d);

            self.add(t);
        }

        unsafe { self.get().as_ref().unwrap() }
    }
}
