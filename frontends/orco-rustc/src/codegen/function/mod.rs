use crate::TyCtxt;
use orco::DefinitionBackend as Backend;
use orco::codegen as oc;

mod operand;

struct CodegenCtx<'tcx, CG> {
    tcx: TyCtxt<'tcx>,
    codegen: CG,
    variables: Vec<oc::Variable>,
}

impl<'tcx, 'a, CG: oc::Codegen<'a>> CodegenCtx<'tcx, CG> {
    fn codegen_statement(&mut self, stmt: &rustc_middle::mir::Statement) {
        use rustc_middle::mir::StatementKind;
        let (place, rvalue) = match &stmt.kind {
            StatementKind::Assign(assign) => assign.as_ref(),
            StatementKind::SetDiscriminant { .. } => todo!(),
            StatementKind::Intrinsic(..) => todo!(),
            stmt => {
                // TODO: Some of them are worth implementing
                self.codegen.comment(&format!("{stmt:?}"));
                return;
            }
        };

        use rustc_middle::mir::Rvalue;
        match rvalue {
            // Rvalue::Cast(_, op, _) => self.codegen.assign(self.op(op), self.var(*place)),
            Rvalue::Use(op) => self.codegen.assign(self.op(op), self.var(*place)),
            _ => self.codegen.comment(&format!("{stmt:?}")), // TODO
        }
    }

    fn codegen_block(&mut self, block: &rustc_middle::mir::BasicBlockData) {
        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }
        use rustc_middle::mir::TerminatorKind;
        match &block.terminator().kind {
            TerminatorKind::Goto { .. } => todo!(),
            TerminatorKind::SwitchInt { .. } => todo!(),
            TerminatorKind::UnwindResume => todo!(),
            TerminatorKind::UnwindTerminate(..) => todo!(),
            TerminatorKind::Return => self
                .codegen
                .return_(oc::Operand::Variable(self.variables[0])),
            TerminatorKind::Unreachable => todo!(),
            TerminatorKind::Drop { .. } => {
                // TODO
            }
            TerminatorKind::Call {
                func,
                args,
                destination,
                ..
            } => self.codegen.call(
                self.op(func),
                args.iter().map(|arg| self.op(&arg.node)).collect(),
                self.var(*destination),
            ),
            TerminatorKind::TailCall { .. } => todo!(),
            TerminatorKind::Assert { .. } => todo!(),
            TerminatorKind::Yield { .. } => todo!(),
            TerminatorKind::CoroutineDrop => todo!(),
            TerminatorKind::FalseEdge { .. } => todo!(),
            TerminatorKind::FalseUnwind { .. } => todo!(),
            TerminatorKind::InlineAsm { .. } => todo!(),
        }
    }
}

/// Define a function specified by key.
/// Uses MIR under the hood.
/// See also [`crate::declare::declare_function`]
pub fn define(tcx: TyCtxt, backend: &impl Backend, key: rustc_hir::def_id::DefId) {
    use oc::Codegen as _;

    let name = crate::declare::convert_path(tcx, key);
    let body = tcx.optimized_mir(key);
    let mut ctx = CodegenCtx {
        tcx,
        codegen: backend.define_function(name),
        variables: Vec::with_capacity(body.local_decls.len()),
    };

    rustc_middle::mir::pretty::MirWriter::new(tcx)
        .write_mir_fn(body, &mut std::io::stdout().lock())
        .unwrap();

    // TODO: debug variable names
    for (idx, local) in body.local_decls.iter_enumerated() {
        let idx = idx.index();
        let var = if idx > 0 && idx - 1 < body.arg_count {
            // An argument
            ctx.codegen.arg_var(idx - 1)
        } else {
            let ty = crate::declare::convert_type(tcx, backend, local.ty);
            ctx.codegen.declare_var(ty)
        };
        ctx.variables.push(var);
    }

    for block in body.basic_blocks.reverse_postorder() {
        ctx.codegen_block(&body[*block]);
    }
}
