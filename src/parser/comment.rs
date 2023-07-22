use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char, multispace0, multispace1, not_line_ending},
    multi::many1,
    IResult,
};

pub fn mightbespace(i: &str) -> IResult<&str, ()> {
    let (i, _) = alt((comment, blank))(i)?;
    Ok((i, ()))
}

pub fn shouldbespace(i: &str) -> IResult<&str, ()> {
    let (i, _) = alt((comment, space))(i)?;
    Ok((i, ()))
}

pub fn comment(i: &str) -> IResult<&str, ()> {
    let (i, _) = multispace0(i)?;
    let (i, _) = many1(alt((block, slash, dash, hash)))(i)?;
    let (i, _) = multispace0(i)?;
    Ok((i, ()))
}

pub fn mightbecomment(i: &str) -> IResult<&str, Vec<&str>> {
    let (i, comments) = alt((comment_wanted, blank_))(i)?;
    Ok((i, comments))
}

pub fn comment_wanted(i: &str) -> IResult<&str, Vec<&str>> {
    let (i, _) = multispace0(i)?;
    let (i, comments) = many1(alt((block, slash, dash, hash)))(i)?;
    let (i, _) = multispace0(i)?;
    Ok((i, comments))
}

pub fn block(i: &str) -> IResult<&str, &str> {
    let (i, _) = multispace0(i)?;
    let (i, _) = char('/')(i)?;
    let (i, _) = char('*')(i)?;
    let (i, comment) = take_until("*/")(i)?;
    let (i, _) = char('*')(i)?;
    let (i, _) = char('/')(i)?;
    let (i, _) = multispace0(i)?;
    Ok((i, comment.trim()))
}

pub fn slash(i: &str) -> IResult<&str, &str> {
    let (i, _) = multispace0(i)?;
    let (i, _) = char('/')(i)?;
    let (i, _) = char('/')(i)?;
    let (i, comment) = not_line_ending(i)?;
    Ok((i, comment.trim()))
}

pub fn dash(i: &str) -> IResult<&str, &str> {
    let (i, _) = multispace0(i)?;
    let (i, _) = char('-')(i)?;
    let (i, _) = char('-')(i)?;
    let (i, comment) = not_line_ending(i)?;
    Ok((i, comment.trim()))
}

pub fn hash(i: &str) -> IResult<&str, &str> {
    let (i, _) = multispace0(i)?;
    let (i, _) = char('#')(i)?;
    let (i, comment) = not_line_ending(i)?;
    Ok((i, comment.trim()))
}

fn blank(i: &str) -> IResult<&str, ()> {
    let (i, _) = multispace0(i)?;
    Ok((i, ()))
}

fn space(i: &str) -> IResult<&str, ()> {
    let (i, _) = multispace1(i)?;
    Ok((i, ()))
}

fn blank_(i: &str) -> IResult<&str, Vec<&str>> {
    let (i, _) = multispace0(i)?;
    Ok((i, vec![]))
}
