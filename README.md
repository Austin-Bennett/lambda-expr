```
 \\\\\\
    \\\\
     \\\\
     /\\\\
    / /\\\\
   / /  \\\\     ||      /\      /\    /\   ||=\\ ||=\\     /\  -E
  / /    \\\\    ||     //\\    //\\  //\\  ||_// ||  \\   //\\  -X
 / /      \\\\   ||    //==\\  //  \\//  \\ || \\ ||  //  //==\\  -P
/_/        \\\\  \\== //    \\ ||        || ||=// ||=//  //    \\  -R
======================================================================
version 0.1.0-beta
```
**Description**

`lambda-expr` is a crate for evaluating string expressions 
named after lambda calculus, a branch of mathematics


**Quick example**
```rust
//function to use in a expression
pub fn abs(args: &[i64]) -> i64 {
    args[0].abs()
}

//create a context to manage variables and functions, aswell
//as the inkwell JIT context
let eval_ctx = IntEvalContextBuilder::new()
    .add_variable("a".to_string(), 4)
    .add_function("abs".to_string(), abs)
    .build();

//expression: should evaluate to 34
let expr_str = "(3 + 4a) * 2 - abs(a)";


//compile the expression to a native function
//runs about 2x faster than the bytecode implementation
//great if the expression is evaluated many many times
let jit_expr: JitIntExpression = eval_ctx.jit_compile(expr_str).unwrap();

//compile the expression to a bytecode instruction set
//runs slower than the JIT function, but compiles in
//a fraction of the time
//much better if the expression is only evaluated once 
//and then discarded
let bc_expr: BcIntExpression = eval_ctx.bytecode_compile(expr_str).unwrap();

//.eval() evaluates the expression
println!("{}", jit_expr.eval());
println!("{}", bc_expr.eval());
```

# Changelog

**Version:** 0.1.0-beta

**Beta release:** 
*limited in scope, some features may be bugged or not fully implemented*

**Changes:**

- Added `IntEvalContextBuilder` *Builder for IntEvalContext*
- Added `IntEvalContext` *Context for Integer expressions*
- Added `JitIntExpression` *Jit-compiled expression*
- Added `BcIntExpression` *Bytecode interpreted expression*

# Dependencies
```toml
anyhow = "1.0.102"
lazy_static = "1.5.0"
placenew = "3.0.0"
strum = { version = "0.28.0", features = ["derive"] }
inkwell = { version = "0.9.0", features = ["llvm22-1"] }
ouroboros = "0.18.5"
```





