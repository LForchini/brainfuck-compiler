#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    PtrAdd(usize),
    PtrSub(usize),
    Add(usize),
    Sub(usize),
    LoopStart(usize),
    LoopEnd(usize),
    PutChar,
    GetChar,
}

pub fn lex(contents: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut loop_counter = 0;
    let mut active_loops = Vec::new();

    for c in contents.chars() {
        match c {
            '>' => tokens.push(Token::PtrAdd(1)),
            '<' => tokens.push(Token::PtrSub(1)),
            '+' => tokens.push(Token::Add(1)),
            '-' => tokens.push(Token::Sub(1)),
            '[' => {
                tokens.push(Token::LoopStart(loop_counter));
                active_loops.push(loop_counter);
                loop_counter += 1;
            }
            ']' => {
                let t = active_loops.pop().expect("Unmapped loop end");
                tokens.push(Token::LoopEnd(t));
            }
            '.' => tokens.push(Token::PutChar),
            ',' => tokens.push(Token::GetChar),
            _ => {}
        }
    }

    assert!(active_loops.is_empty(), "Unmatched loop start");

    tokens
}

pub fn optimise_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut tokens = tokens;

    loop {
        let next = optimise_tokens_inner(&tokens);
        if next == tokens {
            return next;
        }
        tokens = next;
    }
}

fn optimise_tokens_inner(tokens: &[Token]) -> Vec<Token> {
    let tokens = group_tokens(tokens);
    let tokens = cancel_out(&tokens);

    #[allow(clippy::let_and_return)]
    tokens
}

fn group_tokens(tokens: &[Token]) -> Vec<Token> {
    let mut new_tokens = vec![];

    let mut accumulator = None;
    for token in tokens {
        accumulator = match (token, accumulator) {
            (Token::PtrAdd(a), Some(Token::PtrAdd(b))) => Some(Token::PtrAdd(a + b)),
            (Token::PtrSub(a), Some(Token::PtrSub(b))) => Some(Token::PtrSub(a + b)),
            (Token::Add(a), Some(Token::Add(b))) => Some(Token::Add(a + b)),
            (Token::Sub(a), Some(Token::Sub(b))) => Some(Token::Sub(a + b)),

            (tok, Some(acc)) => {
                new_tokens.push(acc);
                Some(*tok)
            }
            (tok, None) => Some(*tok),
        };
    }

    if let Some(acc) = accumulator {
        new_tokens.push(acc);
    }

    new_tokens
}

fn cancel_out(tokens: &[Token]) -> Vec<Token> {
    let mut new_tokens = vec![];

    let mut accumulator = None;
    for token in tokens {
        log::trace!("(tok: {:?}, acc: {:?})", token, accumulator);
        accumulator = match (token, accumulator) {
            (Token::PtrAdd(a), Some(Token::PtrSub(b))) => match a.cmp(&b) {
                std::cmp::Ordering::Less => Some(Token::PtrAdd(a - b)),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(Token::PtrSub(b - a)),
            },
            (Token::PtrSub(a), Some(Token::PtrAdd(b))) => match a.cmp(&b) {
                std::cmp::Ordering::Less => Some(Token::PtrAdd(b - a)),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(Token::PtrSub(a - b)),
            },
            (Token::Add(a), Some(Token::Sub(b))) => match a.cmp(&b) {
                std::cmp::Ordering::Less => Some(Token::Add(a - b)),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(Token::Sub(b - a)),
            },
            (Token::Sub(a), Some(Token::Add(b))) => match a.cmp(&b) {
                std::cmp::Ordering::Less => Some(Token::Add(b - a)),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(Token::Sub(a - b)),
            },

            (tok, Some(acc)) => {
                new_tokens.push(acc);
                Some(*tok)
            }
            (tok, None) => Some(*tok),
        }
    }

    if let Some(acc) = accumulator {
        new_tokens.push(acc);
    }

    new_tokens
}
