mod container;
use container::*;

#[derive(Debug)]
struct X;
impl <C> Resolvable<C> for X {
	type Dependency = ();

	fn resolve(_: Self::Dependency) -> Self {
		X
	}
}

#[derive(Debug)]
struct Y {
	x: X,
}
impl <C> Resolvable<C> for Y {
	type Dependency = D<X>;

	fn resolve(x: Self::Dependency) -> Self {
		Y { x: x.value() }
	}
}

#[derive(Debug)]
struct Z {
	x: X,
	y: Y,
}
impl <C> Resolvable<C> for Z {
	type Dependency = (D<X>, D<Y>);

	fn resolve((x, y): Self::Dependency) -> Self {
		Z { x: x.value(), y: y.value() }
	}
}

#[derive(Debug)]
struct XYZ {
	x: X,
	y: Y,
	z: Z
}
impl <C> Resolvable<C> for XYZ {
	// NOTE: `(D<X>, (D<Y>, D<Z>))` would also work
	type Dependency = (D<X>, D<Y>, D<Z>);

	fn resolve((x, y, z): Self::Dependency) -> Self {
		XYZ { x: x.value(), y: y.value(), z: z.value() }
	}
}

fn main() {
	let c = BasicContainer;

	let x: X = c.resolve();
	let xy: (X, Y) = c.resolve();
    let z: Z = c.resolve();
    let xyz: XYZ = c.resolve();

    println!("{:?}", x);
    println!("{:?}", xy);
    println!("{:?}", z);
    println!("{:?}", xyz);
}
