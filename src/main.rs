use stacklang::tokenizer::Tokenizable;
use stacklang::scanner::Processable;
use stacklang::stack::Executable;

fn main() {
    let input = "+ 1 1 + 1 1 * 3 + 1 *";
    let result = input.tokens().process().execute();

    dbg!(&result);

    assert_eq!(result.len(), 1);

    let only_result = result.into_iter().next().unwrap();

    let _ = dbg!(only_result.downcast::<i64>());
}
