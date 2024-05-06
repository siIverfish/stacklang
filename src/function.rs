use std::fmt::Debug;
use std::sync::Arc;

structstruck::strike! {
    #[strikethrough[derive(Debug)]]
    pub enum Token {
        Function(Function),
        Datum(pub enum DataItem {
            Integer(i64),
            ScanError(crate::scanner::ScanError),
            Pair(Box<(Token, Token)>),
        }),
    }
}


impl Token {
    pub(crate) fn apply(self, rhs: Token) -> Result<Token, (Token, Token)> {
        match self {
            // this clone should be very cheap, as functions are just `Arc`s
            Token::Function(ref x) => (x.0.clone().f)(self, rhs),
            Token::Datum(_) => match rhs {
                Token::Datum(_) => Err((self, rhs)),
                other => other.apply(self),
            },
        }
    }

    // pub fn downcast<T: 'static>(self) -> Result<T, Token> {
    //     let data: Box<dyn std::any::Any + Send + Sync> = match self {
    //         Token::Datum(data) => data,
    //         other => return Err(other),
    //     };

    //     data.downcast::<T>()
    //         .map(|x| *x)
    //         .map_err(Token::Datum)
    // }

    // pub fn downcast_arg<T: 'static>(self, arg: Token) -> Result<(Token, T), (Token, Token)> {
    //     match arg.downcast::<T>() {
    //         Ok(downcasted_arg) => Ok ((self, downcasted_arg)),
    //         Err(arg)       => Err((self, arg)),
    //     }
    // }

    // pub fn of_data(data: impl Any + Send + Sync) -> Self {
    //     Token::Datum(Box::new(data))
    // }
}

#[derive(Debug, Clone)]
pub struct Function(Arc<FunctionInner>);

pub(crate) trait NativeFn = Fn(Token, Token) -> Result<Token, (Token, Token)> + Send + Sync + 'static;

struct FunctionInner {
    meta: FunctionMetadata,
    f: Box<dyn NativeFn>,
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
    pub(crate) fn with_metadata(f: impl NativeFn, meta: FunctionMetadata) -> Self {
        Self ( Arc::new(FunctionInner { meta, f: Box::new(f) }) )
    }

    pub(crate) fn from_fn(f: impl NativeFn) -> Self {
        Self ( Arc::new(FunctionInner { meta: Default::default(), f: Box::new(f) }) )
    }
}