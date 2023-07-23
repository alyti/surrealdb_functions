use std::{future::Future, pin::Pin, str::FromStr, collections::HashMap};

use surrealdb::{engine::local::Mem, Surreal, sql::{Value, value, Bytes, Array, thing}};
use surrealdb_functions::include_fn;

include_fn!{
    driver as gen_$;
    "$CARGO_MANIFEST_DIR/tests/bindings.surql"
}


type FnType = dyn Future<Output = surrealdb::Result<surrealdb::Response>>; 


struct Test {
    name: &'static str,
    input: Pin<Box<FnType>>,
    output: Value,
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    // In-memory database for testing
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;
    gen_define_functions(&db).await?.check()?;

    // Unit tests for each function
    let tests: Vec<Test> = vec![
        Test{
            name: "number",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_number(&db, 10).await })
            },
            output: value(r#"[10]"#)?,
        },
        Test{
            name: "string",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_string(&db, "driver").await })
            },
            output: value(r#"["driver"]"#)?,
        },
        Test{
            name: "bool",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_bool(&db, true).await })
            },
            output: value(r#"[true]"#)?,
        },
        Test{
            name: "datetime",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_datetime(&db, chrono::DateTime::default()).await })
            },
            output: value(r#"["1970-01-01T00:00:00Z"]"#)?,
        },
        Test{
            name: "duration",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_duration(&db, std::time::Duration::new(10, 0)).await })
            },
            output: value(r#"[10s]"#)?,
        },
        Test{
            name: "bytes",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_bytes(&db, vec![10u8, 20u8, 30u8, 40u8, 50u8]).await })
            },
            output: Value::Array(Array::from(Value::Bytes(Bytes::from(vec![10u8, 20u8, 30u8, 40u8, 50u8])))),
        },
        Test{
            name: "uuid",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_uuid(&db, uuid::Uuid::from_str("e72bee20-f49b-11ec-b939-0242ac120002").unwrap()).await })
            },
            output: value(r#"['e72bee20-f49b-11ec-b939-0242ac120002']"#)?,
        },
        Test{
            name: "record",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_record(&db, thing("table:id")?).await })
            },
            output: value(r#"[table:id]"#)?,
        },
        Test{
            name: "int",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_int(&db, 10).await })
            },
            output: value(r#"[10]"#)?,
        },
        Test{
            name: "decimal",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_decimal(&db, 10.69).await })
            },
            output: value(r#"[10.69]"#)?,
        },
        Test{
            name: "point",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_point(&db, (10.0, 15.0)).await })
            },
            output: value(r#"[(10.0, 15.0)]"#)?,
        },
        Test{
            name: "geometry (just point really)",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_geometry(&db, (10.0, 15.0)).await })
            },
            output: value(r#"[(10.0, 15.0)]"#)?,
        },
        Test{
            name: "array",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_array(&db, vec!["hello", "world"]).await })
            },
            output: value(r#"["hello", "world"]"#)?,
        },
        Test{
            name: "set",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_set(&db, vec!["hello", "world", "world"]).await })
            },
            output: value(r#"["hello", "world"]"#)?,
        },
        Test{
            name: "object",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_object(&db, HashMap::from([("hello", value("10")?)])).await })
            },
            output: value(r#"[{"hello": 10}]"#)?,
        },
        Test{
            name: "any",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_any(&db, "hello world").await })
            },
            output: value(r#"["hello world"]"#)?,
        },
        Test{
            name: "either",
            input: {
                let db = db.clone();
                Box::pin(async move { gen_either(&db, true).await })
            },
            output: value(r#"[true]"#)?,
        },
    ];

    let mut failed = false;
    for test in tests {
        let got: Value = test.input.await?.check()?.take(0).unwrap();
        if got.ne(&test.output) {
            failed = true;
            println!("{}\nExpected:\n{:#?}\nGot:\n{:#?}\n{:?}", test.name, test.output, got, got.to_string());
        }
    }

    if failed {
        panic!("Some tests failed");
    }

    Ok(())
}
