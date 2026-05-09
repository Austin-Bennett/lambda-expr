use crate::ast::ExprNode;
use crate::eval::bc::PseudoBuilder;
use crate::eval::integer::int_context::{IntEvalContextBuilder};
use crate::lexer::lexer::Tokens;
use std::time::{Duration, Instant};

macro_rules! time {
    { $($tk: tt)* } => {
        {
            let start = Instant::now();

            {
                $($tk)*
            }

            let end = Instant::now();

            end - start
        }
    };
}


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

pub fn run_a_thousand_times<F>(f: F) -> Duration where F: Fn() {
    let mut avg = 0u128;

    for _i in 0..1000 {
        avg += time!{
            f();
        }.as_nanos();
    }


    avg /= 1000;
    Duration::from_nanos_u128(avg)
}

#[test]
pub fn test_bc_evaluation() {
    let context = IntEvalContextBuilder::new()
        .add_variable("a".to_string(), 3)
        .add_function("abs".to_string(), abs)
        .build();
    let expr_str = "3 + a *   (6+2) + abs(-3)";

    let _compile_warm_up = context.bytecode_compile(expr_str).unwrap();

    let expr;
    let t = time! {
        expr = context.bytecode_compile(expr_str).unwrap();
    };

    println!("Compile time: {:?}", t);


    let dyn_expr;
    let t = time! {
         dyn_expr = context.bytecode_compile(expr_str).unwrap().to_dynamic();
    };
    println!("Compile time 2: {:?}", t);



    let eval = expr.eval();
    let t = run_a_thousand_times(
        || {
            expr.eval();
        }
    );
    println!("expr: {}", eval);
    println!("Eval time: {:?}\n", t);


    let eval = dyn_expr.eval();
    let t = run_a_thousand_times(
        || {
            dyn_expr.eval();
        }
    );

    println!("expr: {}", eval);
    println!("Eval time: {:?}", t);

}

#[test]
pub fn test_jit() {
    let context = IntEvalContextBuilder::new()
        .add_variable("a".to_string(), 3)
        .add_function("abs".to_string(), abs)
        .build();
    let expr_str = "3 + a *   (6+2) + abs(-3)";

    let _warm_compile = context.jit_compile(expr_str).unwrap();

    let expr;
    let t = time! {
        expr = context.jit_compile(expr_str).unwrap();
    };

    println!("Compile time: {:?}", t);


    let dyn_expr;
    let t = time! {
         dyn_expr = context.jit_compile(expr_str).unwrap().to_dynamic();
    };
    println!("Compile time 2: {:?}", t);


    let eval = expr.eval();
    let t = run_a_thousand_times(
        || {
            expr.eval();
        }
    );
    println!("result: {}", eval);
    println!("Eval time: {:?}\n", t);



    let eval = dyn_expr.eval();
    let t = run_a_thousand_times(
        || {
            dyn_expr.eval();
        }
    );

    println!("result: {}", eval);
    println!("Eval time: {:?}", t);
}

#[test]
pub fn compare_jit_bc() {
    println!("--BYTECODE EVALUATION--");
    test_bc_evaluation();

    print!("\n\n--JIT EVALUATION--");
    test_jit();
}