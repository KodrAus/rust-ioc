//! Root dependency implementations
//! 
//! Root dependencies include:
//! 
//! - `()` the only _true_ root dependency that can be used for types
//! that can be materialised from nothing.
//! - `Owned<T>` a unique instance of `T`.
//! - `Rc<T>` a shared instance of `T`.
//! - `RefCell<T>` a mutable instance of `T`.
//! 
//! These can be combined in various ways, like `Rc<RefCell<T>>`.
//! The trait bounds might be tightened up though, since `Rc<Owned<T>>`,
//! `Owned<()>` and `RefCell<T>` alone don't make much sense.

use std::rc::Rc;
use std::cell::RefCell;
use super::*;

/// `()` is a root dependency that has no dependencies of its own.
impl<C> ResolvableFromContainer<C> for ()
    where C: Container
{
    fn resolve_from_container(_: &C) -> Self {
        ()
    }
}

/// Tuples are root dependencies that are constructed from the dependencies
/// of their members.
macro_rules! resolve_tuple {
    ($(($T:ident,$D:ident,$d:ident))*) => (
        impl <C $(,$T)*> ResolvableFromContainer<C> for ($($T,)*)
            where $($T: ResolvableFromContainer<C>,)*
                  C: Container
        {
            fn resolve_from_container(container: &C) -> Self {
                (
                    $($T::resolve_from_container(container),)*
                )
            }
        }

        impl <C $(,$T,$D)*> Resolvable<C> for ($($T,)*)
            where $($T: Resolvable<C, Dependency = $D>, $D: ResolvableFromContainer<C>,)*
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
/// `Owned` makes it possible to implement `ResolvableFromContainer` for any
/// arbitrary `Resolvable` without colliding with other root dependency
/// implementations.
pub struct Owned<T> {
    t: T,
}

impl<T> Owned<T> {
    pub fn value(self) -> T {
        self.t
    }
}

impl<C, T, D> Resolvable<C> for Owned<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<C>
{
    type Dependency = D;

    fn resolve(dependency: D) -> Self {
        Owned { t: T::resolve(dependency) }
    }
}

impl<C, T, D> ResolvableFromContainer<C> for Owned<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<C>
{
    fn resolve_from_container(container: &C) -> Self {
        let d = D::resolve_from_container(container);
        Owned { t: T::resolve(d) }
    }
}

impl<C, T, D> Resolvable<C> for RefCell<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<C>
{
    type Dependency = D;

    fn resolve(dependency: D) -> Self {
        RefCell::new(T::resolve(dependency))
    }
}

impl<C, T, D> ResolvableFromContainer<C> for RefCell<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<C>
{
    fn resolve_from_container(container: &C) -> Self {
        let d = D::resolve_from_container(container);
        RefCell::new(T::resolve(d))
    }
}

impl<C, T, D> ResolvableFromContainer<C> for Rc<T>
    where C: ScopedContainer,
          T: Resolvable<C, Dependency = D> + 'static,
          D: ResolvableFromContainer<C>
{
    fn resolve_from_container(container: &C) -> Self {
        container.get_or_add()
    }
}
