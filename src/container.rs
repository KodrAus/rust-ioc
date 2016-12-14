// TODO: Remove need to specify `D<T>` in `Resolvable`

/// A container that can resolve dependencies.
pub trait Container where Self: Sized {
	fn resolve<D, R>(&self) -> R 
		where R: Resolvable<Self, Dependency = D>,
			  D: ResolvableFromContainer<Self>;
}

/// A dependency that can be resolved directly from the container.
/// 
/// This trait is different from `Resolvable` because it doesn't declare
/// the type of the dependency the implementor requires.
pub trait ResolvableFromContainer<C> where C: Container {
	fn resolve_from_container(container: &C) -> Self;
}

/// A dependency that can be resolved.
pub trait Resolvable<C> {
	type Dependency;

	fn resolve(dependency: Self::Dependency) -> Self;
}

/// `()` is a root dependency that has no dependencies of its own.
impl <C> ResolvableFromContainer<C> for () where C: Container {
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
					$($T::resolve_from_container(&container),)*
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

resolve_tuple!((T1, D1, d1) (T2, D2, d2));
resolve_tuple!((T1, D1, d1) (T2, D2, d2) (T3, D3, d3));
resolve_tuple!((T1, D1, d1) (T2, D2, d2) (T3, D3, d3) (T4, D4, d4));
resolve_tuple!((T1, D1, d1) (T2, D2, d2) (T3, D3, d3) (T4, D4, d4) (T5, D5, d5));

/// A root dependency that wraps some other dependency type.
/// 
/// `D` makes it possible to implement `ResolvableFromContainer` for any
/// arbitrary `Resolvable` without colliding with other root dependency
/// implementations.
pub struct D<T> {
	t: T
}

impl <T> D<T> {
	pub fn value(self) -> T {
		self.t
	}
}

impl <C, T, D_> Resolvable<C> for D<T>
	where C: Container,
		  T: Resolvable<C, Dependency = D_>,
		  D_: ResolvableFromContainer<C>
	{
		type Dependency = D_;

		fn resolve(dependency: D_) -> Self {
			D { t: T::resolve(dependency) }
		}
}

impl <C, T, D_> ResolvableFromContainer<C> for D<T>
	where C: Container,
		  T: Resolvable<C, Dependency = D_>,
		  D_: ResolvableFromContainer<C>
	{
		fn resolve_from_container(container: &C) -> Self {
			let d = D_::resolve_from_container(&container);
			D { t: T::resolve(d) }
		}
}

/// A basic implementation of a container.
#[derive(Clone, Copy)]
pub struct BasicContainer;
impl Container for BasicContainer {
	fn resolve<D, R>(&self) -> R 
		where R: Resolvable<Self, Dependency = D>,
			  D: ResolvableFromContainer<Self>	
	{
		let d = D::resolve_from_container(&self);

		R::resolve(d)
	}
}