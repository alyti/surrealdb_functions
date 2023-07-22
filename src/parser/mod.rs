use nom::{
    bytes::complete::{tag, tag_no_case, take_until},
    character::complete::{char, multispace0},
    multi::{separated_list0, separated_list1},
    IResult,
};

pub mod comment;
pub mod common;
pub mod escape;
pub mod fmt;
pub mod ident;
pub mod kind;
pub mod table;

use comment::{mightbecomment, mightbespace, shouldbespace};
use common::{closebraces, commas, openbraces};
use ident::{ident, Ident};
use kind::{kind, Kind};

use self::common::colons;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DefineFunctionStatement {
    pub comments: Vec<String>,
    pub name: Vec<String>,
    pub args: Vec<(Ident, Kind)>,
}

impl std::hash::Hash for DefineFunctionStatement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub fn functions(i: &str) -> IResult<&str, Vec<DefineFunctionStatement>> {
    let (i, _) = multispace0(i)?;
    let (i, v) = separated_list1(colons, function)(i)?;
    let (i, _) = colons(i)?;
    Ok((i, v))
}

fn function(i: &str) -> IResult<&str, DefineFunctionStatement> {
    let (i, comments) = mightbecomment(i)?;
    let (i, _) = mightbespace(i)?;
    let (i, _) = tag_no_case("DEFINE")(i)?;
    let (i, _) = shouldbespace(i)?;
    let (i, _) = tag_no_case("FUNCTION")(i)?;
    let (i, _) = shouldbespace(i)?;
    let (i, _) = tag("fn::")(i)?;
    let (i, name) = ident::multikeep(i)?;
    let (i, _) = mightbespace(i)?;
    let (i, _) = char('(')(i)?;
    let (i, _) = mightbespace(i)?;
    let (i, args) = separated_list0(commas, |i| {
        let (i, _) = char('$')(i)?;
        let (i, name) = ident(i)?;
        let (i, _) = mightbespace(i)?;
        let (i, _) = char(':')(i)?;
        let (i, _) = mightbespace(i)?;
        let (i, kind) = kind(i)?;
        Ok((i, (name, kind)))
    })(i)?;
    let (i, _) = mightbespace(i)?;
    let (i, _) = char(')')(i)?;
    let (i, _) = mightbespace(i)?;
    let (i, _) = ignored_block(i)?;
    Ok((
        i,
        DefineFunctionStatement {
            comments: comments.iter().map(|s| s.to_string()).collect(),
            name: name.iter().map(|s| s.to_string()).collect(),
            args,
        },
    ))
}

pub fn ignored_block(i: &str) -> IResult<&str, ()> {
    let (i, _) = openbraces(i)?;
    let (i, _) = take_until("}")(i)?;
    let (i, _) = closebraces(i)?;
    Ok((i, ()))
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::table::Table;

    use super::*;

    #[test]
    fn function_normal() {
        let sql = r#"DEFINE FUNCTION fn::greet($name: string) {
	RETURN "Hello, " + $name + "!";
}"#;
        let res = function(sql);
        assert!(res.is_ok());
        let out = res.unwrap().1;
        assert_eq!(
            out,
            DefineFunctionStatement {
                comments: vec![],
                name: vec!["greet".to_string()],
                args: vec![(Ident::from("name"), Kind::String)],
            }
        );
    }

    #[test]
    fn function_with_comments() {
        let sql = r#"
-- It is necessary to prefix the name of your function with "fn::"
-- This indicates that it's a custom function
DEFINE FUNCTION fn::greet($name: string) {
	RETURN "Hello, " + $name + "!";
}"#;
        let res = function(sql);
        assert!(res.is_ok());
        let out = res.unwrap().1;
        assert_eq!(
            out,
            DefineFunctionStatement {
                comments: vec![
                    "It is necessary to prefix the name of your function with \"fn::\"".to_string(),
                    "This indicates that it's a custom function".to_string()
                ],
                name: vec!["greet".to_string()],
                args: vec![(Ident::from("name"), Kind::String)],
            }
        );
    }

    #[test]
    fn function_complex() {
        let sql = r#"
DEFINE FUNCTION fn::relation_exists::nested(
    $in: record<some>,
    $tb: string,
    $out: record<other>
) {};
"#;
        let res = function(sql);
        let out = res.unwrap().1;
        assert_eq!(
            out,
            DefineFunctionStatement {
                comments: vec![],
                name: vec!["relation_exists".to_string(), "nested".to_string()],
                args: vec![
                    (
                        Ident::from("in"),
                        Kind::Record(vec![Table("some".to_string())])
                    ),
                    (Ident::from("tb"), Kind::String),
                    (
                        Ident::from("out"),
                        Kind::Record(vec![Table("other".to_string())])
                    )
                ],
            }
        );
    }

    #[test]
    fn functions_basic() {
        let sql = r#"
-- It is necessary to prefix the name of your function with "fn::"
-- This indicates that it's a custom function
DEFINE FUNCTION fn::greet($name: string) {
    RETURN "Hello, " + $name + "!";
};

// It is necessary to prefix the name of your function with "fn::"
// This indicates that it's a custom function
DEFINE FUNCTION fn::greet($name: string) {
    RETURN "Hello, " + $name + "!";
};

# A different comment style
DEFINE FUNCTION fn::relation_exists::nested(
    $in: record<some>,
    $tb: string,
    $out: record<other>
) {};
"#;
        let res = functions(sql);
        let out = res.unwrap().1;
        assert_eq!(
            out,
            vec![
                DefineFunctionStatement {
                    comments: vec![
                        "It is necessary to prefix the name of your function with \"fn::\""
                            .to_string(),
                        "This indicates that it's a custom function".to_string()
                    ],
                    name: vec!["greet".to_string()],
                    args: vec![(Ident::from("name"), Kind::String)],
                },
                DefineFunctionStatement {
                    comments: vec![
                        "It is necessary to prefix the name of your function with \"fn::\""
                            .to_string(),
                        "This indicates that it's a custom function".to_string()
                    ],
                    name: vec!["greet".to_string()],
                    args: vec![(Ident::from("name"), Kind::String)],
                },
                DefineFunctionStatement {
                    comments: vec!["A different comment style".to_string(),],
                    name: vec!["relation_exists".to_string(), "nested".to_string()],
                    args: vec![
                        (
                            Ident::from("in"),
                            Kind::Record(vec![Table("some".to_string())])
                        ),
                        (Ident::from("tb"), Kind::String),
                        (
                            Ident::from("out"),
                            Kind::Record(vec![Table("other".to_string())])
                        )
                    ],
                }
            ]
        );
    }
}
