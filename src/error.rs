use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct ParseError {
    message: String,
    line_number: usize,
}

impl ParseError {
    pub fn new(message: &str, line_number: usize) -> Self {
        ParseError {
            message: message.to_owned(),
            line_number,
        }
    }

    pub fn boxed(message: &str, line_number: usize) -> Box<Self> {
        Box::new(Self::new(message, line_number))
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError [line {}]: {}", self.line_number, self.message)
    }
}

impl Error for ParseError {}