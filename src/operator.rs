use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Debug)]
/// Describes how tightly an operator binds to its operands.
///
/// Left and right binding powers drive the Pratt parser's precedence climbing:
/// a higher right-hand power makes the operator right-associative.
pub enum BindingPower {
    #[allow(dead_code)]
    Unary,
    Binary(u8, u8),
    BinaryOrUnary(u8, u8),
}

impl BindingPower {
    pub const ADDITIVE_BP: BindingPower = BindingPower::Binary(0, 1);
    pub const MULTIPLICATIVE_BP: BindingPower = BindingPower::Binary(2, 3);
    pub const POWER_BP: BindingPower = BindingPower::Binary(4, 5);


    pub const fn effective_bp(&self) -> (u8, u8) {
        match self {
            BindingPower::Unary => (0, 255), //pulls on its right side the highest
            BindingPower::Binary(lbp, rbp) => (*lbp, *rbp),
            BindingPower::BinaryOrUnary(lbp, rbp) => (*lbp, *rbp)
        }
    }

    pub const fn lbp(&self) -> u8 {
        self.effective_bp().0
    }


    pub const fn rbp(&self) -> u8 {
        self.effective_bp().1
    }

    pub fn is_unary(&self) -> bool {
        match self {
            BindingPower::Unary => true,
            BindingPower::Binary(_, _) => false,
            BindingPower::BinaryOrUnary(_, _) => true,
        }
    }

    #[allow(dead_code)]
    pub fn is_binary(&self) -> bool {
        match self {
            BindingPower::Unary => false,
            BindingPower::Binary(_, _) => true,
            BindingPower::BinaryOrUnary(_, _) => true,
        }
    }

    //converts self to a binary binding power
    pub const fn to_binary_unary(&self) -> BindingPower {
        let (l, r) = self.effective_bp();
        BindingPower::BinaryOrUnary(l, r)
    }
}

#[derive(Copy, Clone)]
/// A static operator definition pairing a token string with its binding power.
///
/// Pre-defined constants (`ADD`, `SUB`, `MUL`, `DIV`, `POW`) cover all
/// operators recognised by the expression language.
pub struct Operator {
    pub token: &'static str,
    pub bp: BindingPower,
}

impl Debug for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.token)
    }
}

impl Operator {

    pub const ADD: Operator = Operator::new("+", BindingPower::ADDITIVE_BP.to_binary_unary());
    pub const SUB: Operator = Operator::new("-", BindingPower::ADDITIVE_BP.to_binary_unary());
    pub const MUL: Operator = Operator::new("*", BindingPower::MULTIPLICATIVE_BP);
    pub const DIV: Operator = Operator::new("/", BindingPower::MULTIPLICATIVE_BP);
    pub const POW: Operator = Operator::new("**", BindingPower::POWER_BP);

    pub const fn new(tk: &'static str, bp: BindingPower) -> Self {
        Self{
            token: tk,
            bp
        }
    }
}