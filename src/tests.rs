use crate::ast::ExprNode;
use crate::eval::bc::PseudoBuilder;
use crate::eval::integer::int_context::{BcIntExpression, IntEvalContext, IntEvalContextBuilder, IntExpr};
use crate::lexer::lexer::Tokens;

#[test]
pub fn test_tokenization() {
    //pure integer expression
    let expr = "3 + a*   (6+2)";
    let tokens = Tokens::new(expr);
    let expected = ["3", "+", "a", "*", "(", "6", "+", "2", ")"];

    for (index, (i, j)) in tokens.zip(expected).enumerate() {
        if format!("{:?}", i) != j {
            println!("Error found [{}]: {:?} != {}", index, i, j);
            panic!();
        }
    }

    //expression with floating point numbers
    let expr = "3.33e2.4 + a *   (67.69+2.2)";
    let tokens = Tokens::new(expr);
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
    let expr = "3 + a*   (6+2)";
    let tokens = Tokens::new(expr);
    let expr = ExprNode::try_from(tokens).unwrap();

    assert_eq!(format!("{:?}", expr), "(3 + (a * (6 + 2)))");

    let expr = "3.33e2.4 + a *   (67.69+2.2)";
    let tokens = Tokens::new(expr);
    let expr = ExprNode::try_from(tokens).unwrap();

    assert_eq!(format!("{:?}", expr), "(836.45818169269 + (a * (67.69 + 2.2)))");

    println!("AST passed")
}




#[test]
pub fn test_pseudo_compiler() {
    let mut compiler = PseudoBuilder::new();

    let expr = "3 + a*   (6+2) + abs(-3)";
    let tokens = Tokens::new(expr);
    let expr = ExprNode::try_from(tokens).unwrap();

    let bc = compiler.compile_expr(&expr);

    for (i, op) in bc.iter().enumerate() {
        println!("{}: {:?}", i, op);
    }
}

fn abs(args: &[i64]) -> i64 {
    let Some(a) = args.get(0) else {
        return 0;
    };

    a.abs()
}

struct MyApp {
    context: IntEvalContext
}

impl MyApp {
    pub fn get_bc(&'_ self, s: &str) -> BcIntExpression<'_> {
        self.context.create_bytecode(s).unwrap()
    }
}

#[test]
pub fn test_int_context() {
    let context = IntEvalContextBuilder::new()
        .add_variable("a".to_string(), 3)
        .add_function("abs".to_string(), abs)
        .build();

    let expr = context.create_bytecode("3 + a *   (6+2) + abs(-3)").unwrap();

    let expr2 = context.create_bytecode("3 + a*   (6+2)").unwrap();
    
    println!("expr: {}", expr.eval());
    println!("expr2: {}", expr2.eval());

    let app = MyApp{
        context: IntEvalContextBuilder::new()
            .add_variable("a".to_string(), 3)
            .add_function("abs".to_string(), abs)
            .build(),
    };

    let expr = app.get_bc("3 + a *   (6+2) + abs(-3)");

    println!("expr: {}", expr.eval())

}