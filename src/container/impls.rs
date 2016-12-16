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

impl<C, T, D> Resolvable<C> for O<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<C>
{
    type Dependency = D;

    fn resolve(dependency: D) -> Self {
        O { t: T::resolve(dependency) }
    }
}

impl<C, T, D> ResolvableFromContainer<C> for O<T>
    where C: Container,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<C>
{
    fn resolve_from_container(container: &C) -> Self {
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

impl<'scope, C, T, D> ResolvableFromContainer<C> for B<'scope, T>
    where C: BrwScopedContainer<'scope>,
          T: Resolvable<C, Dependency = D>,
          D: ResolvableFromContainer<C>
{
    fn resolve_from_container(container: &C) -> Self {
        B { t: container.brw_or_add() }
    }
}
