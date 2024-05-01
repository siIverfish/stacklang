use crate::{function::{RuntimeError, Token}, scanner::{Directive, ScanError}};

pub trait Executable {
    fn execute(self) -> Vec<Token>;
}

impl<I: Iterator<Item = Result<Token, ScanError>>> Executable for I {
    fn execute(self) -> Vec<Token> {
        let mut stack = ExecutionStack::default();

        for item in self {
            println!("Pushing element {:?}...", &item);
            stack.push(item);
            dbg!(&stack);
        }

        stack.stack
    }
}

#[derive(Default, Debug)]
pub struct ExecutionStack {
    child: Option<Box<ExecutionStack>>,
    stack: Vec<Token>,
}

// The message the "child" execution stack sends when it is done executing
// -- ready to be folded back into the parent system.
struct Filicide;

impl ExecutionStack {
    fn apply_functions(&mut self) {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(top_element), Some(function)) => {
                let result = function.apply(top_element);

                match result {
                    Ok(newly_created_token) => self.stack.push(newly_created_token),
                    Err(RuntimeError::DoNotExecute((function, argument))) => {
                        // put them back on the stack -- they didn't work :(
                        self.stack.append(&mut vec![function, argument])
                    },
                    Err(other_err) => panic!("Could not handle error: {other_err:?}"),
                }
            },
            (Some(top_element), None)           => self.stack.push(top_element),
            _ => {}
        }
    }

    fn push_token(&mut self, elt: Token) -> Option<Filicide> {
        self.stack.push(elt);
        self.apply_functions();
        None
    }

    fn handle_err(&mut self, err: ScanError) -> Option<Filicide> {
        match err {
            ScanError::Directive(Directive::End)   => Some(Filicide),
            ScanError::Directive(Directive::Begin) => {
                self.child = Some(Box::new(Self::default()));
                None
            },
            // Things like ParseIntError and unknown idents are
            // irrecoverable... for now.
            other => panic!("{other:?}"),
        }
    }

    fn push_result(&mut self, result: Result<Token, ScanError>) -> Option<Filicide> {
        match result {
            Ok(token)    => self.push_token(token),
            Err(err) => self.handle_err(err)
        }
    }

    fn push_inner(&mut self, elt: Result<Token, ScanError>) -> Option<Filicide> {
        if let Some(found_child) = self.child.as_mut() {
            let filicide_order = found_child.push_inner(elt);

            if let Some(Filicide) = filicide_order {
                self.stack.append(&mut found_child.stack);
                self.apply_functions();
                self.child = None;
            }

            None
        } else {
            self.push_result(elt)
        }
    }

    fn push(&mut self, elt: Result<Token, ScanError>) {
        // Discard the `Option<Filicide>` result value that is only used
        // within `ExecutionStack` logic.
        self.push_inner(elt);
    }
}