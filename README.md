# SurrealDB Functions

This is a proc-macro crate that given a path to a .surql file or a folder of .surql files, will parse `DEFINE FUNCTION fn::`s inside them and output rust fns that wrap around them. 
It will also generate a bootstrap function that stores the defined functions to db, and finally if a function is nested (`fn::a::nested::function`) it will be put in a nested module.

Currently this repo only has the minimal surrealql parser for resolving the custom function definitions.
