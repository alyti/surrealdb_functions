use surrealdb::{engine::local::Mem, Surreal, kvs::Datastore, dbs::Session};
use surrealdb_functions::include_fn;

include_fn!{
    driver as is;
    datastore as ds_$;
    "$CARGO_MANIFEST_DIR/tests/main.surql";
    "$CARGO_MANIFEST_DIR/tests/main.surql"
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    // In-memory database for testing
    let db = Surreal::new::<Mem>(()).await?;

    // Use the test namespace and database
    db.use_ns("test").use_db("test").await?;
    
    // Define the functions using the include_fn! macro defined functions
    define_functions(&db).await?.check()?;

    // Call the example functions
    dbg!(greet_but_with_number(&db, "driver", 10).await?.check()?);

    // Direct datastore access
    let ds = Datastore::new("memory").await?;
    let ses = Session::for_kv().with_ns("test").with_db("test");

    // Same as above but with datastore
    let mut res = ds_define_functions(&ds, &ses).await?;
    let tmp = res.remove(0).result;
    match tmp {
        Ok(_) => {},
        Err(e) => panic!("ERR: {e}"),
    }

    // Wow using datastore sure is very verbose huh...
    let mut res = ds_greet_but_with_number(&ds, &ses, "datastore", 10).await?;
    let tmp = res.remove(0).result;
	match tmp {
        Ok(msg) => println!("OK {msg}"),
        Err(e) => println!("ERR: {e}"),
    }

    Ok(())
}
