use std::num::ParseIntError;

use crate::tokenizer::Lexeme;
use crate::function::Token;
use crate::ops::Builtins;
use ScanError::*;

#[derive(Debug)]
pub enum Directive {
    Begin,
    End,
}

#[derive(thiserror::Error, Debug)]
pub enum ScanError {
    #[error("unknown ident: {0}")]
    Undefined(String),
    #[error("Could not parse integer: {0}")]
    CouldNotParseInteger(#[from] ParseIntError),
    #[error("directive: {0:?}")]
    Directive(Directive),
}

pub trait Processable {
    fn process(self) -> impl Iterator<Item = Result<Token, ScanError>>;
}

impl<I: Iterator<Item = Lexeme>> Processable for I {
    fn process(self) -> impl Iterator<Item = Result<Token, ScanError>> {
        self.map(|lexeme| match lexeme {
                Lexeme::Ident(string)   => match string.as_ref() {
                    "(" => Err(Directive(Directive::Begin)),
                    ")" => Err(Directive(Directive::End  )),

                    other => Builtins::get(other)
                        .ok_or(Undefined(string)),
                }

                Lexeme::Numeric(string) => 
                    string.parse::<i64>()
                        .map(Token::from)
                        .map_err(CouldNotParseInteger),
            })
    }
}