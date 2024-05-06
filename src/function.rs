use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, derive_more::From)]
pub enum Token {
    Function(Function),
    Data(Box<dyn std::any::Any + Sync + Send>),
}

impl Token {
    pub(crate) fn apply(self, rhs: Token) -> Result<Token, (Token, Token)> {
        match self {
            // this clone should be very cheap, as functions are just `Arc`s
            Token::Function(ref x) => (x.0.clone().f)(self, rhs),
            Token::Data(_) => match rhs {
                Token::Data(_) => Err((self, rhs)),
                other => other.apply(self),
            },
        }
    }

    pub fn downcast<T: 'static>(self) -> Result<T, Token> {
        let data: Box<dyn std::any::Any + Send + Sync> = match self {
            Token::Data(data) => data,
            other => return Err(other),
        };

        data.downcast::<T>()
            .map(|x| *x)
            .map_err(Token::Data)
    }

    pub fn downcast_arg<T: 'static>(self, arg: Token) -> Result<(Token, T), (Token, Token)> {
        match arg.downcast::<T>() {
            Ok(downcasted_value)             => Ok((self, downcasted_value)),
            Err(failed_downcasted_value) => Err((self, failed_downcasted_value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function(Arc<FunctionInner>);

struct FunctionInner {
    meta: FunctionMetadata,
    f: Box<dyn Fn(Token, Token) -> Result<Token, (Token, Token)> + Send + Sync>,
}


#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) struct FunctionMetadata {
    pub(crate) name: String,
}

impl Default for FunctionMetadata {
    fn default() -> Self {
        Self { name: "<unnamed>".into() }
    }
}

impl Debug for FunctionInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.meta.fmt(f)
    }
}

impl Function {
    pub(crate) fn with_metadata<F: Fn(Token, Token) -> Result<Token, (Token, Token)> + Send + Sync + 'static>(f: F, meta: FunctionMetadata) -> Self {
        Self ( Arc::new(FunctionInner { meta, f: Box::new(f) }) )
    }

    pub(crate) fn from_fn<F: Fn(Token, Token) -> Result<Token, (Token, Token)> + Send + Sync + 'static>(f: F) -> Self {
        Self ( Arc::new(FunctionInner { meta: Default::default(), f: Box::new(f) }) )
    }
}