pub mod parsers;
pub mod lexer;

use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::operator::Operator;

/// A single lexical token produced by the tokenizer.
///
/// Covers every syntactic element an expression can contain: identifiers,
/// numeric literals, operators, grouping punctuation, and parse errors.
#[derive(Clone)]
pub enum Token {
    Ident(String),
    Integer(i64),
    Float(f64),
    Operator(Operator),
    OpenParen,
    CloseParen,
    Comma,

    Error(Arc<anyhow::Error>)
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => f.write_str(&ident),
            Token::Integer(int) => write!(f, "{}", int),
            Token::Float(float) => write!(f, "{}", float),
            Token::Operator(op) => write!(f, "{:?}", op),
            Token::OpenParen => f.write_str("("),
            Token::CloseParen => f.write_str(")"),
            Token::Comma => f.write_str(","),
            Token::Error(e) => write!(f, "{:?}", *e)
        }
    }
}


/// Parses a single token from the start of a string slice.
pub trait Parser {
    /// Attempts to consume a token from the beginning of `input`.
    ///
    /// Returns `Some((token, bytes_consumed))` on success, or `None` if this
    /// parser does not match the input at the current position.
    fn parse(&self, input: &str) -> Option<(Token, usize)>;
}
