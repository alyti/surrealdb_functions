use surrealdb::{engine::local::Mem, Surreal};
use surrealdb_functions::include_fn;

include_fn!("$CARGO_MANIFEST_DIR/tests/main.surql");

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    // In-memory database for testing
    let db = Surreal::new::<Mem>(()).await?;

    // Use the test namespace and database
    db.use_ns("test").use_db("test").await?;
    
    // Define the functions using the include_fn! macro defined functions
    define_functions(&db).await?.check()?;

    // Call the example functions
    dbg!(greet_but_with_number(&db, "earth", 10).await?.check()?);
    dbg!(nested::greet(&db, "world00").await?.check()?);

    Ok(())
}

