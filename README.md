# SurrealDB Functions [![Crates.io](https://img.shields.io/crates/v/surrealdb_functions.svg)](https://crates.io/crates/surrealdb_functions) [![Docs.rs](https://docs.rs/surrealdb_functions/badge.svg)](https://docs.rs/surrealdb_functions)

This is a proc-macro crate that given a path to a .surql file or a folder of .surql files, will parse `DEFINE FUNCTION fn::`s inside them and output rust fns that wrap around them. 
It will also generate a bootstrap function that stores the defined functions to db, and finally if a function is nested (`fn::a::nested::function`) it will be put in a nested module.

## Usage

Example usage can be found in [examples/main.rs](/examples/main.rs), but in short, its main usage is as follows:

```rust
include_fn!{
    driver as is;
    datastore as ds_$;
    "$CARGO_MANIFEST_DIR/tests/main.surql"
}
```

When calling the macro you need to provide what naming the bindings should use: `driver/datastore as is/prefix_$/$_suffix` 
* `is` will not apply any changes to the method names.
* `prefix_$`/`$_suffix` will replace `$` with function name, effectively prefixing/suffixing it (ex. `prefix_greet` / `greet_suffix`)

At least one of `driver/datastore` must be defined.

* `driver` will generate regular `Surreal<C>` bindings.
* `datastore` will generate bindings for the more low-level locally-available-only `surrealdb::kvs::Datastore`

If both are defined, the parser will validate they don't conflictm. (ex. you can't have both be `as is`)

Finally the last argument type is a file/directory path, if a directory is provided, it will be recursively resolved.

At least one valid path argument is expected.

The docs.rs content is coming later, for now either read the source or ask me in surrealdb discord (same handle as on github).
I am open to new feature/pull requests.

# Crate notes

This is a utility proc-macro for surrealdb, as such it expects presence of surrealdb in user's dependencies.
However, this crate by itself, does not depend on surrealdb.

## Parser notes

Currently this macro only has the minimal surrealql parser for resolving the custom function definitions, sans their body.
