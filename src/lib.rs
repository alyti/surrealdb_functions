mod parser;
mod file;

use std::{path::PathBuf, error::Error, collections::HashMap};

use litrs::Literal;
use nom::combinator::all_consuming;
use parser::{DefineFunctionStatement, kind::Kind};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashSet;

#[proc_macro]
pub fn include_fn(input: TokenStream) -> TokenStream {
    include_fn_impl(input.into()).into()
}

fn include_fn_impl(input: TokenStream2) -> TokenStream2 {
    let paths = include_fn_args(input);
    
    let bootstrap = bootstrap_for_files(paths.iter().cloned().collect::<Vec<PathBuf>>().as_slice()).unwrap();
    let functions = build_mod_tree(paths.iter().cloned().collect::<Vec<PathBuf>>().as_slice()).unwrap();

    quote!{
        #bootstrap

        #functions
    }
}

#[derive(Debug, Default)]
struct Function (Vec<DefineFunctionStatement>, HashMap<String, Function>);

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
                        let nested: &mut Function = current.entry(part.to_owned()).or_default();
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
    fn to_tokens(&self) -> TokenStream2 {
        let mut out = TokenStream2::new();

        for item in &self.0 {
            out.extend(item.to_tokens());
        }

        for (name, item) in &self.1 {
            let name = Ident::new(name, Span::call_site());
            let item = item.to_tokens();
            out.extend(quote!{
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
        match self{
            Kind::Bool => quote!{ impl Into < ::surrealdb::sql::Bool > },
            Kind::Bytes => quote!{ impl Into < ::surrealdb::sql::Bytes > },
            Kind::Datetime => quote!{ impl Into < ::surrealdb::sql::DateTime > },
            Kind::Duration => quote!{ impl Into < ::surrealdb::sql::Duration > },
            Kind::Float | Kind::Int | Kind::Decimal | Kind::Number => quote!{ impl Into < ::surrealdb::sql::Number > },
            Kind::String => quote!{ impl Into< String > },
            Kind::Uuid => quote!{ impl Into < ::surrealdb::sql::Uuid > },
            Kind::Record(_) => quote!{ impl Into < ::surrealdb::sql::Thing > },
            Kind::Point | Kind::Geometry(_) => quote!{ impl Into < ::surrealdb::sql::Geometry > },
            Kind::Option(nested) => {
                let nested = nested.to_tokens();
                quote!{ Option < #nested > }
            },
            Kind::Any | Kind::Object | Kind::Either(_) => quote!{ impl Into < ::surrealdb::sql::Value > },
            Kind::Set(nested, _) | Kind::Array(nested, _) =>  {
                let nested = nested.to_tokens();
                quote!{ Vec< #nested > }
            },
        }
    }
}

impl DefineFunctionStatement {
    fn params_to_args(&self) -> TokenStream2 {
        let mut out = TokenStream2::new();

        for (name, kind) in &self.args {
            let name = Ident::new(name, Span::call_site());
            let kind = kind.to_tokens();
            out.extend(quote!{ #name: #kind, });
        }

        out
    }

    fn params_to_bindings(&self) -> TokenStream2 {
        let mut out = TokenStream2::new();

        for (name, _) in &self.args {
            let key = name.to_string();
            let name = Ident::new(name, Span::call_site());
            out.extend(quote!{ 
                .bind((#key, #name.into())) 
            });
        }

        out
    }

    fn custom_function_query(&self) -> String {
        let mut out = String::new();
        out.push_str("RETURN fn");
        for name in &self.name {
            out.push_str("::");
            out.push_str(&name);
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

    fn to_tokens(&self) -> TokenStream2 {
        let name = Ident::new(&self.name.last().unwrap(), Span::call_site());
        let args = self.params_to_args();
        let bind = self.params_to_bindings();
        let query = self.custom_function_query();
        // turn comments into rust comments
        let comments = self.comments.iter().map(|s| quote!{
            #[doc = #s]
        }).collect::<TokenStream2>();
        quote!{
            #comments
            pub async fn #name<C: ::surrealdb::Connection>(db: &::surrealdb::Surreal<C>, #args) -> ::surrealdb::Result<::surrealdb::Response> {
                db.query(#query)
                #bind
                .await
            }
        }
    }
}

fn build_mod_tree(paths: &[PathBuf]) -> Result<TokenStream2, Box<dyn Error>> {
    // Takes a list of files, parses them for functions
    let functions = parse_surrealql_files(paths)?;

    // Builds a tree of functions
    let functions = Function::from(functions);

    Ok(functions.to_tokens())
}

fn parse_surrealql_files(paths: &[PathBuf]) -> Result<Vec<DefineFunctionStatement>, Box<dyn Error>> {
    let mut out = vec![];

    for path in paths {
        out.extend(parse_surrealql_file(path)?);
    }

    Ok(out)
}

fn parse_surrealql_file(path: &PathBuf) -> Result<Vec<DefineFunctionStatement>, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;
    let (_, fns) = all_consuming(parser::functions)(&contents).map_err(|e| e.to_string())?;
    Ok(fns)
}

fn transform_filename_to_const_name(path: &PathBuf) -> Ident {
    let mut name = path.file_name().unwrap().to_str().unwrap().to_owned();
    name.retain(|c| c.is_ascii_alphanumeric() || c == '_');
    let name = name.to_uppercase();
    Ident::new(&format!("_SURQL_FILE_{}", name), Span::call_site())
}

fn bootstrap_for_files(paths: &[PathBuf]) -> Result<TokenStream2, Box<dyn Error>> {
    let mut consts = TokenStream2::new();
    let mut consts_names = TokenStream2::new();

    for path in paths {
        let name = transform_filename_to_const_name(path);
        consts.extend(generate_include(&name, path.to_str().unwrap()));

        consts_names.extend(quote! {
            out.push_str(#name);
        });
    }

    Ok(quote!{
        #consts

        #[doc = "Returns a string containing all the functions defined in the included files."]
        pub fn stored_functions() -> String {
            let mut out = String::new();
            #consts_names
            out
        }

        #[doc = "Defines all the functions using the provided connection."]
        pub async fn define_functions<C: ::surrealdb::Connection>(db: &::surrealdb::Surreal<C>) -> ::surrealdb::Result<::surrealdb::Response> {
            db.query(stored_functions()).await
        }

        #[doc = "Defines all the functions using the provided datastore and session."]
        pub async fn define_functions_with_datastore(ds: &::surrealdb::kvs::Datastore, session: &::surrealdb::dbs::Session) -> Result<Vec<::surrealdb::dbs::Response>, ::surrealdb::err::Error> {
            ds.execute(&stored_functions(), session, None).await
        }
    })
}

fn include_fn_args(input: TokenStream2) -> HashSet<PathBuf> {
    let mut out = HashSet::new();

    for tt in input {
        let lit = match Literal::try_from(tt) {
            Ok(lit) => lit,
            Err(e) => panic!("failed to parse literal: {}", e),
        };

        match lit {
            Literal::String(s) => {
                match file::resolve_path(s.value(), file::get_env) {
                    Ok(path) => {
                        if path.exists() {
                            out.extend(expand_path(&path).unwrap());
                        } else {
                            panic!("file does not exist: {}", path.to_str().unwrap());
                        }
                    },
                    Err(e) => panic!("failed to resolve path: {}", e),
                }
            },
            _ => panic!("input has to be a string literal, but this is not: {}", lit),
        }
    }

    out
}

fn add_path_if_surql(path: &PathBuf, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    if path.extension().unwrap_or_default() == "surql" {
        out.push(path.to_owned());
    }
    Ok(())
}

fn expand_path(path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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
        add_path_if_surql(&path, &mut out)?;
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
        let paths = vec![
            PathBuf::from("tests/main.surql")
        ];
        let functions = parse_surrealql_files(&paths).unwrap();
        let _ = Function::from(functions);
    }
}
