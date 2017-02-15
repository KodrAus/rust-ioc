# Injector Factories for Rust

This is a sandbox for playing around with some dependency injection ideas in the [Rust programming language](https://www.rust-lang-org). Upfront let's not call this _inversion of control_ or _dependency injection_ because it lacks many of the fundamental features of a proper ioc container. What's currently there is a _very_ basic factory pattern that can be used to declare and inject owned or borrowed dependencies without having to know about their dependencies.

## Soundness

The original design used `&T` for borrowed dependencies, but this had a soundness issue that allowed callers to request data that lived longer than the container. I've hacked together a solution of using an `Rc<T>` instead of a straight `&T`. On the surface this seems unfortunate; lifetimes can't help me solve a problem that seems purely about lifetimes. It's not such an issue when you think about it though. Reference counting is a simple and effective mechanism for handling dynamic lifetimes. It means you could also depend on an `Rc<RefCell<T>>` for shared mutable references, which wouldn't be possible with an `&T`.

I think the trait design is fine, and with some attention the boxing of scopes could be made to be better.

## The gist of it

The dependency tree is verified at compile-time, and Rust will helpfully blow up for you if it encounters circular references. All resolution is statically dispatched.

All paths must eventually end with a `()` dependency, so the container doesn't need any additional state to get started. Speaking of the container, here's the definition for the basic implementation:

```rust
struct BasicContainer;
```

You might notice that it contains exactly zero things, so it's not _really_ a container at all. This is possible because of the requirement that all dependency trees must terminate with a `()` at some point. So they can all be built up from scratch on-demand.

## What does this actually do?

Some [examples](https://github.com/KodrAus/rust-ioc/blob/master/src/main.rs).

### Basic factory usage

Say we have a struct `X`, that has no dependencies. We can mark `X` as `Resolvable`, with a dependency on `()`:

```rust
struct X;

impl<C> Resolvable<C> for X {
    type Dependency = ();

    fn resolve(_: Self::Dependency) -> Self { X }
}
```

This dependency can then be resolved from a container using the `resolve` method:

```rust
let x: X = BasicContainer.resolve();

// do something with x
```

Now say we have a struct `Y`, that depends on `X`. We can mark `Y` as `Resolvable`, with a dependency on `X`:

```rust
struct Y {
    x: X,
}

impl<C> Resolvable<C> for Y {
    type Dependency = Owned<X>;

    fn resolve(x: Self::Dependency) -> Self {
        Y { x: x.value() }
    }
}
```

The `Owned<T>` type means an _owned_ dependency.

And a struct `Z` that depends on both `X` and `Y` can be marked as `Resolvable` with a dependency on `(X, Y)`:

```rust
struct Z {
    x: X,
    y: Y,
}

impl<C> Resolvable<C> for Z {
    type Dependency = (Owned<X>, Owned<Y>);

    fn resolve((x, y): Self::Dependency) -> Self {
        Z {
            x: x.value(),
            y: y.value(),
        }
    }
}
```

Note that when specifying `Y` as a dependency we don't need to specify its dependencies again. Tuples are used for encapsulating multiple dependencies in a single type. For ergonomics, tuples of up to 5 elements are supported, but you can technically support `n` dependencies using nested tuples with 2 elements: `(A, (B, (C, D)))`.

### Polymorphism

One of the key benefits of dependency injection is not having to know the concrete type of a dependency. This is where this static approach starts to fall over. Right now, the closest you can get to polymorphic dependencies is using generics:

```rust
struct D<T> {
	t: T
}

impl<C, T> Resolvable<C> for D<T> {
    type Dependency = Owned<T>;

    fn resolve(t: Self::Dependency) -> Self {
        D {
            t: t.value(),
        }
    }
}
```

Then you have the classic issue of generics leaking all over your graph. Anyone depending on `D` would need to supply some `T`. With a bit of thought this could possibly be worked around without going to far down the dynamic rabbit-hole. Maybe some careful use of generics and associated types could be helpful here.

### Borrowed dependencies

You can borrow dependencies wrapped in a standard `Rc<T>` where `T` is the dependency. This is a reference counted, heap allocated dependency, so each dependency will point to the same value for the lifetime of the scope it comes from.

These dependencies are borrowed in much the same way as owned ones:


```rust
struct BorrowY {
	y: Rc<Y>
}

impl<C> Resolvable<C> for BorrowY {
	type Dependency = Rc<Y>;

	fn resolve(y: Self::Dependency) -> Self {
		BorrowY { y: y }
	}
}
```

The `Rc<T>` type means an owned reference to a _borrowed_ dependency, tracking using reference counting. This dependency can then be resolved from a scoped container using the same `resolve` method:

```rust
BasicContainer.scope(|scope| {
	let y: BorrowY = scope.resolve();

	// do something with y
});
```

### (OLD) Borrowed dependencies

> This section is no longer valid, but I'm keeping it around to show what might've been. It's probably worth revisiting this idea in the future with features like Associated Type Constructors to get a bound on the lifetime of borrowed dependencies, without that bound outliving the scope it comes from. I've grown on the `Rc` implementation though, because it gives us possible mutability too.

Dependencies can be borrowed for some lifetime `'a`:

```rust
struct BorrowY<'a> {
	y: &'a Y
}

impl<'a, C> Resolvable<C> for BorrowY<'a> {
	type Dependency = B<'a, Y>;

	fn resolve(y: Self::Dependency) -> Self {
		BorrowY {
			y: y.value()
		}
	}
}
```

The `B` type means a _borrowed_ dependency. This dependency can then be resolved from a scoped container using the same `resolve` method:

```rust
BasicContainer.scope(|scope| {
	let y: BorrowY = scope.resolve();

	// do something with y
});
```

This is where things start to get interesting. Borrowed dependencies use a special container that implements `ScopedContainer`. The `ScopedContainer` has a `TypeMap` of dependencies so it can hand out borrowed references to them.

All dependencies borrowed for the lifetime of a scope will point to the same instance.

## Performance

Everything is static dispatch so optimisations abound. Injecting `Owned<T>` is a _zero-cost abstraction_. For borrowed or scoped dependencies, the cost is in hashing and ref counting. There are 2 heap allocations per shared dependency; the dependency itself and a boxed closure that runs on drop.

I forget this every time so am listing the steps I'm using for benchmarking:

```shell
$ cargo bench --no-run
$ perf record -g target/release/mod-* --bench the_bench_to_run
$ perf script | rust-unmangle | stackcollapse-perf.pl | flamegraph.pl > flame.svg
$ firefox flame.svg
```

## Flaws

A lack of non-leaky polymorphism for dependencies is a bit of a downer, but static analysis of the dependency tree is kind of neat. Tradeoffs galore.

This design also requires types to specify the _way_ they want their dependencies, either as owned `Owned<T>` or borrowed `Rc<T>`. I'm in two minds about this. On the one hand it's nice to be able to describe exactly the things you require of your dependencies. On the other hand it might not be desirable to force knowledge of where `T` comes from onto its dependents.

It's also important to note that `Owned<T>`, `Rc<T>` and `Rc<RefCell<T>>` are distinct types, and each will recieve a different instance of `T`. This is reasonable when you think about it, but means a caller also needs to consider how much isolation they need for their dependencies. With other dependency injection solutions users don't need to worry about dependency storage or ownership, wheras here we need both. That may not necessarily be a bad thing, but the result of an `Rc<T>` not observing changes made to an `Rc<RefCell<T>>` may be surprising.

So overall, the design is pretty leaky in a few ways, but that could be justified by calling it 'flexibility'.

## The verdict

In its current form, it's not possible to introduce this approach to manage dependencies without imposing a specific structure on them (`Rc<T>`) and forcing questions of ownership on the user. This is a bit of a problem, but could be solved in some sense by forcing an `Rc<RefCell<T>>` on everyone. Resolving the dependency graph at compile-time is pretty neat though, and catching things like missing dependencies and cycles, which should also prevent issues with `Rc` leaking due to cycles.

The issue with borrowed dependencies comes from borrowing data for a lifetime that the scope can't manage. We don't know when any particular dependency will go out of scope so the whole thing falls over. Enhancements to lifetimes may improve this in the future, perhaps with something as simple as a _does not outlive_ bound. That's a reactionary solution though.
