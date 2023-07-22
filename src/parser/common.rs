use super::comment::mightbespace;
use nom::{
    character::{complete::{char, multispace0}, is_alphanumeric},
    multi::many1,
    IResult,
};

pub fn colons(i: &str) -> IResult<&str, ()> {
    let (i, _) = multispace0(i)?;
    let (i, _) = many1(char(';'))(i)?;
    let (i, _) = multispace0(i)?;
    Ok((i, ()))
}

pub fn commas(i: &str) -> IResult<&str, ()> {
    let (i, _) = mightbespace(i)?;
    let (i, _) = char(',')(i)?;
    let (i, _) = mightbespace(i)?;
    Ok((i, ()))
}

pub fn verbar(i: &str) -> IResult<&str, ()> {
    let (i, _) = mightbespace(i)?;
    let (i, _) = char('|')(i)?;
    let (i, _) = mightbespace(i)?;
    Ok((i, ()))
}

pub fn openparentheses(i: &str) -> IResult<&str, ()> {
    let (i, _) = char('(')(i)?;
    let (i, _) = mightbespace(i)?;
    Ok((i, ()))
}

pub fn closeparentheses(i: &str) -> IResult<&str, ()> {
    let (i, _) = mightbespace(i)?;
    let (i, _) = char(')')(i)?;
    Ok((i, ()))
}

pub fn openbraces(i: &str) -> IResult<&str, ()> {
    let (i, _) = char('{')(i)?;
    let (i, _) = mightbespace(i)?;
    Ok((i, ()))
}

pub fn closebraces(i: &str) -> IResult<&str, ()> {
    let (i, _) = mightbespace(i)?;
    let (i, _) = char('}')(i)?;
    Ok((i, ()))
}

#[inline]
pub fn val_u8(chr: u8) -> bool {
    is_alphanumeric(chr) || chr == b'_'
}

#[inline]
pub fn val_char(chr: char) -> bool {
    chr.is_ascii_alphanumeric() || chr == '_'
}
