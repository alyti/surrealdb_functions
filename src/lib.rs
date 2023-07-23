//! [![github]](https://github.com/alyti/surrealdb_functions)&ensp;[![crates-io]](https://crates.io/crates/surrealdb_functions)&ensp;[![docs-rs]](crate)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//! 
//! <br>
//! 
//! SurrealQL Functions is a procedural macro that allows you to include functions from SurrealQL files into your Rust project.
//! 
//! Check the [`surrealdb_functions::include_fn`] macro for more information.
//! 
//! [`surrealdb_functions::include_fn`]: macro.include_fn.html
mod file;
mod parser;

use proc_macro::TokenStream;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
};

use nom::combinator::all_consuming;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse::Parse, parse_macro_input};

use parser::{kind::Kind, DefineFunctionStatement};



/// Include functions from .surql files and generate wrappers for them.
///
/// This function parses a list of .surql files and generates a module containing functions for each function defined in the files.
/// 
/// Output:
/// - `stored_functions() -> String`: Returns a string containing all the functions defined in the included files.
/// - `define_functions(db: &Surreal) -> Result<Response>`: Defines all the functions using the provided connection.
/// - `async fn <name>(db: &Surreal, /* parsed arguments */) -> Result<Response>`: Defined functions from the .surql file.
///   If a function has a comment above it, the comment will be used as the documentation for the function.
///   <name> is the last part of the function's name that's transformed based on the driver and datastore arguments.
///   If a function in the .surql file has a name that is more than one part, each part is treated as a module.
///   For example, a function named `foo::bar` will be generated as `mod foo { async fn bar(/* ... */) } }`.
/// 
/// Arguments:
/// - `driver as <alias>`: The alias to use for the driver functions. If not provided, the functions will not be generated.
/// - `datastore as <alias>`: The alias to use for the datastore functions. If not provided, the functions will not be generated.
/// - `[<path>]`: The path to the .surql file to include. If the path is a directory, all .surql files in the directory will be included.
/// 
/// <alias> can be one of the following:
/// - `is` will not apply any changes to the method names.
/// - `prefix_$`/`$_suffix` will replace `$` with function name, effectively prefixing/suffixing it (ex. `prefix_greet` / `greet_suffix`)
/// 
///
/// # Example
///
/// ```
/// # extern crate surrealdb;
/// # extern crate tokio;
/// #
/// use surrealdb::{engine::local::Mem, Surreal, kvs::Datastore, dbs::Session};
/// use surrealdb_functions::include_fn;
/// 
/// include_fn!{
///     driver as is;
///     "$CARGO_MANIFEST_DIR/tests/main.surql";
/// }
///
/// #[tokio::main]
/// async fn main() -> surrealdb::Result<()> {
///     // In-memory database for testing
///     let db = Surreal::new::<Mem>(()).await?;
///     // Use the test namespace and database
///     db.use_ns("test").use_db("test").await?;
///     // Define the functions using the include_fn! macro defined functions
///     define_functions(&db).await?.check()?;
///     // Call the example functions
///     dbg!(greet_but_with_number(&db, "driver", 10).await?.check()?);
///     Ok(())
/// }
/// ```
/// 
/// More examples can be found in the [examples](examples) directory.
#[proc_macro]
#[proc_macro_error]
pub fn include_fn(input: TokenStream) -> TokenStream {
    include_fn_impl(parse_macro_input!(input as IncludeFnArgs)).into()
}

fn include_fn_impl(input: IncludeFnArgs) -> TokenStream2 {
    let bootstrap = bootstrap_for_files(&input).unwrap();
    let functions = build_mod_tree(&input).unwrap();

    // eprintln!("{}", functions.to_string());
    quote! {
        #bootstrap

        #functions
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Alias {
    AsIs,
    Prefix(String),
    Suffix(String),
}

impl Alias {
    fn transform(&self, name: &str) -> String {
        match self {
            Alias::AsIs => name.to_string(),
            Alias::Prefix(prefix) => format!("{}{}", prefix, name),
            Alias::Suffix(suffix) => format!("{}{}", name, suffix),
        }
    }
}

impl Parse for Alias {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        // Right now we only support ident `is`, prefix notation `prefix_$` and suffix notation `$_suffix`.
        // $ is treated as substituting the name of the function, but syn will not parse it as an ident, instead its punctuation, so we have to handle it ourselves.
        let ident: Option<Ident> = input.parse()?;

        match ident {
            Some(ident) if ident.to_string().as_str() == "is" => Ok(Self::AsIs),
            Some(ident) => {
                let punct: Option<syn::token::Dollar> = input.parse()?;
                match punct {
                    Some(_) => Ok(Self::Prefix(ident.to_string())),
                    _ => {
                        abort! {punct, "invalid alias"; help = "expected `is`, `$_suffix` or `prefix_$`"}
                    }
                }
            }
            None => {
                let punct: Option<syn::token::Dollar> = input.parse()?;
                match punct {
                    Some(_) => {
                        let ident: Ident = input.parse()?;
                        Ok(Self::Suffix(ident.to_string()))
                    }
                    _ => {
                        abort! {ident, "invalid alias"; help = "expected `is`, `$_suffix` or `prefix_$`"}
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct IncludeFnArgs {
    paths: HashSet<PathBuf>,
    driver: Option<Alias>,
    datastore: Option<Alias>,
}

impl IncludeFnArgs {
    fn transform_fn_name(&self, name: &str) -> (Option<Ident>, Option<Ident>) {
        (
            self.driver
                .as_ref()
                .map(|alias| Ident::new(&alias.transform(name), Span::call_site())),
            self.datastore
                .as_ref()
                .map(|alias| Ident::new(&alias.transform(name), Span::call_site())),
        )
    }
}

impl Parse for IncludeFnArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut paths = HashSet::new();
        let mut driver = None;
        let mut datastore = None;

        while !input.is_empty() {
            let ident: Option<Ident> = input.parse()?;
            if let Some(ident) = ident {
                match ident.to_string().as_str() {
                    "driver" => {
                        input.parse::<syn::Token![as]>()?;
                        driver = Some(Alias::parse(input)?);
                        if driver.eq(&datastore) {
                            abort!(ident, "driver and datastore cannot be the same")
                        }
                    }
                    "datastore" => {
                        input.parse::<syn::Token![as]>()?;
                        datastore = Some(Alias::parse(input)?);
                        if driver.eq(&datastore) {
                            abort!(ident, "driver and datastore cannot be the same")
                        }
                    }
                    _ => {
                        abort!(ident, "unknown argument"; help="only driver and datastore are supported")
                    }
                }
            } else {
                let lit: Literal = input.parse()?;
                match file::resolve_path(lit.to_string().trim_matches('"'), file::get_env) {
                    Ok(path) => {
                        if path.exists() {
                            paths.extend(expand_path(&path).unwrap());
                        } else {
                            abort!(lit, "file does not exist"; note="make sure the file exists");
                        }
                    }
                    Err(e) => {
                        abort!(lit, format!("failed to resolve path: {e}"); note="make sure the path is valid")
                    }
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<syn::Token![;]>()?;
        }

        if datastore.is_none() && driver.is_none() {
            panic!("no driver or datastore provided");
        }

        if paths.is_empty() {
            panic!("no paths provided");
        }

        Ok(Self {
            paths,
            driver,
            datastore,
        })
    }
}

#[derive(Debug, Default)]
struct Function(Vec<DefineFunctionStatement>, HashMap<String, Function>);

impl From<Vec<DefineFunctionStatement>> for Function {
    fn from(v: Vec<DefineFunctionStatement>) -> Self {
        let mut rooted = vec![];
        let mut nested = HashMap::new();

        for item in v {
            if item.name.len() == 1 {
                // This function doesn't have a parent, so it's treated as a root function
                rooted.push(item);
            } else {
                // This function has more than one part for it's name so each part is treated as a module
                let mut current = &mut nested;
                let mut next_items: &mut Vec<DefineFunctionStatement> = &mut vec![];
                // Iterate over each part of the name, if the part doesn't exist in the current module, create it
                // If it's the last part of the name, add the function to the module
                let len = item.name.len();
                for (i, part) in item.name.iter().enumerate() {
                    if i == len - 1 {
                        next_items.push(item.clone());
                    } else {
                        let nested: &mut Function = current.entry(part.clone()).or_default();
                        next_items = &mut nested.0;
                        current = &mut nested.1;
                    }
                }
            }
        }

        Self(rooted, nested)
    }
}

impl Function {
    fn to_tokens(&self, args: &IncludeFnArgs) -> TokenStream2 {
        let mut out = TokenStream2::new();

        for item in &self.0 {
            out.extend(item.to_tokens(args));
        }

        for (name, item) in &self.1 {
            let name = Ident::new(name, Span::call_site());
            let item = item.to_tokens(args);
            out.extend(quote! {
                pub mod #name {
                    #item
                }
            });
        }

        out
    }
}

impl Kind {
    fn to_tokens(&self) -> TokenStream2 {
        // TODO: These are best guess only, still need to test them
        match self {
            Kind::Bool => quote! { impl Into < bool > },
            Kind::Bytes => quote! { impl Into < ::surrealdb::sql::Bytes > },
            Kind::Datetime => quote! { impl Into < ::surrealdb::sql::Datetime > },
            Kind::Duration => quote! { impl Into < ::surrealdb::sql::Duration > },
            Kind::Float | Kind::Int | Kind::Decimal | Kind::Number => {
                quote! { impl Into < ::surrealdb::sql::Number > }
            }
            Kind::String => quote! { impl Into< ::surrealdb::sql::Strand > },
            Kind::Uuid => quote! { impl Into < ::surrealdb::sql::Uuid > },
            Kind::Record(_) => quote! { impl Into < ::surrealdb::sql::Thing > },
            Kind::Point | Kind::Geometry(_) => quote! { impl Into < ::surrealdb::sql::Geometry > },
            Kind::Option(nested) => {
                let nested = nested.to_tokens();
                quote! { Option < #nested > }
            }
            Kind::Any | Kind::Either(_) => {
                // TODO: Either probably needs to be resolved better than throwing it all into Value
                quote! { impl Into < ::surrealdb::sql::Value > }
            }
            Kind::Object => {
                quote! { impl Into < ::surrealdb::sql::Object >  }
            }
            Kind::Set(_, _) | Kind::Array(_, _) => {
                quote! { impl Into < ::surrealdb::sql::Array >  }
            }
        }
    }
}

impl DefineFunctionStatement {
    fn params_to_args(&self) -> TokenStream2 {
        let mut out = TokenStream2::new();

        for (name, kind) in &self.args {
            let name = Ident::new(name, Span::call_site());
            let kind = kind.to_tokens();
            out.extend(quote! { #name: #kind, });
        }

        out
    }

    fn params_to_bindings(&self) -> TokenStream2 {
        let mut out = TokenStream2::new();

        for (name, _) in &self.args {
            let key = name.to_string();
            let value = Ident::new(name, Span::call_site());
            out.extend(quote! {
                .bind((#key, #value.into()))
            });
        }

        out
    }

    fn params_to_variables(&self) -> TokenStream2 {
        // Build a Option<BTreeMap<String, Value>> for the variables
        let mut out = quote! {
            let mut variables: std::collections::BTreeMap<String, ::surrealdb::sql::Value> = ::std::collections::BTreeMap::new();
        };
        for (name, _) in &self.args {
            let key = name.to_string();
            let value = Ident::new(name, Span::call_site());
            out.extend(quote! {
                variables.insert(#key.to_string(), ::surrealdb::sql::Value::from(#value.into()));
            });
        }

        out
    }

    fn custom_function_query(&self) -> String {
        let mut out = String::new();
        out.push_str("RETURN fn");
        for name in &self.name {
            out.push_str("::");
            out.push_str(name);
        }

        out.push('(');
        for (i, (name, _)) in self.args.iter().enumerate() {
            let name = name.to_string();
            out.push('$');
            out.push_str(&name);
            if i < self.args.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push(')');

        out
    }

    fn to_tokens(&self, args: &IncludeFnArgs) -> TokenStream2 {
        let (driver, datastore) = args.transform_fn_name(self.name.last().unwrap());
        let args = self.params_to_args();
        let query = self.custom_function_query();
        // turn comments into rust comments
        let comments = self
            .comments
            .iter()
            .map(|s| {
                quote! {
                    #[doc = #s]
                }
            })
            .collect::<TokenStream2>();

        let mut tokens = TokenStream2::new();
        if let Some(name) = driver {
            let bind = self.params_to_bindings();
            tokens.extend(quote! {
                #comments
                pub async fn #name<C: ::surrealdb::Connection>(db: &::surrealdb::Surreal<C>, #args) -> ::surrealdb::Result<::surrealdb::Response> {
                    db.query(#query)
                    #bind
                    .await
                }
            });
        }

        if let Some(name) = datastore {
            let bind = self.params_to_variables();
            tokens.extend(quote! {
                #comments
                pub async fn #name(ds: &::surrealdb::kvs::Datastore, session: &::surrealdb::dbs::Session, #args) -> Result<Vec<::surrealdb::dbs::Response>, ::surrealdb::err::Error> {
                    #bind
                    ds.execute(#query, session, Some(variables)).await
                }
            });
        }
        tokens
    }
}

fn build_mod_tree(args: &IncludeFnArgs) -> Result<TokenStream2, Box<dyn Error>> {
    // Takes a list of files, parses them for functions
    let functions = parse_surrealql_files(args)?;

    // Builds a tree of functions
    let functions = Function::from(functions);

    Ok(functions.to_tokens(args))
}

fn parse_surrealql_files(
    paths: &IncludeFnArgs,
) -> Result<Vec<DefineFunctionStatement>, Box<dyn Error>> {
    let mut out = vec![];

    for path in paths.paths.iter() {
        out.extend(parse_surrealql_file(path)?);
    }

    Ok(out)
}

fn parse_surrealql_file(path: &PathBuf) -> Result<Vec<DefineFunctionStatement>, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;
    let (_, fns) = all_consuming(parser::functions)(&contents).map_err(|e| e.to_string())?;
    Ok(fns)
}

fn transform_filename_to_const_name(path: &Path) -> Ident {
    let mut name = path.file_name().unwrap().to_str().unwrap().to_owned();
    name.retain(|c| c.is_ascii_alphanumeric() || c == '_');
    let name = name.to_uppercase();
    Ident::new(&format!("_SURQL_FILE_{name}"), Span::call_site())
}

fn bootstrap_for_files(args: &IncludeFnArgs) -> Result<TokenStream2, Box<dyn Error>> {
    let mut consts = TokenStream2::new();
    let mut consts_names = TokenStream2::new();

    for path in args.paths.iter() {
        let name = transform_filename_to_const_name(path);
        consts.extend(generate_include(&name, path.to_str().unwrap()));

        consts_names.extend(quote! {
            out.push_str(#name);
        });
    }

    let (driver, datastore) = args.transform_fn_name("define_functions");

    let mut tokens = quote! {
        #consts

        #[doc = "Returns a string containing all the functions defined in the included files."]
        pub fn stored_functions() -> String {
            let mut out = String::new();
            #consts_names
            out
        }
    };

    if let Some(name) = driver {
        tokens.extend(quote!{
            #[doc = "Defines all the functions using the provided connection."]
            pub async fn #name<C: ::surrealdb::Connection>(db: &::surrealdb::Surreal<C>) -> ::surrealdb::Result<::surrealdb::Response> {
                db.query(stored_functions()).await
            }
        });
    }

    if let Some(name) = datastore {
        tokens.extend(quote!{
            #[doc = "Defines all the functions using the provided datastore and session."]
            pub async fn #name(ds: &::surrealdb::kvs::Datastore, session: &::surrealdb::dbs::Session) -> Result<Vec<::surrealdb::dbs::Response>, ::surrealdb::err::Error> {
                ds.execute(&stored_functions(), session, None).await
            }
        });
    }

    Ok(tokens)
}

fn add_path_if_surql(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    if path.extension().unwrap_or_default() == "surql" {
        out.push(path.to_path_buf());
    }
    Ok(())
}

fn expand_path(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut out = vec![];

    if path.is_dir() {
        for entry in path.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                out.append(&mut expand_path(&path)?);
            } else {
                add_path_if_surql(&path, &mut out)?;
            }
        }
    } else {
        add_path_if_surql(path, &mut out)?;
    }

    Ok(out)
}

fn generate_include(name: &Ident, path: &str) -> TokenStream2 {
    quote! {
        const #name : & 'static str = include_str ! (#path) ;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_tree() {
        let paths = vec![PathBuf::from("tests/main.surql")];
        let args = IncludeFnArgs {
            paths: paths.iter().cloned().collect(),
            driver: Some(Alias::AsIs),
            datastore: Some(Alias::AsIs),
        };
        let functions = parse_surrealql_files(&args).unwrap();
        let _ = Function::from(functions);
    }
}
