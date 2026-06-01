use crate::TyCtxt;
use orco::codegen as oc;
use orco::codegen::AcfCodegen as _;
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
        // self.codegen.comment(&format!("{stmt:#?}"));

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
            Rvalue::Use(op, _) => {
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

                let ty = operands.0.ty(self.body, self.tcx).to_string();
                let value = crate::intrinsics().inline_call(
                    &mut self.codegen,
                    format!("__{op:?}#{ty}").into(),
                    params,
                );
                if let (Some(place), Some(value)) = (self.place(*place), value) {
                    self.codegen.assign(place, value);
                }
            }
            _ => self.codegen.comment("TODO: {stmt:?}"), // TODO
        }
    }

    fn codegen_block(&mut self, block: rustc_middle::mir::BasicBlock) {
        self.codegen.acf().label(self.labels[&block]);
        let block = &self.body[block];

        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }

        // self.codegen.comment(&format!("{:#?}", block.terminator()));
        use rustc_middle::mir::TerminatorKind;
        match &block.terminator().kind {
            TerminatorKind::Goto { target } => self.codegen.acf().jump(self.labels[target]),
            TerminatorKind::SwitchInt { discr, targets } => {
                use oc::Intrinsics as _;
                for (value, target) in targets.iter() {
                    let discr = self.op(discr).expect("SwitchInt on unit discriminant");
                    let value = match self.codegen.type_of(discr.0) {
                        orco::Type::Integer(is) => self.codegen.iconst(value as _, is),
                        orco::Type::Unsigned(is) => self.codegen.uconst(value as _, is),
                        orco::Type::Bool => {
                            assert!(
                                [0, 1].contains(&value),
                                "invalid bool branch in SwitchInt: {value} (expected 0 or 1)"
                            );
                            self.codegen.bconst(value != 0)
                        }
                        orco::Type::Symbol(name) => {
                            todo!("symbol discriminant type in SwitchInt ({name})")
                        }
                        ty => panic!("invalid discriminant type in SwitchInt: {ty}"),
                    };
                    let condition = self.codegen.intrinsics().eq(discr, value);
                    self.codegen.acf().cjump(condition, self.labels[&target]);
                }
                self.codegen.acf().jump(self.labels[&targets.otherwise()]);
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
                func,
                args,
                destination,
                target,
                ..
            } => {
                let func = self.op(func).expect("trying to call a unit value");
                let args = args.iter().flat_map(|arg| self.op(&arg.node)).collect();
                let retval = self.codegen.call(func, args);
                if let Some(place) = self.place(*destination) {
                    self.codegen.assign(
                        place,
                        retval.expect("can't use the return value of a unit function"),
                    );
                }
                if let Some(target) = target {
                    self.codegen.acf().jump(oc::Label(target.index()));
                }
            }
            TerminatorKind::TailCall { func, args, .. } => {
                let func = self.op(func).expect("trying to call a unit value");
                let args = args.iter().flat_map(|arg| self.op(&arg.node)).collect();
                let retval = self.codegen.call(func, args);
                self.codegen.return_(retval);
            }
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

    let mut local_names = HashMap::new();
    for info in &body.var_debug_info {
        use rustc_middle::mir::VarDebugInfoContents as VDIC;
        match info.value {
            VDIC::Place(place) => {
                if !place.projection.is_empty() && local_names.contains_key(&place.local) {
                    continue;
                }
                local_names.insert(place.local, info.name);
            }
            VDIC::Const(..) => (),
        }
    }

    for (idx, local) in body.local_decls.iter_enumerated() {
        let var = if (1..body.arg_count + 1).contains(&idx.index()) {
            // An argument
            Some(oc::Variable(idx.index() - 1))
        } else if !local.ty.is_unit() {
            let ty = crate::types::convert(tcx, local.ty);
            ty.map(|ty| {
                ctx.codegen
                    .declare_var(ty, local_names.get(&idx).map(|name| name.as_str()))
            })
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
