pub mod parsers;
pub mod lexer;

use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::operator::Operator;

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


pub trait Parser {
    fn parse(&self, input: &str) -> Option<(Token, usize)>;
}
