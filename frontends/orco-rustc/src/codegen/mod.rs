use crate::TyCtxt;
use orco::codegen as oc;
use orco::codegen::ACFCodegen as _;
use std::collections::HashMap;

mod operand;

struct CodegenCtx<'tcx, CG> {
    tcx: TyCtxt<'tcx>,
    codegen: CG,
    body: &'tcx rustc_middle::mir::Body<'tcx>,
    variables: HashMap<rustc_middle::mir::Local, Option<oc::Variable>>,
    labels: HashMap<rustc_middle::mir::BasicBlock, oc::Label>,
}

impl<'tcx, CG: oc::BodyCodegen> CodegenCtx<'tcx, CG> {
    fn codegen_statement(&mut self, stmt: &rustc_middle::mir::Statement<'tcx>) {
        self.codegen.comment(&format!("{stmt:#?}"));

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
                if let (Some(place), Some(value)) = (self.place(*place), self.op(op)) {
                    self.codegen.assign(place, value);
                }
            }
            Rvalue::Aggregate(kind, fields) => {
                use rustc_middle::mir::AggregateKind as AK;
                match kind.as_ref() {
                    AK::Array(..) => todo!(),
                    AK::Tuple => {
                        for (idx, op) in fields.iter_enumerated() {
                            let place = place.project_deeper(
                                &[rustc_middle::mir::PlaceElem::Field(
                                    idx,
                                    op.ty(&self.body.local_decls, self.tcx),
                                )],
                                self.tcx,
                            );
                            if let (Some(place), Some(value)) = (self.place(place), self.op(op)) {
                                self.codegen.assign(place, value);
                            }
                        }
                    }
                    AK::Adt(key, variant, ..) => {
                        let adt = self.tcx.adt_def(*key);
                        let variant = &adt.variants()[*variant];
                        for (idx, op) in fields.iter_enumerated() {
                            let field = &variant.fields[idx];
                            let place = place.project_deeper(
                                &[rustc_middle::mir::PlaceElem::Field(
                                    idx,
                                    self.tcx.type_of(field.did).skip_binder(), // TODO: Generics?!!!
                                )],
                                self.tcx,
                            );
                            if let (Some(place), Some(value)) = (self.place(place), self.op(op)) {
                                self.codegen.assign(place, value);
                            }
                        }
                    }
                    AK::Closure(..) => todo!(),
                    AK::Coroutine(..) => todo!(),
                    AK::CoroutineClosure(..) => todo!(),
                    AK::RawPtr(..) => todo!(),
                }
            }
            Rvalue::BinaryOp(op, operands) => {
                let params: Vec<_> = self
                    .op(&operands.0)
                    .into_iter()
                    .chain(self.op(&operands.1))
                    .collect();
                let ty = self.codegen.type_of(params[0].0).hashable_name();
                let op = self
                    .codegen
                    .read(oc::Place::Global(format!("__{op:?}#{ty}").into()));
                let value = self.codegen.call(op, params);
                if let (Some(place), Some(value)) = (self.place(*place), value) {
                    self.codegen.assign(place, value);
                }
            }
            _ => eprintln!("TODO: {stmt:?}"), // TODO
        }
    }

    fn codegen_block(&mut self, block: rustc_middle::mir::BasicBlock) {
        self.codegen.acf().label(self.labels[&block]);
        let block = &self.body[block];

        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }

        use rustc_middle::mir::TerminatorKind;
        match &block.terminator().kind {
            TerminatorKind::Goto { target } => self.codegen.acf().jump(oc::Label(target.index())),
            TerminatorKind::SwitchInt { targets, .. } => {
                self.codegen
                    .acf()
                    .jump(oc::Label(targets.otherwise().index()));
            }
            TerminatorKind::UnwindResume => todo!(),
            TerminatorKind::UnwindTerminate(..) => todo!(),
            TerminatorKind::Return => {
                let value = self.variables[&rustc_middle::mir::RETURN_PLACE]
                    .map(|var| self.codegen.read(var.into()));
                self.codegen.return_(value)
            }
            TerminatorKind::Unreachable => todo!(),
            TerminatorKind::Drop { target, .. } => {
                self.codegen.acf().jump(oc::Label(target.index()));
                // TODO
            }
            TerminatorKind::Call {
                // func,
                // args,
                // destination,
                // target,
                ..
            } => {
                // TODO!!!
                // let func = self.op(func);
                // let args = args.iter().map(|arg| self.op(&arg.node)).collect();
                // self.codegen.call(func, args, self.place(*destination));
                // if let Some(target) = target {
                //     self.codegen.acf().jump(oc::Label(target.index()));
                // }
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
        variables: HashMap::with_capacity(body.local_decls.len()),
        labels: HashMap::with_capacity(body.basic_blocks.len()),
    };

    // TODO: debug variable names
    for (idx, local) in body.local_decls.iter_enumerated() {
        let var = if (1..body.arg_count + 1).contains(&idx.index()) {
            // An argument
            Some(oc::Variable(idx.index() - 1))
        } else if !local.ty.is_unit() {
            let ty = crate::types::convert(tcx, local.ty);
            ty.map(|ty| ctx.codegen.declare_var(ty))
        } else {
            None
        };
        ctx.variables.insert(idx, var);
    }

    for idx in body.basic_blocks.indices() {
        ctx.labels.insert(idx, ctx.codegen.acf().alloc_label());
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
                IK::Trait { .. } => (),
                IK::Impl(..) => (),
                _ => (),
            }
            Ok(())
        })
        .unwrap();

    items.par_impl_items(|_| todo!()).unwrap();
}
