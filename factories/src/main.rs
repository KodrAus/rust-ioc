/*!
What do we want to do?
Describe a dependency graph without having to specify implementations of things.
One of the tricky things about Rust is that neither trait objects nor generics are as ergonomic as interfaces in other languages.
It just doesn't work that way.
So how can we achieve loose coupling and good testability without interfaces?
Without sacrificing ergonomics?
Can we use factories or something to resolve dependencies?
Will these factories be any more useful than just using generics?
Generics are more problematic because we need to use phantom data.
What if we take a different approach? And separate ambient state from application state?
That is, separate the state we work on from the state we work on it with?
Sounds pretty obvious, but maybe it's worth calling out?
You don't _need_ to have separate objects to implement functionality.
Is that something to consider?
The idea is:
- We don't rely on the fact that there's a single concrete implementation of a thing
- But we have a single concrete implementation of a thing for convenience
Pass the required types to a function as generics.
What do we want to do?
# Separate implementation inputs from action inputs
Why?
So we can execute the same logic with different implementations, or use the same implementations multiple times.
We have functions, so why not use them?
Is there a more ergonomic way to separate boilerplate from app logic?
Is the glue between services part of the app logic?
I'd say yes, and we should treat it as such.
But is it less useful to compose smaller commands?
Is it a separate 'command' when it's not pure?
# Separate impure commands from pure ones
Why?
So we can test the impure code independently, and how it affects the pure code.
How will this compose?
We need to write a test app that uses different kinds of things and see.
The point is not to build abstractions unless they're useful.
The use we're getting here is separating injection from execution.
The pattern is to have common plumbing that tests and APIs and CLIs etc can run through.
*/

#![feature(conservative_impl_trait)]

// Could be done on stable without `impl Trait` by returning `Box<Fn>`
fn get_products<TGetProducts>(get: TGetProducts) -> impl Fn(GetProduct) -> Result<Product, String>
    where TGetProducts: GetProductHandler
{
    move |action| {
        let product = get.get_product(action)?;

        // Do another thing

        Ok(product)
    }
}

struct GetProduct {
    id: i32
}

struct Product {
    title: String
}

trait GetProductHandler {
    fn get_product(&self, action: GetProduct) -> Result<Product, String>;
}

struct DbConnection {

}

impl GetProductHandler for DbConnection {
    fn get_product(&self, _action: GetProduct) -> Result<Product, String> {
        Ok(Product {
            title: "Some product".into()
        })
    }
}

// Any function can also be a `GetProductHandler`
impl<F> GetProductHandler for F 
    where F: Fn(GetProduct) -> Result<Product, String>
{
    fn get_product(&self, action: GetProduct) -> Result<Product, String> {
        self(action)
    }
}

fn main() {
    // Separate construction from action.
    // Why have a `GetProductHandler` if we're not going to use it?
    // Can this be made more ergonomic for the single case?
    let action = get_products(DbConnection {});
  
    // Execute the action with the inputs
    action(GetProduct { id: 1 }).unwrap();
}