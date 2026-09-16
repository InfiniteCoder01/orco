/// Format an infix intrinsic, with operator precedence in mind.
pub fn infix(
    ctx: &super::Context,
    f: &mut std::fmt::Formatter,
    mut idx: usize,
    precedence: u8,
) -> Result<usize, std::fmt::Error> {
    let instr = ctx.body.instructions[idx];
    let orco::ir::Instr::Intrinsic(intr) = instr else {
        panic!("Expected intrinsic, got {instr}");
    };
    idx += 1;

    // Match the intrinsic
    use orco::ir::Intrinsic;
    let (op_precedence, operator) = match intr {
        Intrinsic::Add => (6, "+"),
        Intrinsic::Sub => (6, "-"),
        Intrinsic::Mul => (5, "*"),
        Intrinsic::Div => (5, "/"),
        Intrinsic::Rem => (5, "%"),

        Intrinsic::And => (1, "&"),
        Intrinsic::Or => todo!(),
        Intrinsic::Xor => todo!(),
        Intrinsic::Not => todo!(),

        Intrinsic::Shl => todo!(),
        Intrinsic::Shr => todo!(),

        Intrinsic::Eq => (10, "=="),
        Intrinsic::Lt => (9, "<"),
        Intrinsic::Le => (9, "<="),
        Intrinsic::Gt => (9, ">"),
        Intrinsic::Ge => (9, ">="),

        Intrinsic::AggregateLiteral(_) => todo!(),
        intrinsic => panic!("not infix: {intrinsic}"),
    };

    // Precedence check
    if precedence < op_precedence {
        write!(f, "(")?;
    }

    // All values
    for i in 0..intr.arg_count() {
        if i > 0 {
            write!(f, " {operator} ")?;
        }

        idx = ctx.instr(f, idx, op_precedence - i.min(1) as u8)?;
    }

    // Precedence check
    if precedence < op_precedence {
        write!(f, ")")?;
    }
    Ok(idx)
}
