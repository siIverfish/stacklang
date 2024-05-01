use stacklang::tokenizer::Tokenizable;
use stacklang::scanner::Processable;
use stacklang::stack::Executable;

fn main() {
    let input = "(+ 1 1)";
    let result = input.tokens().process().execute();

    let only_result = result.into_iter().next().unwrap();

    dbg!(only_result.downcast::<i64>());
}
