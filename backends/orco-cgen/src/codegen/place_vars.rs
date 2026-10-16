use orco::ir::{Instr, VariableId};
use std::collections::HashMap;

/// Get a map from index to variable IDs that should be placed in front of this index.
pub fn compute_placement(body: &orco::ir::Body) -> HashMap<usize, Vec<VariableId>> {
    let mut var2place = vec![usize::MAX; body.variables.len()];

    let mut idx = 0;
    while idx < body.instructions.len() {
        let next_idx = body.skip_instr(idx);
        let start_idx = idx;
        while idx < next_idx {
            if let Instr::Var(var) = body.instructions[idx] {
                if var2place[var.0 as usize] == usize::MAX {
                    var2place[var.0 as usize] = start_idx;
                }
            }
            idx += 1;
        }
    }

    let mut var_places: HashMap<_, Vec<_>> = HashMap::new();
    for (var, idx) in var2place.iter().enumerate() {
        var_places
            .entry(*idx)
            .or_default()
            .push(orco::ir::VariableId(var as _));
    }

    var_places
}

/// Place an instruction accompanied by variables.
pub fn instr(
    f: &mut std::fmt::Formatter<'_>,
    ctx: &super::Context<'_>,
    mut idx: usize,
) -> Result<usize, std::fmt::Error> {
    let assignment = match (
        ctx.body.instructions[idx],
        ctx.body.instructions.get(idx + 1),
    ) {
        (Instr::Assign, Some(Instr::Var(id))) => Some(*id),
        _ => None,
    };

    let mut fuse_assignment = false;
    for id in ctx.var_places.get(&idx).into_iter().flatten().copied() {
        let var = ctx.body.var(id);
        if var.arg || crate::ty::is_unit(&var.ty) {
            continue;
        }

        if assignment == Some(id) {
            fuse_assignment = true;
            continue;
        }

        writeln!(
            f,
            "  {};",
            crate::ty::FmtType {
                ty: &var.ty,
                constant: false,
                name: Some(&ctx.var_names[id.0 as usize])
            }
        )?;
    }

    if fuse_assignment {
        let id = assignment.unwrap();
        let var = ctx.body.var(id);
        write!(
            f,
            "  {} = ",
            crate::ty::FmtType {
                ty: &var.ty,
                constant: false,
                name: Some(&ctx.var_names[id.0 as usize])
            }
        )?;

        idx += 2;
    } else {
        write!(f, "  ")?;
    }

    idx = ctx.instr(f, idx, 14)?;
    writeln!(f, ";")?;

    Ok(idx)
}
