use std::fmt::{Debug, Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};

use crate::parsing_error::ParsingErrorType::{JSONParseError, NetworkError};

#[derive(Debug)]
pub enum ParsingErrorType {
    NetworkError,
    JSONParseError,
    InvalidDataType(String),
    MissingKey,
}

impl Display for ParsingErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct ParsingError {
    error_type: ParsingErrorType,
    name: Option<String>,
}

impl ParsingError {
    pub(crate) fn new(error_type: ParsingErrorType, name: Option<String>) -> ParsingError {
        ParsingError {
            error_type,
            name,
        }
    }
}

impl From<reqwest::Error> for ParsingError {
    fn from(err: reqwest::Error) -> Self {
        ParsingError { error_type: NetworkError, name: Some(format!("{:?}", err)) }
    }
}

impl From<serde_json::Error> for ParsingError {
    fn from(err: serde_json::Error) -> Self {
        ParsingError { error_type: JSONParseError, name: Some(format!("{:?}", err)) }
    }
}

impl From<serde_plain::Error> for ParsingError {
    fn from(err: serde_plain::Error) -> Self {
        ParsingError { error_type: JSONParseError, name: Some(format!("{:?}", err)) }
    }
}

impl From<ParseIntError> for ParsingError {
    fn from(e: ParseIntError) -> Self {
        ParsingError { error_type: ParsingErrorType::InvalidDataType(format!("Failed to parse int")), name: Some(format!("{:?}", e)) }
    }
}

impl From<ParseFloatError> for ParsingError {
    fn from(e: ParseFloatError) -> Self {
        ParsingError { error_type: ParsingErrorType::InvalidDataType(format!("Failed to parse float")), name: Some(format!("{:?}", e)) }
    }
}

impl Debug for ParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{:?} for {:?} ", self.error_type, name)
        } else {
            write!(f, "{:?} for", self.error_type)
        }
    }
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{} for {} ", self.error_type, name)
        } else {
            write!(f, "{} for", self.error_type)
        }
    }
}

