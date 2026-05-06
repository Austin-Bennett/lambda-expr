use std::ops::Deref;
use std::sync::Arc;
use anyhow::Error;
use lazy_static::lazy_static;
use crate::lexer::{Parser, Token};
use crate::operator::Operator;

pub struct IdentifierParser;

impl IdentifierParser {
    pub fn can_start_with(c: char) -> bool {
        c.is_alphabetic() || c =='_'
    }

    pub fn is_identifier_character(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}

impl Parser for IdentifierParser {
    fn parse(&self, input: &str) -> Option<(Token, usize)> {
        let mut chars = input.chars();

        let first = chars.next()?;

        if !Self::can_start_with(first) {
            return None;
        }

        let mut res = String::new();
        res.push(first);

        while let Some(c) = chars.next() && Self::is_identifier_character(c) {
            res.push(c);
        }

        let l = res.len();

        Some((Token::Ident(res), l))
    }
}


//supports decimal, duotrigesimal (0t) hex (0x) octal (0o) binary (0b)
//todo: support any other number system by using the prefix 0nNN, binary is 0n02, ternary is 0n03,
//todo: duotrigesimal is 0n32, NN must be between 2 and 32
pub struct IntegerParser;

impl IntegerParser {
    fn convert_digit(c: char, base: i64) -> Option<i64> {
        c.to_digit(base as u32).map(|v| v as i64)
    }
}

impl Parser for IntegerParser {
    fn parse(&self, input: &str) -> Option<(Token, usize)> {
        let mut base: i64 = 10;
        let mut len = 0;

        if input.starts_with("0t") {
            base = 32;
            len = 2;
        } else if input.starts_with("0x") {
            base = 16;
            len = 2;
        } else if input.starts_with("0o") {
            base = 8;
            len = 2;
        } else if input.starts_with("0b") {
            base = 2;
            len = 2;
        } //todo 0n prefix

        if input.len() <= len && len > 0 {
            return Some((Token::Error(Arc::new(Error::msg("Expected number after prefix"))), len));
        }

        let prefix_len = len;
        let mut res = 0i64;
        let mut chars = input[len..].chars();

        while let Some(c) = chars.next() {
            if c == '.' {
                return None;
            }
            let Some(dig) = Self::convert_digit(c, base) else {
                break;
            };
            len += c.len_utf8();
            res = res * base + dig;
        }

        // no digits consumed — not an integer (or error if a prefix was matched)
        if len == prefix_len {
            if prefix_len > 0 {
                return Some((Token::Error(Arc::new(Error::msg("Expected number after prefix"))), prefix_len));
            }
            return None;
        }

        Some((Token::Integer(res), len))
    }
}

pub struct FloatParser;

impl Parser for FloatParser {
    fn parse(&self, input: &str) -> Option<(Token, usize)> {
        let mut len = 0;
        let mut res = 0.0f64;

        let mut chars = input.chars().peekable();

        while let Some(c) = chars.peek() {
            if !c.is_numeric() {
                break;
            }
            let Some(c) = chars.next() else { unreachable!() };
            let dig = c.to_digit(10).unwrap();

            res = res * 10.0 + dig as f64;
            len += c.len_utf8();
        }

        if let Some('.') = chars.peek() {
            chars.next();
            len += '.'.len_utf8();

            let mut divisor = 10.0f64;

            while let Some(c) = chars.peek() {
                if !c.is_numeric() {
                    break;
                }
                let c = chars.next().unwrap();
                let dig = c.to_digit(10).unwrap();

                res += dig as f64 / divisor;
                divisor *= 10.0;
                len += c.len_utf8();
            }
        }

        if let Some('e') = chars.peek().map(|c| c.to_lowercase().next().unwrap_or('\0')) {
            chars.next();
            len += 'e'.len_utf8();
            let Some((Token::Float(power), l)) = self.parse(&input[len..]) else {
                return Some((Token::Error(Arc::new(Error::msg("Expected number after 'e'"))), len));
            };

            res = res * 10f64.powf(power);
            len += l;
        }

        if len == 0 {
            return None;
        }

        Some((Token::Float(res), len))
    }
}

lazy_static!{
    pub static ref OPERATORS: Vec<Operator> = {
        let mut res = vec![
            Operator::ADD,
            Operator::SUB,
            Operator::MUL,
            Operator::DIV,
            Operator::POW,
        ];

        res.sort_by(|c1, c2| {
            c2.token.len().cmp(&c1.token.len())
        });

        res
    };
}

pub struct OperatorParser;

impl Parser for OperatorParser {
    fn parse(&self, input: &str) -> Option<(Token, usize)> {
        for o in OPERATORS.deref() {
            if input.starts_with(o.token) {
                return Some((Token::Operator(*o), o.token.len()));
            }
        }

        None
    }
}

pub struct SingleCharParser<const C: char> { tk: Token }


impl<const C: char> SingleCharParser<C> {
    pub const fn new(tk: Token) -> Self {
        Self{ tk }
    }
}

impl<const C: char> Parser for SingleCharParser<C> {
    fn parse(&self, input: &str) -> Option<(Token, usize)> {
        if input.starts_with(C) {
            return Some((self.tk.clone(), C.len_utf8()));
        }
        None
    }
}
