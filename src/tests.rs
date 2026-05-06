use std::process::abort;
use crate::ast::ExprNode;
use crate::lexer::lexer::Tokens;

#[test]
pub fn test_tokenization() {
    //pure integer expression
    let expr = "3 + a*   (6+2)".to_owned();
    let mut tokens = Tokens::new(expr);
    let expected = ["3", "+", "a", "*", "(", "6", "+", "2", ")"];

    for (index, (i, j)) in tokens.zip(expected).enumerate() {
        if format!("{:?}", i) != j {
            println!("Error found [{}]: {:?} != {}", index, i, j);
            panic!();
        }
    }

    //expression with floating point numbers
    let expr = "3.33e2.4 + a *   (67.69+2.2)".to_owned();
    let mut tokens = Tokens::new(expr);
    let expected = ["836.45818169269", "+", "a", "*", "(", "67.69", "+", "2.2", ")"];

    for (index, (i, j)) in tokens.zip(expected).enumerate() {
        if format!("{:?}", i) != j {
            println!("Error found [{}]: {:?} != {}", index, i, j);

            panic!();
        }
    }

    println!("Tokenization passed.")
}

#[test]
pub fn test_ast() {
    let expr = "3 + a*   (6+2)".to_owned();
    let tokens = Tokens::new(expr);
    let expr = ExprNode::try_from(tokens).unwrap();

    assert_eq!(format!("{:?}", expr), "(3 + (a * (6 + 2)))");

    let expr = "3.33e2.4 + a *   (67.69+2.2)".to_owned();
    let tokens = Tokens::new(expr);
    let expr = ExprNode::try_from(tokens).unwrap();

    assert_eq!(format!("{:?}", expr), "(836.45818169269 + (a * (67.69 + 2.2)))");

    println!("AST passed")
}