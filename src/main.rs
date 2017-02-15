#![feature(rc_raw)]

extern crate fnv;

mod container;
use container::*;

use std::rc::Rc;
use std::cell::RefCell;

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
    i: i32
}
impl<C> Resolvable<C> for Y {
    type Dependency = RefCell<X>;

    fn resolve(x: Self::Dependency) -> Self {
        Y { x: x.into_inner(), i: 1 }
    }
}

#[derive(Debug)]
struct Z {
    x: X,
    y: Y,
}
impl<C> Resolvable<C> for Z {
    type Dependency = (RefCell<X>, RefCell<Y>);

    fn resolve((x, y): Self::Dependency) -> Self {
        Z {
            x: x.into_inner(),
            y: y.into_inner(),
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
    // NOTE: `(RefCell<X>, (RefCell<Y>, RefCell<Z>))` would also work
    type Dependency = (RefCell<X>, RefCell<Y>, RefCell<Z>);

    fn resolve((x, y, z): Self::Dependency) -> Self {
        XYZ {
            x: x.into_inner(),
            y: y.into_inner(),
            z: z.into_inner(),
        }
    }
}

#[derive(Debug)]
struct XorY<T> {
    t: T,
}
impl<C, T> Resolvable<C> for XorY<T> {
    type Dependency = RefCell<T>;

    fn resolve(t: Self::Dependency) -> Self {
        XorY { t: t.into_inner() }
    }
}

#[derive(Debug)]
struct BorrowY {
    x: X,
    y: Rc<Y>,
    k: &'static str
}
impl<C> Resolvable<C> for BorrowY {
    type Dependency = (RefCell<X>, Rc<Y>);

    fn resolve((x, y): Self::Dependency) -> Self {
        BorrowY {
            x: x.into_inner(),
            y: y,
            k: "some string value"
        }
    }
}

#[derive(Debug)]
struct BorrowMoreY {
    y: Rc<BorrowY>,
}
impl<C> Resolvable<C> for BorrowMoreY {
    type Dependency = Rc<BorrowY>;

    fn resolve(y: Self::Dependency) -> Self {
        BorrowMoreY { y: y }
    }
}

#[derive(Debug)]
struct BorrowAndMutateY {
    y: Rc<RefCell<Y>>
}
impl<C> Resolvable<C> for BorrowAndMutateY {
    type Dependency = Rc<RefCell<Y>>;

    fn resolve(y: Self::Dependency) -> Self {
        BorrowAndMutateY { y: y }
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

        {
            let y: BorrowMoreY = scope.resolve();

            println!("y count: {}", Rc::strong_count(&y.y));
        }
        {
            let y: BorrowMoreY = scope.resolve();

            println!("{:?}", y);
            println!("y count: {}", Rc::strong_count(&y.y));
        }
        {
            let y: BorrowAndMutateY = scope.resolve();

            let mut iy = y.y.borrow_mut();

            iy.i += 1;

            println!("{:?}", y);
            println!("y.i: {}", iy.i);
        }
        {
            let y: BorrowAndMutateY = scope.resolve();

            let mut iy = y.y.borrow_mut();

            iy.i += 1;

            println!("y.i: {}", iy.i);
        }

        println!("{:?}", z);
    });
}
