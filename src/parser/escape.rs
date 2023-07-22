use super::common::val_u8;
use nom::character::is_digit;
use std::borrow::Cow;

const BACKTICK: char = '`';
const BACKTICK_ESC: &str = r#"\`"#;

#[inline]
/// Escapes an ident if necessary
pub fn escape_ident(s: &str) -> Cow<'_, str> {
    escape_numeric(s, BACKTICK, BACKTICK, BACKTICK_ESC)
}

#[inline]
pub fn escape_numeric<'a>(s: &'a str, l: char, r: char, e: &str) -> Cow<'a, str> {
    // Presume this is numeric
    let mut numeric = true;
    // Loop over each character
    for x in s.bytes() {
        // Check if character is allowed
        if !val_u8(x) {
            return Cow::Owned(format!("{l}{}{r}", s.replace(r, e)));
        }
        // Check if character is non-numeric
        if !is_digit(x) {
            numeric = false;
        }
    }
    // Output the id value
    match numeric {
        // This is numeric so escape it
        true => Cow::Owned(format!("{l}{}{r}", s.replace(r, e))),
        // No need to escape the value
        _ => Cow::Borrowed(s),
    }
}
