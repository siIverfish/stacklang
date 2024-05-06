use crate::{function::Token, scanner::Directive};

pub trait Executable {
    fn execute(self) -> Vec<Token>;
}

impl<I: Iterator<Item = Result<Token, Directive>>> Executable for I {
    fn execute(self) -> Vec<Token> {
        let mut stack = ExecutionStack::default();

        for item in self {
            println!("Pushing element {:?}...", &item);
            stack.push_newly_scanned(item);
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

/// The message the "child" execution stack sends when it is done executing
/// -- ready to be folded back into the parent system.
/// TODO: This will eventually be a fully `Message` enum for conveying more information
/// to the parent than just `kill me`
struct Filicide;

impl ExecutionStack {
    /// Some functions will return two tokens in the form Token::Data((a, b)).
    /// In this case, both will be added independently to the stack.
    /// 
    /// This is a special case that is not *necessarily* needed -- one other
    /// solution would be to have a `Pair` type (like pure lc). However,
    /// the `Token` type is using a `Box<dyn Any>` and so cannot derive `Clone`.
    /// If, in the future, I replace the box with a concrete enum tree spelling out all possible data values,
    /// it might be more reasonable to use `Pair`.
    fn push_application_result_token(&mut self, t: Token) {
        match t.downcast::<(Token, Token)>() {
            Ok((a, b)) => {
                self.push_application_result_token(a);
                self.push_application_result_token(b);
            },

            Err(just_one_token) => self.push_monotoken(just_one_token),
        }
    }
    
    /// As opposed to `push_application_result_token`, which contains special logic for 
    /// unpairing `Token::Data((a, b))`s
    fn push_monotoken(&mut self, new_token: Token) {
        // self.stack.push(elt);
        // self.apply_functions();

        match self.stack.pop() {
            Some(function) => 
                match function.apply(new_token) {
                    Ok(result) => self.push_application_result_token(result),
                    Err((function, argument)) => 
                        // put them back on the stack -- they didn't work :(
                        // importantly, this bypasses the pushing-then-calling system 
                        // that would usually cause an infinite loop here
                        // (because the failed application would repeat forever)
                        self.stack.append(&mut vec![function, argument]),
                }
            None => self.stack.push(new_token),
        }
    }

    fn handle_directive(&mut self, directive: Directive) -> Result<(), Filicide> {
        match directive {
            Directive::End   => Err(Filicide),
            Directive::Begin => {
                self.child = Some(Default::default());
                Ok(())
            },
        }
    }

    fn push_newly_scanned_recursive(&mut self, elt: Result<Token, Directive>) -> Result<(), Filicide> {
        // Take the child and make it do work.
        if let Some(mut child) = self.child.take() {
            if let Err(Filicide) = child.push_newly_scanned_recursive(elt) {
                // When the child stack receives a ")", its elements are moved back into the parent stack.
                // They must be added individually so that function applications can be processed.
                for token in child.stack {
                    self.push_monotoken(token);
                }
            } else {
                self.child = Some(child)
            }
        } else {
            match elt {
                Ok(token)          =>        self.push_monotoken(token),
                Err(directive) => return self.handle_directive(directive),
            }
        }

        Ok(())
    }

    fn push_newly_scanned(&mut self, elt: Result<Token, Directive>) {
        // Discard the `Result<(), Filicide>` value that is only used
        // within `ExecutionStack` logic.
        let _ = self.push_newly_scanned_recursive(elt);
    }
}