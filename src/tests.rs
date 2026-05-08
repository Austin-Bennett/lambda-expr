use std::time::{Duration, Instant};
use inkwell::{AddressSpace, OptimizationLevel};
use inkwell::context::Context;
use inkwell::passes::OptimizationLevel;
use inkwell::types::BasicType;
use crate::ast::ExprNode;
use crate::eval::bc::PseudoBuilder;
use crate::eval::integer::int_context::{BcIntExpression, IntEvalContext, IntEvalContextBuilder, IntExpr};
use crate::lexer::lexer::Tokens;

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



#[test]
pub fn test_int_context() {
    let context = IntEvalContextBuilder::new()
        .add_variable("a".to_string(), 3)
        .add_function("abs".to_string(), abs)
        .build();
    let expr_str = "3 + a *   (6+2) + abs(-3)";

    let expr;
    let t = time! {
        expr = context.create_bytecode(expr_str).unwrap();
    };

    println!("Compile time: {:?}", t);


    let dyn_expr;
    let t = time! {
         dyn_expr = context.create_bytecode(expr_str).unwrap().to_dynamic();
    };
    println!("Dynamic creation time: {:?}", t);

    let eval;
    let t = time!{
        eval = expr.eval();
    };
    println!("expr: {}", eval);
    println!("Eval time: {:?}\n", t);


    let eval;
    let t = time!{
        eval = dyn_expr.eval();
    };

    println!("expr2: {}", eval);
    println!("Dynamic eval time: {:?}", t);

}

#[test]
pub fn test_inkwell_dyn_compilation() {
    //manually compile the same expression as above (3 + a * (6 + 2) + abs(-3))
    let context = IntEvalContextBuilder::new()
        .add_variable("a".to_string(), 3)
        .add_function("abs".to_string(), abs)
        .build();

    let icontext = Context::create();

    let module = icontext.create_module("test_expr");

    let jit_engine = module
        .create_jit_execution_engine(OptimizationLevel::Default)
        .unwrap();

    let none_t = icontext.void_type();
    let i64_t = icontext.i64_type();
    let ptr_type = icontext.ptr_type(AddressSpace::default());

    let expr_fn_ty = i64_t.fn_type(
        &[
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
            ptr_type.into(),
        ], false
    );

    let u32_t = icontext.i32_type();
    let u8_t = icontext.i8_type();

    let load_v_cback = i64_t.fn_type(
        &[
            ptr_type.into(), u32_t.into()
        ], false
    );

    let store_a_cback = none_t.fn_type(
        &[
            ptr_type.into(), u32_t.into(), i64_t.into()
        ], false
    );

    let set_ac_cback = none_t.fn_type(
        &[
            ptr_type.into(),
            u8_t.into(),
        ], false
    );

    let call_cback = i64_t.fn_type(
        &[
            ptr_type.into(), u32_t.into()
        ], false
    );

    // pub(crate) type LoadVarCallback = fn(*mut IntEvalContextState, u32) -> i64;
    // pub(crate) type StoreArgCallback = fn(*mut IntEvalContextState, u32, i64);
    // pub(crate) type SetArgcCallback = fn(*mut IntEvalContextState, u8);
    // pub(crate) type CallFnCallback = fn(*mut IntEvalContextState, u32) -> i64;
    // pub(crate) type RawJitIntExpr = fn(*mut IntEvalContextState, LoadVarCallback, StoreArgCallback, SetArgcCallback, CallFnCallback) -> i64;


    let func = module.add_function("expr_1", expr_fn_ty, None);
    let builder = icontext.create_builder();
    let block = icontext.append_basic_block(func, "entry");
    builder.position_at_end(block);

    let
}