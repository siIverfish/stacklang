use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, derive_more::From)]
pub enum Token {
    Function(Function),
    Data(Box<dyn std::any::Any + Sync + Send>),
    Pair(Box<(Token, Token)>),
}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("invalid arguments -- do not apply, put the tokens back.")]
    ArgumentError(Token),
    #[error("put tokens back")]
    DoNotExecute((Token, Token)),
}

impl Token {
    pub(crate) fn apply(self, rhs: Token) -> Result<Token, RuntimeError> {
        match self {
            Token::Function(ref x) => 
                (x.0.as_ref().f)(rhs)
                .map_err(|e| {
                    if let RuntimeError::ArgumentError(rhs_token) = e {
                        RuntimeError::DoNotExecute((self, rhs_token))
                    } else {
                        e
                    }
                }),

            Token::Data(_) => match rhs {
                Token::Data(_) => Ok(Token::Pair(Box::new((self, rhs)))),
                other => other.apply(self),
            },
            Token::Pair(box (f, g)) => f.apply(g.apply(rhs)?),
        }
    }

    pub fn downcast<T: 'static>(self) -> Result<T, RuntimeError> {
        let data: Box<dyn std::any::Any + Send + Sync> = match self {
            Token::Data(data) => data,
            other => return Err(RuntimeError::ArgumentError(other)),
        };

        data.downcast::<T>()
            .map(|x| *x)
            .map_err(Token::Data)
            .map_err(RuntimeError::ArgumentError)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Function(Arc<FunctionInner>);

struct FunctionInner {
    meta: FunctionMetadata,
    f: Box<dyn Fn(Token) -> Result<Token, RuntimeError> + Send + Sync>,
}


#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) struct FunctionMetadata {
    pub(crate) name: String,
}

// Equality means that they are the same function.
// impl PartialEq for Function {
//     fn eq(&self, other: &Self) -> bool {
//         std::ptr::eq(self.0.as_ref(), other.0.as_ref())
//     }
// }
// impl Eq for Function {}

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

impl FunctionInner {
    fn from_fn<F: Fn(Token) -> Result<Token, RuntimeError> + Send + Sync + 'static>(f: F) -> Arc<Self> {
        Arc::new(FunctionInner { meta: Default::default(), f: Box::new(f) })
    }

    fn with_metadata<F: Fn(Token) -> Result<Token, RuntimeError> + Send + Sync + 'static>(f: F, meta: FunctionMetadata) -> Arc<Self> {
        Arc::new(FunctionInner { meta, f: Box::new(f) })
    }
}

impl<F: Fn(Token) -> Result<Token, RuntimeError> + Send + Sync + 'static> From<F> for Function {
    fn from(f: F) -> Self {
        Self ( FunctionInner::from_fn(f) )
    }
}

impl Function {
    pub(crate) fn with_metadata<F: Fn(Token) -> Result<Token, RuntimeError> + Send + Sync + 'static>(f: F, meta: FunctionMetadata) -> Self {
        Self ( FunctionInner::with_metadata(f, meta) )
    }
}

// impl<F: Fn(Token) -> Result<Token, RuntimeError> + Send + Sync + 'static> From<F> for Token {
//     fn from(value: F) -> Self {
//         Token::Function ( Function::from_fn(value) )
//     }
// }
