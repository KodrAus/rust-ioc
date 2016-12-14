use std::cell::RefCell;
use typemap::{TypeMap, Key};

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

pub trait Scope {
    type Container: ScopedContainer;

    fn scope<F>(&self, f: F) where F: FnOnce(Self::Container) -> ();
}

pub trait ScopedContainer where Self: Container {
    fn get_or_add<'a, T, D>(&'a self) -> &'a T
        where T: Resolvable<Self, Dependency = D> + 'static,
              D: ResolvableFromContainer<'a, Self>;
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

impl<'a, C, T, D> ResolvableFromContainer<'a, C> for B<'a, T>
    where C: ScopedContainer,
          T: Resolvable<C, Dependency = D> + 'static,
          D: ResolvableFromContainer<'a, C>
{
    fn resolve_from_container(container: &'a C) -> Self {
        B { t: container.get_or_add() }
    }
}

/// A basic implementation of a container.
#[derive(Default)]
pub struct BasicContainer;

impl Container for BasicContainer { }

impl Scope for BasicContainer {
    type Container = BasicScopedContainer;

    fn scope<F>(&self, f: F) where F: FnOnce(Self::Container) -> () {
        let scope = BasicScopedContainer::new();

        f(scope);
    }
}

/// A type map key for dependencies.
struct K<T> {
    _t: ::std::marker::PhantomData<T>
}

impl <T: 'static> Key for K<T> {
    type Value = RefCell<T>;
}

/// A basic implementation of a scoped container.
pub struct BasicScopedContainer {
    map: RefCell<TypeMap>,
}

impl BasicScopedContainer {
    fn new() -> Self {
        BasicScopedContainer {
            map: RefCell::new(TypeMap::new()),
        }
    }

    fn exists<T>(&self) -> bool where T: 'static {
        self.map.borrow().get::<K<T>>().is_some()
    }

    fn get<T>(&self) -> *mut T where T: 'static {
        self.map.borrow().get::<K<T>>().unwrap().as_ptr()
    }

    fn add<T>(&self, t: T) where T: 'static {
        self.map.borrow_mut().insert::<K<T>>(RefCell::new(t));
    }
}

impl Container for BasicScopedContainer { }

impl ScopedContainer for BasicScopedContainer {
    fn get_or_add<'a, T, D>(&'a self) -> &'a T
    where T: Resolvable<Self, Dependency = D> + 'static,
          D: ResolvableFromContainer<'a, Self> 
    {
        if self.exists::<K<T>>() {
            unsafe {
                self.get().as_ref().unwrap()
            }
        }
        else {
            let d = D::resolve_from_container(self);
            let t = T::resolve(d);

            self.add(t);

            unsafe {
                self.get().as_ref().unwrap()
            }
        }
    }
}