use super::comment::{mightbespace, shouldbespace};
use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while_m_n},
    character::{complete::char, is_alphanumeric},
    error::Error,
    multi::many1,
    IResult,
};
use std::ops::RangeBounds;

pub fn colons(i: &str) -> IResult<&str, ()> {
    let (i, _) = mightbespace(i)?;
    let (i, _) = many1(char(';'))(i)?;
    let (i, _) = mightbespace(i)?;
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

pub fn commasorspace(i: &str) -> IResult<&str, ()> {
    alt((commas, shouldbespace))(i)
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

pub fn openbracket(i: &str) -> IResult<&str, ()> {
    let (i, _) = char('[')(i)?;
    let (i, _) = mightbespace(i)?;
    Ok((i, ()))
}

pub fn closebracket(i: &str) -> IResult<&str, ()> {
    let (i, _) = mightbespace(i)?;
    let (i, _) = char(']')(i)?;
    Ok((i, ()))
}

pub fn openchevron(i: &str) -> IResult<&str, ()> {
    let (i, _) = char('<')(i)?;
    let (i, _) = mightbespace(i)?;
    Ok((i, ()))
}

pub fn closechevron(i: &str) -> IResult<&str, ()> {
    let (i, _) = mightbespace(i)?;
    let (i, _) = char('>')(i)?;
    Ok((i, ()))
}

#[inline]
pub fn is_hex(chr: char) -> bool {
    chr.is_ascii_hexdigit()
}

#[inline]
pub fn is_digit(chr: char) -> bool {
    chr.is_ascii_digit()
}

#[inline]
pub fn val_u8(chr: u8) -> bool {
    is_alphanumeric(chr) || chr == b'_'
}

#[inline]
pub fn val_char(chr: char) -> bool {
    chr.is_ascii_alphanumeric() || chr == '_'
}

pub fn take_u64(i: &str) -> IResult<&str, u64> {
    let (i, v) = take_while(is_digit)(i)?;
    match v.parse::<u64>() {
        Ok(v) => Ok((i, v)),
        _ => Err(nom::Err::Error(Error::new(i, nom::error::ErrorKind::Digit))),
    }
}

pub fn take_u32_len(i: &str) -> IResult<&str, (u32, usize)> {
    let (i, v) = take_while(is_digit)(i)?;
    match v.parse::<u32>() {
        Ok(n) => Ok((i, (n, v.len()))),
        _ => Err(nom::Err::Error(Error::new(i, nom::error::ErrorKind::Digit))),
    }
}

pub fn take_digits(i: &str, n: usize) -> IResult<&str, u32> {
    let (i, v) = take_while_m_n(n, n, is_digit)(i)?;
    match v.parse::<u32>() {
        Ok(v) => Ok((i, v)),
        _ => Err(nom::Err::Error(Error::new(i, nom::error::ErrorKind::Digit))),
    }
}

pub fn take_digits_range(i: &str, n: usize, range: impl RangeBounds<u32>) -> IResult<&str, u32> {
    let (i, v) = take_while_m_n(n, n, is_digit)(i)?;
    match v.parse::<u32>() {
        Ok(v) if range.contains(&v) => Ok((i, v)),
        _ => Err(nom::Err::Error(Error::new(i, nom::error::ErrorKind::Digit))),
    }
}
