use itertools::Itertools;
use derive_more::From;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, From)]
pub enum Lexeme {
    Ident(String),
    #[from(ignore)]
    Numeric(String),
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'text>(std::iter::Peekable<std::str::Chars<'text>>);

impl<'text> Tokenizer<'text> {
    const fn is_numeric(&c: &char) -> bool {
        c.is_ascii_digit() || c == '.'
    }

    fn number(&mut self) -> Lexeme {
        Lexeme::Numeric(
            self.0
                .peeking_take_while(Tokenizer::is_numeric)
                .collect()
        )
    }

    fn word(&mut self) -> Lexeme {
        self.0
            .peeking_take_while(char::is_ascii_alphabetic)
            .collect::<String>()
            .into()
    }
}

impl<'text> Iterator for Tokenizer<'text> {
    type Item = Lexeme;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.0.peek()? {
             n if n.is_whitespace() => self.0.next().and_then(|_| self.next())?,
             n if Tokenizer::is_numeric(n) => self.number(),
             n if n.is_ascii_alphabetic() => self.word(),
            &_ => self.0.next().map(String::from).map(Lexeme::from)?,
        })
    }
} 

pub trait Tokenizable<'text> {
    fn tokens(self) -> Tokenizer<'text>;
}

impl<'text> Tokenizable<'text> for &'text str {
    fn tokens(self) -> Tokenizer<'text> {
        Tokenizer ( self.chars().peekable() )
    }
}

impl From<&str> for Lexeme {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}