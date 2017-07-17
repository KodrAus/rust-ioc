#![feature(conservative_impl_trait)]

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

// A db connection can be a `GetProductHandler`
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

// An example of returning a closure as a handler implementation
// This could take `DbConn` as an argument instead of another handler. But there's no reason not to take other handlers.
fn get_product<TGetProduct>(get: TGetProduct) -> impl GetProductHandler
    where TGetProduct: GetProductHandler
{
    move |action| {
        let product = get.get_product(action)?;

        // Do another thing

        Ok(product)
    }
}

fn main() {
    // Separate construction from action.
    let handler = get_product(DbConnection {});
  
    // Execute the action with the inputs
    handler.get_product(GetProduct { id: 1 }).unwrap();
}