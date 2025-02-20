use crate::parser::errors::ParserError;
use tinystr::{TinyStr4, TinyStr8};

pub fn parse_language_subtag(subtag: &str) -> Result<Option<TinyStr8>, ParserError> {
    let slen = subtag.len();

    let s: TinyStr8 = subtag.parse().map_err(|_| ParserError::InvalidLanguage)?;
    if slen < 2 || slen > 8 || slen == 4 || !s.is_ascii_alphabetic() {
        return Err(ParserError::InvalidLanguage);
    }

    let value = s.to_ascii_lowercase();

    if value == "und" {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub fn parse_script_subtag(subtag: &str) -> Result<TinyStr4, ParserError> {
    let slen = subtag.len();

    let s: TinyStr4 = subtag.parse().map_err(|_| ParserError::InvalidSubtag)?;
    if slen != 4 || !s.is_ascii_alphabetic() {
        return Err(ParserError::InvalidSubtag);
    }
    Ok(s.to_ascii_titlecase())
}

pub fn parse_region_subtag(subtag: &str) -> Result<TinyStr4, ParserError> {
    let slen = subtag.len();

    match slen {
        2 => {
            let s: TinyStr4 = subtag.parse().map_err(|_| ParserError::InvalidSubtag)?;
            if !s.is_ascii_alphabetic() {
                return Err(ParserError::InvalidSubtag);
            }
            Ok(s.to_ascii_uppercase())
        }
        3 => {
            if subtag.contains(|c: char| !c.is_ascii_digit()) {
                return Err(ParserError::InvalidSubtag);
            }
            Ok(subtag.parse().unwrap())
        }
        _ => Err(ParserError::InvalidSubtag),
    }
}

pub fn parse_variant_subtag(subtag: &str) -> Result<TinyStr8, ParserError> {
    let slen = subtag.len();

    if slen < 4 || slen > 8 {
        return Err(ParserError::InvalidSubtag);
    }

    if slen >= 5 && subtag.contains(|c: char| !c.is_ascii_alphanumeric()) {
        return Err(ParserError::InvalidSubtag);
    }

    if slen == 4
        && !subtag.as_bytes()[0].is_ascii_digit()
        && subtag[1..].contains(|c: char| !c.is_ascii_alphanumeric())
    {
        return Err(ParserError::InvalidSubtag);
    }

    let s: TinyStr8 = subtag.parse().unwrap();

    Ok(s.to_ascii_lowercase())
}
