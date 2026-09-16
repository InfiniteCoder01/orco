/// Format an infix intrinsic, with operator precedence in mind.
pub fn format(
    ctx: &super::Context,
    f: &mut std::fmt::Formatter,
    mut idx: usize,
    precedence: u8,
) -> Result<usize, std::fmt::Error> {
    let instr = ctx.body.instructions[idx];
    let orco::ir::Instr::Intrinsic(mut intr) = instr else {
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
        Intrinsic::Neg => (3, "-"),

        Intrinsic::And => match ctx.body.value_ty(idx) {
            orco::Type::Bool => (14, "&&"),
            _ => (11, "&"),
        },
        Intrinsic::Or => match ctx.body.value_ty(idx) {
            orco::Type::Bool => (15, "||"),
            _ => (13, "|"),
        },
        Intrinsic::Xor => (12, "^"),
        Intrinsic::Not => {
            if ctx.body.instructions[idx] == orco::ir::Instr::Intrinsic(Intrinsic::Eq) {
                intr = Intrinsic::Eq;
                idx += 1;
                (10, "!=")
            } else {
                match ctx.body.value_ty(idx) {
                    orco::Type::Bool => (3, "!"),
                    _ => (3, "~"),
                }
            }
        }

        Intrinsic::Shl => (7, "<<"),
        Intrinsic::Shr => (7, ">>"),

        Intrinsic::Eq => (10, "=="),
        Intrinsic::Lt => (9, "<"),
        Intrinsic::Le => (9, "<="),
        Intrinsic::Gt => (9, ">"),
        Intrinsic::Ge => (9, ">="),

        Intrinsic::AggregateLiteral(_) => todo!(),
    };

    // Precedence check
    if precedence < op_precedence {
        write!(f, "(")?;
    }

    if intr.arg_count() == 1 {
        write!(f, "{operator}")?;
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
