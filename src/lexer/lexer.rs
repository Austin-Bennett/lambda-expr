use crate::lexer::{Parser, Token};
use crate::lexer::parsers::{FloatParser, IdentifierParser, IntegerParser, OperatorParser, SingleCharParser};


/// A lazy iterator that tokenizes an expression string on demand.
///
/// Skips unrecognized characters and tries each registered [`Parser`] in
/// priority order, yielding one [`Token`] per call to [`Iterator::next`].
pub struct Tokens<'str> {
    original: &'str str,
    pos: usize,
}

const PARSERS: &[&dyn Parser] = &[
    &IdentifierParser,
    &IntegerParser,
    &FloatParser,
    &OperatorParser,
    &SingleCharParser::<'('>::new(Token::OpenParen),
    &SingleCharParser::<')'>::new(Token::CloseParen),
    &SingleCharParser::<','>::new(Token::Comma),
];

impl<'str> Tokens<'str> {
    
    
    /// Creates a new token iterator over the given expression string.
    pub fn new(expr: &'str str) -> Self {
        Self{
            original: expr,
            pos: 0,
        }
    }
}


impl<'str> Iterator for Tokens<'str> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.original.len() {
            
            for p in PARSERS {
                if let Some((tk, len)) = p.parse(&self.original[self.pos..]) {
                    self.pos += len;
                    
                    return Some(tk);
                }
            }
            self.pos += self.original[self.pos..].chars().next()?.len_utf8()
            
        }
        
        None
    }
}