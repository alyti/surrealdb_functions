use super::{
    common::commas,
    escape::escape_ident,
    fmt::Fmt,
    ident::{ident_raw, Ident},
};
use nom::{multi::separated_list1, IResult};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    str,
};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Hash)]
pub struct Tables(pub Vec<Table>);

impl From<Table> for Tables {
    fn from(v: Table) -> Self {
        Tables(vec![v])
    }
}

impl Deref for Tables {
    type Target = Vec<Table>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Tables {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&Fmt::comma_separated(&self.0), f)
    }
}

pub fn tables(i: &str) -> IResult<&str, Tables> {
    let (i, v) = separated_list1(commas, table)(i)?;
    Ok((i, Tables(v)))
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Hash)]
pub struct Table(pub String);

impl From<String> for Table {
    fn from(v: String) -> Self {
        Self(v)
    }
}

impl From<&str> for Table {
    fn from(v: &str) -> Self {
        Self::from(String::from(v))
    }
}

impl From<Ident> for Table {
    fn from(v: Ident) -> Self {
        Self(v.0)
    }
}

impl Deref for Table {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&escape_ident(&self.0), f)
    }
}

pub fn table(i: &str) -> IResult<&str, Table> {
    let (i, v) = ident_raw(i)?;
    Ok((i, Table(v)))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn table_normal() {
        let sql = "test";
        let res = table(sql);
        assert!(res.is_ok());
        let out = res.unwrap().1;
        assert_eq!("test", format!("{}", out));
        assert_eq!(out, Table(String::from("test")));
    }

    #[test]
    fn table_quoted_backtick() {
        let sql = "`test`";
        let res = table(sql);
        assert!(res.is_ok());
        let out = res.unwrap().1;
        assert_eq!("test", format!("{}", out));
        assert_eq!(out, Table(String::from("test")));
    }

    #[test]
    fn table_quoted_brackets() {
        let sql = "⟨test⟩";
        let res = table(sql);
        assert!(res.is_ok());
        let out = res.unwrap().1;
        assert_eq!("test", format!("{}", out));
        assert_eq!(out, Table(String::from("test")));
    }
}
