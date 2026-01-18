use crate::TyCtxt;
use orco::codegen as oc;
use orco::codegen::ACFCodegen as _;

mod operand;

struct CodegenCtx<'tcx, CG> {
    tcx: TyCtxt<'tcx>,
    codegen: CG,
    body: &'tcx rustc_middle::mir::Body<'tcx>,
    variables: Vec<oc::Variable>,
}

impl<'tcx, CG: oc::BodyCodegen> CodegenCtx<'tcx, CG> {
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
            Rvalue::Use(op) => {
                let op = self.op(op);
                self.codegen.assign(op, self.place(*place))
            }
            Rvalue::Aggregate(kind, fields) => {
                use rustc_middle::mir::AggregateKind as AK;
                match kind.as_ref() {
                    AK::Array(..) => todo!(),
                    AK::Tuple => {
                        for (idx, op) in fields.iter_enumerated() {
                            let orco_op = self.op(op);
                            self.codegen.assign(
                                orco_op,
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
                            let op = self.op(op);
                            self.codegen.assign(
                                op,
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
                let params = vec![self.op(&operands.0), self.op(&operands.1)];
                self.codegen.call(
                    // TODO: Operators themselves
                    oc::Operand::Place(oc::Place::Global(format!("{op:?}").into())),
                    params,
                    self.place(*place),
                );
            }
            _ => eprintln!("TODO: {stmt:?}"), // TODO
        }
    }

    fn codegen_block(&mut self, block: rustc_middle::mir::BasicBlock) {
        self.codegen.acf().label(oc::Label(block.index()));
        let block = &self.body[block];
        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }
        use rustc_middle::mir::TerminatorKind;
        match &block.terminator().kind {
            TerminatorKind::Goto { target } => self.codegen.acf().jump(oc::Label(target.index())),
            TerminatorKind::SwitchInt { discr, targets } => {
                let lhs = self.op(discr);
                for (value, target) in targets.iter() {
                    self.codegen
                        .acf()
                        .cjump(lhs.clone(), value, true, oc::Label(target.index()));
                }
                self.codegen
                    .acf()
                    .jump(oc::Label(targets.otherwise().index()));
            }
            TerminatorKind::UnwindResume => todo!(),
            TerminatorKind::UnwindTerminate(..) => todo!(),
            TerminatorKind::Return => self
                .codegen
                .return_(oc::Operand::Place(oc::Place::Variable(self.variables[0]))),
            TerminatorKind::Unreachable => todo!(),
            TerminatorKind::Drop { target, .. } => {
                self.codegen.acf().jump(oc::Label(target.index()));
                // TODO
            }
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                ..
            } => {
                let func = self.op(func);
                let args = args.iter().map(|arg| self.op(&arg.node)).collect();
                self.codegen.call(func, args, self.place(*destination));
                if let Some(target) = target {
                    self.codegen.acf().jump(oc::Label(target.index()));
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
pub fn body<'a>(
    tcx: TyCtxt<'a>,
    codegen: impl oc::BodyCodegen,
    body: &'a rustc_middle::mir::Body<'a>,
) {
    let mut ctx = CodegenCtx {
        tcx,
        codegen,
        body,
        variables: Vec::with_capacity(body.local_decls.len()),
    };

    // TODO: debug variable names
    for (idx, local) in body.local_decls.iter_enumerated() {
        let idx = idx.index();
        let var = if idx > 0 && idx - 1 < body.arg_count {
            // An argument
            ctx.codegen.arg_var(idx - 1)
        } else {
            let ty = crate::types::convert(tcx, local.ty);
            ctx.codegen.declare_var(ty)
        };
        ctx.variables.push(var);
    }

    for block in body.basic_blocks.reverse_postorder() {
        ctx.codegen_block(*block);
    }
}

/// Codegen all the functions using the backend provided.
/// See [`crate::declare`]
pub fn codegen(
    tcx: TyCtxt<'_>,
    backend: &impl oc::CodegenBackend,
    items: &rustc_middle::hir::ModuleItems,
) {
    let backend = rustc_data_structures::sync::IntoDynSyncSend(backend);
    items
        .par_items(|item| {
            let item = tcx.hir_item(item);
            let key = item.owner_id.def_id;
            let name = crate::names::convert_path(tcx, key.into()).into();

            use rustc_hir::ItemKind as IK;
            // TODO: All of theese
            match item.kind {
                IK::Static(..) => (),
                IK::Const(..) => (),
                IK::Fn { .. } => {
                    body(tcx, backend.function(name), tcx.optimized_mir(key));
                }
                IK::GlobalAsm { .. } => (),
                IK::Trait(..) => (),
                IK::Impl(..) => (),
                _ => (),
            }
            Ok(())
        })
        .unwrap();

    items.par_impl_items(|_| todo!()).unwrap();
}
