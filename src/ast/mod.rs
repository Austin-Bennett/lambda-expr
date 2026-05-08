use std::fmt::{Debug, Formatter, Write};
use std::iter::Peekable;
use std::process::id;
use std::sync::Arc;
use anyhow::Error;
use crate::lexer::lexer::Tokens;
use crate::lexer::Token;
use crate::operator::BindingPower;

#[derive(Clone)]
pub enum BinaryOp {
    ADD,
    SUB,
    MUL,
    DIV,
    POW,
}

impl Debug for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::ADD => f.write_str("+"),
            BinaryOp::SUB => f.write_str("-"),
            BinaryOp::MUL => f.write_str("*"),
            BinaryOp::DIV => f.write_str("/"),
            BinaryOp::POW => f.write_str("**"),
        }
    }
}


#[derive(Clone)]
pub enum UnaryOp {
    Def, //+
    Neg, //-
}

impl Debug for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Def => f.write_str("+"),
            UnaryOp::Neg => f.write_str("-"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BinaryExprNode {
    pub lhs: ExprNode,
    pub rhs: ExprNode,

    pub operator: BinaryOp
}


#[derive(Clone, Debug)]
pub struct UnaryExprNode {
    pub node: ExprNode,
    pub op: UnaryOp,
}

#[derive(Clone, Debug)]
pub struct CallExprNode {
    pub caller: ExprNode,
    pub args: Vec<ExprNode>,
}

#[derive(Clone)]
pub enum ExprNode {
    Ident(String),
    Integer(i64),
    Float(f64),
    Binary(Box<BinaryExprNode>),
    Unary(Box<UnaryExprNode>),
    Call(Box<CallExprNode>)
}

impl Debug for ExprNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprNode::Ident(ident) => { f.write_str(ident) }
            ExprNode::Integer(i) => { write!(f, "{}", i) }
            ExprNode::Float(ft) => { write!(f, "{}", ft) }
            ExprNode::Binary(b) => {
                write!(f, "({:?} {:?} {:?})", b.lhs, b.operator, b.rhs)
            }
            ExprNode::Unary(u) => {
                write!(f, "{:?}{:?}", u.op, u.node)
            }
            ExprNode::Call(call) => {
                write!(f, "{:?}(", call.caller)?;
                
                
                for (i, arg) in call.args.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{:?}", arg)?;
                }
                f.write_str(")")?;
                
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug)]
enum ExprRes {
    Expr(ExprNode),
    None,
    Err(Arc<anyhow::Error>)
}

impl ExprNode {

    fn __parse_parentheses(tks: &mut Peekable<Tokens>) -> Result<Vec<ExprNode>, Arc<Error>> {
        //optional '(' since some contexts already consumed it
        if let Some(Token::OpenParen) = tks.peek() {
            tks.next();
        };

        let mut res = vec![];

        loop {
            match tks.peek() {
                None | Some(Token::CloseParen) => { tks.next(); break; }
                _ => {}
            }

            let e = match Self::__parse(tks, 0) {
                ExprRes::Expr(e) => e,
                ExprRes::None => break,
                ExprRes::Err(e) => return Err(e)
            };

            res.push(e);

            if let Some(Token::Comma) = tks.peek() {
                tks.next();
            }
        }

        Ok(res)
    }


    fn __parse(tks: &mut Peekable<Tokens>, min_bp: u8) -> ExprRes {
        let Some(lhs) = tks.next() else {
            return ExprRes::None;
        };

        let mut lhs = match lhs {
            Token::Ident(ident) => ExprNode::Ident(ident),
            Token::Integer(i) => ExprNode::Integer(i),
            Token::Float(f) => ExprNode::Float(f),
            Token::Operator(o) => {
                //must be a unary operator
                if !o.bp.is_unary() {
                    return ExprRes::Err(Arc::new(Error::msg(format!("Expected unary operator, got operator '{}'", o.token))))
                }

                let rhs = match Self::__parse(tks, o.bp.rbp()) {
                    ExprRes::Expr(e) => e,
                    ExprRes::None => return ExprRes::Err(Arc::new(
                        Error::msg(format!("Expected expression after '{}'", o.token)))),
                    e => return e,
                };

                let uop = match o.token {
                    "+" => UnaryOp::Def,
                    "-" => UnaryOp::Neg,
                    _ => unreachable!()
                };



                ExprNode::Unary(
                    Box::new(
                        UnaryExprNode{
                            node: rhs,
                            op: uop
                        }
                    )
                )
            }
            Token::OpenParen => {
                //parse in the parentheses
                let mut exprs = match Self::__parse_parentheses(tks) {
                    Ok(v) => v,
                    Err(e) => return ExprRes::Err(e),
                };

                if exprs.len() == 0 {
                    exprs.push(ExprNode::Integer(0));
                }
                if exprs.len() != 1 {
                    return ExprRes::Err(Arc::new(Error::msg("Tuple expression was not expected here!")));
                }

                let expr = exprs.swap_remove(0);

                expr
            }
            Token::CloseParen => {
                return ExprRes::Err(Arc::new(Error::msg("Unmatched closing parentheses!")));
            }
            Token::Comma => {
                return ExprRes::Err(Arc::new(Error::msg("Extraneous ','")));
            }
            Token::Error(e) => { return ExprRes::Err(e) }
        };


        loop {
            //we expect an operator next
            let next = match tks.peek() {
                Some(v) => v,
                None => break,
            };

            match next {
                Token::Ident(_) | Token::Integer(_) | Token::Float(_) => {
                    let rhs = match Self::__parse(tks, BindingPower::MULTIPLICATIVE_BP.rbp()) {
                        ExprRes::Expr(e) => e,
                        ExprRes::None => unreachable!(),
                        e => return e,
                    };

                    lhs = ExprNode::Binary(
                        Box::new(
                            BinaryExprNode{
                                lhs,
                                rhs,
                                operator: BinaryOp::MUL,
                            }
                        )
                    )
                }
                Token::OpenParen => {
                    let params = match Self::__parse_parentheses(tks) {
                        Ok(v) => v,
                        Err(e) => return ExprRes::Err(e),
                    };

                    lhs = ExprNode::Call(Box::new(
                        CallExprNode{
                            caller: lhs,
                            args: params,
                        }
                    ));
                }
                Token::Operator(op) => {
                    if op.bp.lbp() < min_bp {
                        break;
                    }

                    let Some(Token::Operator(op)) = tks.next() else { unreachable!() };

                    let rhs = match Self::__parse(tks, op.bp.rbp()) {
                        ExprRes::Expr(e) => e,
                        ExprRes::None => return ExprRes::Err(Arc::new(
                            Error::msg(format!("Expected expression after '{}'", op.token))
                        )),
                        e => return e,
                    };

                    match op.token {
                        "+" => lhs = ExprNode::Binary(
                            Box::new(
                                BinaryExprNode{
                                    lhs,
                                    rhs,
                                    operator: BinaryOp::ADD,
                                }
                            )
                        ),
                        "-" => lhs = ExprNode::Binary(
                            Box::new(
                                BinaryExprNode{
                                    lhs,
                                    rhs,
                                    operator: BinaryOp::SUB,
                                }
                            )
                        ),
                        "*" => lhs = ExprNode::Binary(
                            Box::new(
                                BinaryExprNode{
                                    lhs,
                                    rhs,
                                    operator: BinaryOp::MUL,
                                }
                            )
                        ),
                        "/" => lhs = ExprNode::Binary(
                            Box::new(
                                BinaryExprNode{
                                    lhs,
                                    rhs,
                                    operator: BinaryOp::DIV,
                                }
                            )
                        ),
                        "**" => lhs = ExprNode::Binary(
                            Box::new(
                                BinaryExprNode{
                                    lhs,
                                    rhs,
                                    operator: BinaryOp::POW,
                                }
                            )
                        ),
                        _ => unreachable!()
                    }
                }
                Token::CloseParen | Token::Comma => { break; }
                Token::Error(e) => return ExprRes::Err(e.clone())
            }
        }

        ExprRes::Expr(lhs)
    }
}

impl<'a> TryFrom<Tokens<'a>> for ExprNode {
    type Error = Arc<anyhow::Error>;

    fn try_from(value: Tokens) -> Result<ExprNode, Arc<anyhow::Error>> {
        let mut peek = value.peekable();
        match Self::__parse(&mut peek, 0) {
            ExprRes::Expr(e) => Ok(e),
            ExprRes::None => Ok(ExprNode::Integer(0)),
            ExprRes::Err(e) => Err(e)
        }
    }
}