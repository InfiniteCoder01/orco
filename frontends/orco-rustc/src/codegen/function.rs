use crate::TyCtxt;
use orco::DefinitionBackend as Backend;
use orco::codegen as oc;

struct CodegenCtx<CG> {
    codegen: CG,
    variables: Vec<oc::Variable>,
}

impl<'a, CG: oc::Codegen<'a>> CodegenCtx<CG> {
    fn var(&self, place: rustc_middle::mir::Place) -> oc::Variable {
        self.variables[place.local.index()] // TODO: projection
    }

    fn op(&self, op: &rustc_middle::mir::Operand) -> oc::Operand {
        use rustc_const_eval::interpret::Scalar;
        use rustc_middle::mir::{Const, ConstValue, Operand};
        match op {
            Operand::Copy(place) => oc::Operand::Variable(self.var(*place)),
            Operand::Move(place) => oc::Operand::Variable(self.var(*place)),
            Operand::Constant(value) => match value.const_ {
                Const::Ty(..) => todo!(),
                Const::Unevaluated(uc, ..) => {
                    panic!("unevaluated const encountered ({uc:?})")
                }
                Const::Val(value, ..) => match value {
                    ConstValue::Scalar(scalar) => match scalar {
                        Scalar::Int(value) => oc::Operand::UConst(value.to_bits(value.size())),
                        Scalar::Ptr(..) => todo!(),
                    },
                    ConstValue::ZeroSized => todo!(),
                    ConstValue::Slice { .. } => todo!(),
                    ConstValue::Indirect { .. } => todo!(),
                },
            },
        }
    }

    fn codegen_statement(&mut self, stmt: &rustc_middle::mir::Statement) {
        use rustc_middle::mir::StatementKind;
        let (place, rvalue) = match &stmt.kind {
            StatementKind::Assign(assign) => assign.as_ref(),
            stmt => {
                self.codegen.comment(&format!("{stmt:?}"));
                return;
            }
        };

        use rustc_middle::mir::Rvalue;
        match rvalue {
            Rvalue::Cast(_, op, _) => self.codegen.cast(self.op(op), self.var(*place)),
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
            TerminatorKind::Drop { .. } => todo!(),
            TerminatorKind::Call { .. } => todo!(),
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
pub fn define(tcx: TyCtxt, backend: &mut impl Backend, key: rustc_hir::def_id::LocalDefId) {
    use oc::Codegen as _;

    let name = crate::declare::convert_path(tcx, key);
    let mut ctx = CodegenCtx {
        codegen: backend.define_function(name),
        variables: Vec::new(),
    };

    let body = tcx.optimized_mir(key);
    rustc_middle::mir::pretty::MirWriter::new(tcx)
        .write_mir_fn(body, &mut std::io::stdout().lock())
        .unwrap();

    // TODO: debug variable names
    for local in &body.local_decls {
        let ty = crate::declare::convert_type(backend, local.ty);
        ctx.variables.push(ctx.codegen.declare_var(&ty));
    }

    for block in body.basic_blocks.reverse_postorder() {
        ctx.codegen_block(&body[*block]);
    }
}
