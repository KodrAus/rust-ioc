#![feature(test)]

extern crate ioc;
extern crate test;

use ioc::*;
use test::{Bencher,black_box};

struct X;
impl<C> Resolvable<C> for X {
    type Dependency = ();

    fn resolve(_: Self::Dependency) -> Self {
        X
    }
}

#[allow(dead_code)]
struct Y {
    x: X,
}
impl<C> Resolvable<C> for Y {
    type Dependency = O<X>;

    fn resolve(x: Self::Dependency) -> Self {
        Y { x: x.value() }
    }
}

#[bench]
pub fn resolve_owned_y(b: &mut Bencher) {
	let c = BasicContainer;

	b.iter(|| {
		let y: Y = c.resolve();

		black_box(y);
	})
}

#[allow(dead_code)]
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

#[bench]
pub fn resolve_brwd_y_first(b: &mut Bencher) {
	let c = BasicContainer;

	b.iter(|| {
		c.scope(|scope| {
			let y: BorrowY = scope.resolve();

			black_box(y);
		})
	})
}

#[bench]
pub fn resolve_brwd_y_subsequent(b: &mut Bencher) {
	let scope = Scoped::new();

	b.iter(|| {
		let y: BorrowY = scope.resolve();

		black_box(y);
	})
}