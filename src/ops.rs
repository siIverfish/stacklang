use std::collections::HashMap;
use std::sync::OnceLock;

use crate::function::{Function, Token::*, Token, DataItem::*};
use crate::function::FunctionMetadata;

macro_rules! impl_tk_function {
    ($name:ident, $tk_type:ty, $f:path) => {
        pub(crate) fn $name(this: Token, addend_one_token: Token) -> Result<Token, (Token, Token)> {
            let Datum(Integer(addend_one)) = addend_one_token 
                else { return Err((this, addend_one_token)) };

            Ok(Token::Function(Function::from_fn(
                move |this: Token, addend_two_token: Token| {
                    let Datum(Integer(addend_two)) = addend_two_token 
                        else { return Err((this, addend_two_token)) };
                    let result = $f(addend_one, addend_two);
                    Ok(Datum(Integer(result)))
                }
            )))
        }
    };
}

macro_rules! impl_num_ops {
    ($name:ident, $number_type:ty) => {
        pub(crate) mod $name {
            use super::*;
        
            impl_tk_function! { add, $number_type, std::ops::Add::add }
            impl_tk_function! { sub, $number_type, std::ops::Sub::sub }
            impl_tk_function! { mul, $number_type, std::ops::Mul::mul }
            impl_tk_function! { div, $number_type, std::ops::Div::div }
        }
    };
}

pub(crate) mod tk {
    use super::*;

    // impl_num_ops! { i8, i8 }
    // impl_num_ops! { i16, i16 }
    // impl_num_ops! { i32, i32 }
    impl_num_ops! { i64, i64 }
}


static BUILTINS: OnceLock<Builtins> = OnceLock::new();

pub(crate) struct Builtins(HashMap<&'static str, Function>);

macro_rules! f {
    ($symbol:expr, $function:expr) => {
        (
            $symbol, 
            Function::with_metadata(
                $function, 
                FunctionMetadata { name: String::from($symbol) }
            )
        )
    };
}

impl Default for Builtins {
    fn default() -> Self {
        Self(HashMap::from([
            f!("+", tk::i64::add),
            f!("-", tk::i64::sub),
            f!("/", tk::i64::div),
            f!("*", tk::i64::mul),
        ]))
    }
}

impl Builtins {
    pub(crate) fn global() -> &'static Self {
        BUILTINS.get_or_init(Builtins::default)
    }

    pub(crate) fn get(value: &str) -> Option<Token> {
        Self::global().0
            .get(value)
            .cloned()
            .map(Token::Function)
    }
}



// macro_rules! impl_token_from_for {
//     ($($t:ty)*) => {
//         $(
//             impl From<$t> for Token {
//                 fn from(value: $t) -> Self {
//                     Token::Datum(Integer(value))
//                 }
//             }
//         )*
//     }
// }

// impl_token_from_for! { i8 i16 i32 i64 }