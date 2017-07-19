#![feature(conservative_impl_trait)]

struct Product {
    id: i32,
    title: String
}

#[derive(Clone)]
struct DbConnection {

}

impl DbConnection {
    fn do_stuff(&self) {

    }
}

struct GetProduct {
    id: i32
}

trait GetProductQuery {
    fn get_product(&self, action: GetProduct) -> Result<Product, String>;
}

// Any function can be a `GetProductQuery`
impl<F> GetProductQuery for F 
    where F: Fn(GetProduct) -> Result<Product, String>
{
    fn get_product(&self, action: GetProduct) -> Result<Product, String> {
        self(action)
    }
}

// Return a closure to get a product
fn get_product(conn: DbConnection) -> impl GetProductQuery {
    move |action: GetProduct| {
        conn.do_stuff();

        Ok(Product {
            id: action.id,
            title: "Some product".into()
        })
    }
}

struct SetProductTitle {
    id: i32,
    title: String
}

trait SetProductTitleCommand {
    fn set_product_title(self, action: SetProductTitle) -> Result<(), String>;
}

// Any function can be a `SetProductTitleCommand`
impl<F> SetProductTitleCommand for F 
    where F: FnOnce(SetProductTitle) -> Result<(), String>
{
    fn set_product_title(self, action: SetProductTitle) -> Result<(), String> {
        self(action)
    }
}

// Return a closure to get a product
// Could this be made more ergonomic?
fn set_product_title<TGetProduct>(conn: DbConnection, get_product: TGetProduct) -> impl SetProductTitleCommand 
    where TGetProduct: GetProductQuery
{
    move |action: SetProductTitle| {
        let mut product = get_product.get_product(GetProduct { id: action.id })?;
        product.title = action.title;

        conn.do_stuff();

        Ok(())
    }
}

fn main() {
    let conn = DbConnection {};

    // Separate construction from action.
    let query = get_product(conn.clone());
  
    // Execute the action with the inputs
    query.get_product(GetProduct { id: 1 }).unwrap();
    
    let command = set_product_title(conn.clone(), query);

    command.set_product_title(SetProductTitle { id: 1, title: "A new title".into() });

    // An alternative implementation of `GetProductQuery` that doesn't use a db connection
    let command = set_product_title(conn.clone(), |action| Ok(Product { id: 1, title: "Stuff".into() }));

    command.set_product_title(SetProductTitle { id: 1, title: "A new title".into() });
}