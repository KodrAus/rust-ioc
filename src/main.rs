extern crate typemap;

mod container;
use container::*;

#[derive(Debug)]
struct X;
impl<C> Resolvable<C> for X {
    type Dependency = ();

    fn resolve(_: Self::Dependency) -> Self {
        X
    }
}

#[derive(Debug)]
struct Y {
    x: X,
}
impl<C> Resolvable<C> for Y {
    type Dependency = O<X>;

    fn resolve(x: Self::Dependency) -> Self {
        Y { x: x.value() }
    }
}

#[derive(Debug)]
struct Z {
    x: X,
    y: Y,
}
impl<C> Resolvable<C> for Z {
    type Dependency = (O<X>, O<Y>);

    fn resolve((x, y): Self::Dependency) -> Self {
        Z {
            x: x.value(),
            y: y.value(),
        }
    }
}

#[derive(Debug)]
struct XYZ {
    x: X,
    y: Y,
    z: Z,
}
impl<C> Resolvable<C> for XYZ {
    // NOTE: `(O<X>, (O<Y>, O<Z>))` would also work
    type Dependency = (O<X>, O<Y>, O<Z>);

    fn resolve((x, y, z): Self::Dependency) -> Self {
        XYZ {
            x: x.value(),
            y: y.value(),
            z: z.value(),
        }
    }
}

#[derive(Debug)]
struct XorY<T> {
    t: T,
}
impl<C, T> Resolvable<C> for XorY<T> {
    type Dependency = O<T>;

    fn resolve(t: Self::Dependency) -> Self {
        XorY { t: t.value() }
    }
}

#[derive(Debug)]
struct BorrowY<'a> {
    x: X,
    y: &'a Y,
}
impl<'a, C> Resolvable<C> for BorrowY<'a> {
    type Dependency = (O<X>, B<'a, Y>);

    fn resolve((x, y): Self::Dependency) -> Self {
        BorrowY {
            x: x.value(),
            y: y.value(),
        }
    }
}

#[derive(Debug)]
struct BorrowMoreY<'a> {
    y: &'a BorrowY<'a>,
}
impl<'a, C> Resolvable<C> for BorrowMoreY<'a> {
    type Dependency = B<'a, BorrowY<'a>>;

    fn resolve(y: Self::Dependency) -> Self {
        BorrowMoreY { y: y.value() }
    }
}

fn main() {
    // A basic container for only owned resources.
    let c = BasicContainer;

    let x: X = BasicContainer.resolve();
    let xy: (X, Y) = c.resolve();
    let z: Z = c.resolve();
    let xyz: XYZ = c.resolve();

    let xory_x: XorY<X> = c.resolve();
    let xory_y: XorY<Y> = c.resolve();

    println!("{:?}", x);
    println!("{:?}", xy);
    println!("{:?}", z);
    println!("{:?}", xyz);
    println!("{:?}", xory_x);
    println!("{:?}", xory_y);

    // Create a scope that can be used to resolve references.
    // Each B<'a, T> dependency will be the same instance for the lifetime of the scope.
    c.scope(|scope| {
        let y: BorrowY = scope.resolve();
        let z: Z = scope.resolve();

        // NOTE: The typemap requirement for T: 'static kills this
        // let y: BorrowMoreY = scope.resolve();

        println!("{:?}", y);
        println!("{:?}", z);
    });
}
