



pub enum IBcOp {
    Push, //pushes the value in RET to the stack
    PushI(i64), //push a integer to the stack
    Pop, //pops a value off the stack into AUX
    MoveRet(i64),
    MoveAux(i64),
    Add, //add aux to ret
    Sub,
    Mul,
    Div,
    Pow,
    Call(fn(*const i64) -> i64),
}