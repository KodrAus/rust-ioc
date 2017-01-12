#![feature(core_intrinsics)]

// NOTE: Working with a fork of libcore where TypeId doesn't have to be static

extern crate fnv;

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
struct BorrowY<'scope> {
    x: X,
    y: &'scope Y,
}
impl<'scope, C> Resolvable<C> for BorrowY<'scope> {
    type Dependency = (O<X>, B<'scope, Y>);

    fn resolve((x, y): Self::Dependency) -> Self {
        BorrowY {
            x: x.value(),
            y: y.value(),
        }
    }
}

#[derive(Debug)]
struct BorrowMoreY<'scope> {
    y: &'scope BorrowY<'scope>,
}
impl<'scope, C> Resolvable<C> for BorrowMoreY<'scope> {
    type Dependency = B<'scope, BorrowY<'scope>>;

    fn resolve(y: Self::Dependency) -> Self {
        BorrowMoreY { y: y.value() }
    }
}

#[derive(Debug)]
struct Unsound {
    x: X,
    y: &'static Y,
}
impl<C> Resolvable<C> for Unsound {
    type Dependency = (O<X>, B<'static, Y>);

    fn resolve((x, y): Self::Dependency) -> Self {
        Unsound {
            x: x.value(),
            y: y.value(),
        }
    }
}

fn main() {
    // A basic container for only owned resources.
    let c = BasicContainer;

    let x: X = c.resolve();
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
        let z: Z = scope.resolve();

        let y: BorrowMoreY = scope.resolve();

        // UNSOUND: borrow with a static dependency
        //let u: Unsound = scope.resolve();

        println!("{:?}", y);
        println!("{:?}", z);
    });

    let scope = Scoped::new();

    // UNSOUND: resolve a static dependency
    //let x: &'static X = scope.brw_or_add();
}
