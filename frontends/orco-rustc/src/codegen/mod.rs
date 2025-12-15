use crate::TyCtxt;
use orco::Backend;
use orco::codegen as oc;

mod operand;

struct CodegenCtx<'tcx, CG> {
    tcx: TyCtxt<'tcx>,
    codegen: CG,
    body: &'tcx rustc_middle::mir::Body<'tcx>,
    variables: Vec<oc::Variable>,
}

impl<'tcx, 'a, CG: oc::BodyCodegen> CodegenCtx<'tcx, CG> {
    fn codegen_statement(&mut self, stmt: &rustc_middle::mir::Statement<'tcx>) {
        use rustc_middle::mir::StatementKind;
        let (place, rvalue) = match &stmt.kind {
            StatementKind::Assign(assign) => assign.as_ref(),
            StatementKind::SetDiscriminant { .. } => todo!(),
            StatementKind::Intrinsic(..) => todo!(),
            stmt => {
                // TODO: Some of them are worth implementing
                eprintln!("TODO: {stmt:?}");
                return;
            }
        };

        use rustc_middle::mir::Rvalue;
        match rvalue {
            // Rvalue::Cast(_, op, _) => self.codegen.assign(self.op(op), self.var(*place)),
            Rvalue::Use(op) => self.codegen.assign(self.op(op), self.place(*place)),
            Rvalue::Aggregate(kind, fields) => {
                use rustc_middle::mir::AggregateKind as AK;
                match kind.as_ref() {
                    AK::Array(..) => todo!(),
                    AK::Tuple => {
                        for (idx, op) in fields.iter_enumerated() {
                            self.codegen.assign(
                                self.op(op),
                                self.place(place.project_deeper(
                                    &[rustc_middle::mir::PlaceElem::Field(
                                        idx,
                                        op.ty(&self.body.local_decls, self.tcx),
                                    )],
                                    self.tcx,
                                )),
                            );
                        }
                    }
                    AK::Adt(key, variant, ..) => {
                        let adt = self.tcx.adt_def(key);
                        let variant = &adt.variants()[*variant];
                        for (idx, op) in fields.iter_enumerated() {
                            let field = &variant.fields[idx];
                            self.codegen.assign(
                                self.op(op),
                                self.place(place.project_deeper(
                                    &[rustc_middle::mir::PlaceElem::Field(
                                        idx,
                                        self.tcx.type_of(field.did).skip_binder(), // TODO: Generics?!!!
                                    )],
                                    self.tcx,
                                )),
                            );
                        }
                    }
                    AK::Closure(..) => todo!(),
                    AK::Coroutine(..) => todo!(),
                    AK::CoroutineClosure(..) => todo!(),
                    AK::RawPtr(..) => todo!(),
                }
            }
            Rvalue::BinaryOp(op, operands) => {
                self.codegen.call(
                    // TODO: Operators themselves
                    oc::Operand::Place(oc::Place::Global(format!("{op:?}").into())),
                    vec![self.op(&operands.0), self.op(&operands.1)],
                    self.place(*place),
                );
            }
            _ => eprintln!("TODO: {stmt:?}"), // TODO
        }
    }

    fn codegen_block(&mut self, block: rustc_middle::mir::BasicBlock) {
        self.codegen.label(oc::Label(block.index()));
        let block = &self.body[block];
        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }
        use rustc_middle::mir::TerminatorKind;
        match &block.terminator().kind {
            TerminatorKind::Goto { target } => self.codegen.jump(oc::Label(target.index())),
            TerminatorKind::SwitchInt { discr, targets } => {
                let lhs = self.op(discr);
                for (value, target) in targets.iter() {
                    self.codegen
                        .cjump(lhs.clone(), value, true, oc::Label(target.index()));
                }
                self.codegen.jump(oc::Label(targets.otherwise().index()));
            }
            TerminatorKind::UnwindResume => todo!(),
            TerminatorKind::UnwindTerminate(..) => todo!(),
            TerminatorKind::Return => self
                .codegen
                .return_(oc::Operand::Place(oc::Place::Variable(self.variables[0]))),
            TerminatorKind::Unreachable => todo!(),
            TerminatorKind::Drop { target, .. } => {
                self.codegen.jump(oc::Label(target.index()));
                // TODO
            }
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                ..
            } => {
                self.codegen.call(
                    self.op(func),
                    args.iter().map(|arg| self.op(&arg.node)).collect(),
                    self.place(*destination),
                );
                if let Some(target) = target {
                    self.codegen.jump(oc::Label(target.index()));
                }
            }
            TerminatorKind::TailCall { .. } => todo!(),
            TerminatorKind::Assert { .. } => {
                // TODO
            }
            TerminatorKind::Yield { .. } => todo!(),
            TerminatorKind::CoroutineDrop => todo!(),
            TerminatorKind::FalseEdge { .. } => todo!(),
            TerminatorKind::FalseUnwind { .. } => todo!(),
            TerminatorKind::InlineAsm { .. } => todo!(),
        }
    }
}

/// Codegen a body
/// Note: Generates dirty code, not meant to be human-readable
pub fn body<'a, 'b>(
    tcx: TyCtxt<'b>,
    backend: &impl Backend,
    codegen: impl orco::BodyCodegen,
    body: &'b rustc_middle::mir::Body<'b>,
) {
    let mut ctx = CodegenCtx {
        tcx,
        codegen,
        body,
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
            let ty = crate::types::convert(tcx, backend, local.ty);
            ctx.codegen.declare_var(ty)
        };
        ctx.variables.push(var);
    }

    for block in body.basic_blocks.reverse_postorder() {
        ctx.codegen_block(*block);
    }
}
