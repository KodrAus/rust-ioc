use std::any::{TypeId, Any};
use std::cell::RefCell;
use std::collections::HashMap;

/// A container that can resolve dependencies.
pub trait Container
    where Self: Sized
{
    fn resolve<'brw, D, R>(&'brw self) -> R
        where R: Resolvable<Self, Dependency = D>,
              D: ResolvableFromContainer<'brw, Self>
    {
        let d = D::resolve_from_container(self);

        R::resolve(d)
    }
}

/// A trait for creating a new scope and using it within a closure.
pub trait Scope<'scope> {
    type Container: ScopedContainer<'scope>;

    fn scope<F>(&self, f: F) where F: FnOnce(Self::Container) -> ();
}

/// A container that can can resolve dependencies for a given lifetime.
pub trait ScopedContainer<'scope>
    where Self: Container
{
    fn get_or_add<'brw, T, D>(&'brw self) -> &'scope T
        where 'scope: 'brw,
              T: Resolvable<Self, Dependency = D>,
              D: ResolvableFromContainer<'brw, Self>;
}

/// A dependency that can be resolved directly from the container.
///
/// This trait is different from `Resolvable` because it doesn't declare
/// the type of the dependency the implementor requires.
pub trait ResolvableFromContainer<'brw, C>
    where C: Container
{
    fn resolve_from_container(container: &'brw C) -> Self;
}

/// A dependency that can be resolved.
pub trait Resolvable<C> {
    type Dependency;

    fn resolve(dependency: Self::Dependency) -> Self;
}

/// `()` is a root dependency that has no dependencies of its own.
impl<'brw, C> ResolvableFromContainer<'brw, C> for ()
    where C: Container
{
    fn resolve_from_container(_: &'brw C) -> Self {
        ()
    }
}

/// Tuples are root dependencies that are constructed from the dependencies
/// of their members.
macro_rules! resolve_tuple {
    ($(($T:ident,$D:ident,$d:ident))*) => (
        impl <'brw, C $(,$T)*> ResolvableFromContainer<'brw, C> for ($($T,)*)
            where $($T: ResolvableFromContainer<'brw, C>,)*
                  C: Container
        {
            fn resolve_from_container(container: &'brw C) -> Self {
                (
                    $($T::resolve_from_container(container),)*
                )
            }
        }

        impl <'brw, C $(,$T,$D)*> Resolvable<C> for ($($T,)*)
            where $($T: Resolvable<C, Dependency = $D>, $D: ResolvableFromContainer<'brw, C>,)*
                  C: Container
        {
            type Dependency = ($($D,)*);

            fn resolve(($($d,)*): Self::Dependency) -> Self {
                (
                    $($T::resolve($d),)*
                )
            }
        }
    )
}

resolve_tuple!((T1, D1, d1)(T2, D2, d2));
resolve_tuple!((T1, D1, d1)(T2, D2, d2)(T3, D3, d3));
resolve_tuple!((T1, D1, d1)(T2, D2, d2)(T3, D3, d3)(T4, D4, d4));
resolve_tuple!((T1, D1, d1)(T2, D2, d2)(T3, D3, d3)(T4, D4, d4)(T5, D5, d5));

/// A root dependency that wraps some other owned dependency type.
///
/// `O` makes it possible to implement `ResolvableFromContainer` for any
/// arbitrary `Resolvable` without colliding with other root dependency
/// implementations.
pub struct O<T> {
    t: T,
}

impl<T> O<T> {
    pub fn value(self) -> T {
        self.t
    }
}

impl<'brw, C, T, D> Resolvable<C> for O<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<'brw, C>
{
    type Dependency = D;

    fn resolve(dependency: D) -> Self {
        O { t: T::resolve(dependency) }
    }
}

impl<'brw, C, T, D> ResolvableFromContainer<'brw, C> for O<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<'brw, C>
{
    fn resolve_from_container(container: &'brw C) -> Self {
        let d = D::resolve_from_container(container);
        O { t: T::resolve(d) }
    }
}

/// A root dependency that wraps some other borrowed dependency type.
pub struct B<'scope, T: 'scope> {
    t: &'scope T,
}

impl<'scope, T> B<'scope, T> {
    pub fn value(self) -> &'scope T {
        self.t
    }
}

impl<'scope, 'brw, C, T, D> ResolvableFromContainer<'brw, C> for B<'scope, T>
    where 'scope: 'brw,
          C: ScopedContainer<'scope>,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<'brw, C>
{
    fn resolve_from_container(container: &'brw C) -> Self {
        B { t: container.get_or_add() }
    }
}

/// A basic implementation of a container.
#[derive(Default)]
pub struct BasicContainer;

impl Container for BasicContainer {}

impl<'scope> Scope<'scope> for BasicContainer {
    type Container = BasicScopedContainer<'scope>;

    fn scope<F>(&self, f: F)
        where F: FnOnce(Self::Container) -> ()
    {
        let scope = BasicScopedContainer::new();

        f(scope);
    }
}

struct TypeMap<'scope> {
    refs: HashMap<TypeId, Box<Any + 'scope>>,
}

impl<'scope> TypeMap<'scope> {
    pub fn new() -> Self {
        TypeMap { refs: HashMap::new() }
    }

    fn key<T>() -> TypeId {
        TypeId::of::<T>()
    }

    fn exists<T>(&self) -> bool {
        self.refs.get(&Self::key::<T>()).is_some()
    }

    unsafe fn get<T>(&self) -> *const T {
        &**self.refs.get(&Self::key::<T>()).unwrap() as *const Any as *const T
    }

    fn insert<T>(&mut self, t: T)
        where T: 'scope
    {
        self.refs.insert(Self::key::<T>(), Box::new(t));
    }
}

/// A basic implementation of a scoped container.
pub struct BasicScopedContainer<'scope> {
    map: RefCell<TypeMap<'scope>>,
}

impl<'scope> BasicScopedContainer<'scope> {
    fn new() -> Self {
        BasicScopedContainer { map: RefCell::new(TypeMap::new()) }
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

impl<'scope> Container for BasicScopedContainer<'scope> {}

impl<'scope> ScopedContainer<'scope> for BasicScopedContainer<'scope> {
    fn get_or_add<'brw, T, D>(&'brw self) -> &'scope T
        where 'scope: 'brw,
              T: Resolvable<Self, Dependency = D>,
              D: ResolvableFromContainer<'brw, Self>
    {
        if !self.exists::<T>() {
            let d = D::resolve_from_container(self);
            let t = T::resolve(d);

            self.add(t);
        }

        unsafe { self.get().as_ref().unwrap() }
    }
}
