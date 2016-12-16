use std::any::{TypeId,Any};
use std::cell::RefCell;
use std::collections::HashMap;

/// A container that can resolve dependencies.
pub trait Container
    where Self: Sized
{
    fn resolve<'a, D, R>(&'a self) -> R
        where R: Resolvable<Self, Dependency = D>,
              D: ResolvableFromContainer<'a, Self>
    {
        let d = D::resolve_from_container(self);

        R::resolve(d)
    }
}

/// A trait for creating a new scope and using it within a closure.
pub trait Scope<'a> {
    type Container: ScopedContainer<'a>;

    fn scope<F>(&self, f: F) where F: FnOnce(Self::Container) -> ();
}

/// A container that can can resolve dependencies for a given lifetime.
pub trait ScopedContainer<'a>
    where Self: Container
{
    fn get_or_add<'b, T, D>(&'b self) -> &'a T
        where 'a: 'b,
              T: Resolvable<Self, Dependency = D>,
              D: ResolvableFromContainer<'b, Self>;
}

/// A dependency that can be resolved directly from the container.
///
/// This trait is different from `Resolvable` because it doesn't declare
/// the type of the dependency the implementor requires.
pub trait ResolvableFromContainer<'a, C>
    where C: Container
{
    fn resolve_from_container(container: &'a C) -> Self;
}

/// A dependency that can be resolved.
pub trait Resolvable<C> {
    type Dependency;

    fn resolve(dependency: Self::Dependency) -> Self;
}

/// `()` is a root dependency that has no dependencies of its own.
impl<'a, C> ResolvableFromContainer<'a, C> for ()
    where C: Container
{
    fn resolve_from_container(_: &'a C) -> Self {
        ()
    }
}

/// Tuples are root dependencies that are constructed from the dependencies
/// of their members.
macro_rules! resolve_tuple {
    ($(($T:ident,$D:ident,$d:ident))*) => (
        impl <'a, C $(,$T)*> ResolvableFromContainer<'a, C> for ($($T,)*)
            where $($T: ResolvableFromContainer<'a, C>,)*
                  C: Container
        {
            fn resolve_from_container(container: &'a C) -> Self {
                (
                    $($T::resolve_from_container(container),)*
                )
            }
        }

        impl <'a, C $(,$T,$D)*> Resolvable<C> for ($($T,)*)
            where $($T: Resolvable<C, Dependency = $D>, $D: ResolvableFromContainer<'a, C>,)*
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

impl<'a, C, T, D> Resolvable<C> for O<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<'a, C>
{
    type Dependency = D;

    fn resolve(dependency: D) -> Self {
        O { t: T::resolve(dependency) }
    }
}

impl<'a, C, T, D> ResolvableFromContainer<'a, C> for O<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<'a, C>
{
    fn resolve_from_container(container: &'a C) -> Self {
        let d = D::resolve_from_container(container);
        O { t: T::resolve(d) }
    }
}

/// A root dependency that wraps some other borrowed dependency type.
pub struct B<'a, T: 'a> {
    t: &'a T,
}

impl<'a, T> B<'a, T> {
    pub fn value(self) -> &'a T {
        self.t
    }
}

impl<'a, 'b, C, T, D> ResolvableFromContainer<'b, C> for B<'a, T>
    where 'a: 'b,
          C: ScopedContainer<'a>,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<'b, C>
{
    fn resolve_from_container(container: &'b C) -> Self {
        B { t: container.get_or_add() }
    }
}

/// A basic implementation of a container.
#[derive(Default)]
pub struct BasicContainer;

impl Container for BasicContainer {}

impl <'a> Scope<'a> for BasicContainer {
    // NOTE: With ATC for lifetimes this would work for BasicScopedContainer<'a>
    type Container = BasicScopedContainer<'a>;

    fn scope<F>(&self, f: F)
        where F: FnOnce(Self::Container) -> ()
    {
        let scope = BasicScopedContainer::new();

        f(scope);
    }
}

struct TypeMap<'a> {
    refs: HashMap<TypeId, Box<Any + 'a>>
}

impl <'a> TypeMap<'a> {
    pub fn new() -> Self {
        TypeMap {
            refs: HashMap::new()
        }
    }

    fn key<T>() -> TypeId {
        TypeId::of::<T>()
    }

    fn exists<T>(&self) -> bool {
        self.refs.get(&Self::key::<T>()).is_some()
    }

    unsafe fn get_raw<T>(&self) -> *const T {
        (&**self.refs.get(&Self::key::<T>()).unwrap()) as *const Any as *const T
    }

    fn insert<T>(&mut self, t: T) where T: 'a {
        let k = Self::key::<T>();

        self.refs.insert(k, Box::new(t));
    }
}

/// A basic implementation of a scoped container.
pub struct BasicScopedContainer<'a> {
    map: RefCell<TypeMap<'a>>,
}

impl <'a> BasicScopedContainer<'a> {
    fn new() -> Self {
        BasicScopedContainer { map: RefCell::new(TypeMap::new()) }
    }

    #[inline]
    fn exists<T>(&self) -> bool
    {
        self.map.borrow().exists::<T>()
    }

    #[inline]
    unsafe fn get<T>(&self) -> *const T
    {
        self.map.borrow().get_raw::<T>()
    }

    #[inline]
    fn add<T>(&self, t: T) where T: 'a
    {
        self.map.borrow_mut().insert::<T>(t);
    }
}

impl <'a> Container for BasicScopedContainer<'a> {}

impl <'a> ScopedContainer<'a> for BasicScopedContainer<'a> {
    fn get_or_add<'b, T, D>(&'b self) -> &'a T
        where 'a: 'b,
              T: Resolvable<Self, Dependency = D>,
              D: ResolvableFromContainer<'b, Self>
    {
        if !self.exists::<T>() {
            let d = D::resolve_from_container(self);
            let t = T::resolve(d);

            self.add(t);
        }

        unsafe { self.get().as_ref().unwrap() }
    }
}
