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
}

pub trait Processable {
    fn process(self) -> impl Iterator<Item = Result<Token, Directive>>;
}

impl<I: Iterator<Item = Lexeme>> Processable for I {
    fn process(self) -> impl Iterator<Item = Result<Token, Directive>> {
        self.map(|lexeme| match lexeme {
                Lexeme::Ident(string)   => match string.as_ref() {
                    "(" => Err(Directive::Begin),
                    ")" => Err(Directive::End  ),

                    other => Ok(Builtins::get(other).unwrap_or_else(|| Token::of_data(Undefined(string))))
                }

                Lexeme::Numeric(string) => 
                    string.parse::<i64>()
                        .map(Token::from)
                        .or_else(|_| Ok(Token::of_data(CouldNotParseInteger)))
            })
    }
}