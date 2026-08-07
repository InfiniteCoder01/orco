use crate::TyCtxt;
use ir::{AcfInstr, Instr};
use orco::ir;
use std::collections::HashMap;

mod operand;

struct CodegenCtx<'tcx, 'a> {
    ctx: super::Context<'tcx, 'a>,
    ir_body: ir::Body,
    rs_body: &'a rustc_middle::mir::Body<'tcx>,
    variables: HashMap<rustc_middle::mir::Local, ir::VariableId>,
}

impl<'tcx, 'a> std::ops::Deref for CodegenCtx<'tcx, 'a> {
    type Target = super::Context<'tcx, 'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'tcx> CodegenCtx<'tcx, '_> {
    fn instr(&mut self, instr: impl Into<Instr>) {
        self.ir_body.instructions.push(instr.into());
    }

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
        let is_unit = place.ty(self.rs_body, self.tcx).ty.is_unit();

        use rustc_middle::mir::Rvalue;
        match rvalue {
            Rvalue::Use(op, _) => {
                if is_unit {
                    return;
                }

                self.instr(Instr::Assign);
                self.place(*place);
                self.op(op);
            }
            Rvalue::Aggregate(kind, fields) => {
                use rustc_middle::mir::AggregateKind as AK;
                match kind.as_ref() {
                    AK::Array(..) => todo!(),
                    AK::Tuple => {
                        for (idx, op) in fields.iter_enumerated() {
                            let ty = op.ty(&self.rs_body.local_decls, self.tcx);
                            if ty.is_unit() {
                                continue;
                            }

                            self.instr(Instr::Assign);
                            self.place(place.project_deeper(
                                &[rustc_middle::mir::PlaceElem::Field(idx, ty)],
                                self.tcx,
                            ));
                            self.op(op);
                        }
                    }
                    AK::Adt(key, variant, ..) => {
                        let adt = self.tcx.adt_def(*key);
                        let variant = &adt.variants()[*variant];
                        for (idx, op) in fields.iter_enumerated() {
                            let field = &variant.fields[idx];
                            let ty = self
                                .tcx
                                .type_of(field.did)
                                .instantiate_identity()
                                .skip_norm_wip();
                            if ty.is_unit() {
                                continue;
                            }

                            let place = place.project_deeper(
                                &[rustc_middle::mir::PlaceElem::Field(idx, ty)],
                                self.tcx,
                            );

                            self.instr(Instr::Assign);
                            self.place(place);
                            self.op(op);
                        }
                    }
                    AK::Closure(..) => todo!(),
                    AK::Coroutine(..) => todo!(),
                    AK::CoroutineClosure(..) => todo!(),
                    AK::RawPtr(..) => todo!(),
                }
            }
            Rvalue::BinaryOp(op, operands) => {
                // let params: Vec<_> = self
                //     .op(&operands.0)
                //     .into_iter()
                //     .chain(self.op(&operands.1))
                // .collect();

                // let ty = operands.0.ty(self.rs_body, self.tcx).to_string();
                // let value = crate::intrinsics().inline_call(
                //     &mut self.codegen,
                //     format!("__{op:?}#{ty}").into(),
                //     params,
                // );
                // if let (Some(place), Some(value)) = (self.place(*place), value) {
                //     self.codegen.assign(place, value);
                // }
            }
            _ => println!("TODO: {stmt:?}"), // TODO
        }
    }

    /// Codegen a basic block, inserting a label to it.
    /// Previous and next blocks are needed for optimization of jumps.
    fn codegen_block(
        &mut self,
        block: rustc_middle::mir::BasicBlock,
        prev: Option<rustc_middle::mir::BasicBlock>,
        next: Option<rustc_middle::mir::BasicBlock>,
    ) {
        let predecessors = self.rs_body.basic_blocks.predecessors();
        type Pred<'a> = &'a [rustc_middle::mir::BasicBlock];
        if &*predecessors[block] != prev.as_ref().map_or::<Pred, _>(&[], core::slice::from_ref) {
            self.instr(Instr::Acf(AcfInstr::Label(ir::LabelId(block.as_u32()))));
        }

        let block = &self.rs_body[block];
        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }

        let next_block = move |this: &mut Self, block| {
            if next != Some(block) {
                this.instr(AcfInstr::Jump(ir::LabelId(block.as_u32())));
            }
        };

        use rustc_middle::mir::TerminatorKind;
        match &block.terminator().kind {
            TerminatorKind::Goto { target } => next_block(self, *target),
            TerminatorKind::SwitchInt { discr, targets } => {
                for (value, target) in targets.iter() {
                    self.instr(AcfInstr::CJump(ir::LabelId(target.as_u32())));
                    // TODO!!!
                    // self.instr(Intrinsic::Eq);

                    let idx = self.ir_body.instructions.len();
                    self.op(discr);
                    match self.ir_body.value_ty(idx) {
                        orco::Type::Integer(is) => self.instr(Instr::IConst(value as _, is)),
                        orco::Type::Unsigned(is) => self.instr(Instr::UConst(value as _, is)),
                        orco::Type::Bool => {
                            assert!(
                                [0, 1].contains(&value),
                                "invalid bool branch in SwitchInt: {value} (expected 0 or 1)"
                            );
                            self.instr(Instr::BConst(value != 0))
                        }
                        orco::Type::Symbol(name, _) => {
                            todo!("symbol discriminant type in SwitchInt ({name})")
                        }
                        ty => panic!("invalid discriminant type in SwitchInt: {ty}"),
                    }
                }

                next_block(self, targets.otherwise())
            }
            TerminatorKind::UnwindResume => (),
            TerminatorKind::UnwindTerminate(..) => todo!(),
            TerminatorKind::Return => {
                let value = self
                    .variables
                    .get(&rustc_middle::mir::RETURN_PLACE)
                    .copied();
                if next.is_none() && value.is_none() {
                    return; // TODO: Idk if it's useful or not
                }
                self.instr(Instr::Return(value.is_some()));
                if let Some(value) = value {
                    self.instr(Instr::Var(value));
                }
            }
            TerminatorKind::Unreachable => todo!(),
            TerminatorKind::Drop { target, .. } => {
                self.instr(AcfInstr::Jump(ir::LabelId(target.as_u32())));
                // TODO
            }
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                ..
            } => {
                // let func = self.op(func).expect("trying to call a unit value");
                // let args = args.iter().filter_map(|arg| self.op(&arg.node)).collect();
                // let retval = self.codegen.call(func, args);
                // if let Some(place) = self.place(*destination) {
                //     self.codegen.assign(
                //         place,
                //         retval.expect("can't use the return value of a unit function"),
                //     );
                // }
                if let Some(target) = target {
                    next_block(self, *target);
                }
            }
            TerminatorKind::TailCall { func, args, .. } => {
                // let func = self.op(func).expect("trying to call a unit value");
                // let args = args.iter().filter_map(|arg| self.op(&arg.node)).collect();
                // let retval = self.codegen.call(func, args);
                // self.codegen.return_(retval);
            }
            TerminatorKind::Assert { target, .. } => {
                // TODO
                next_block(self, *target);
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
pub fn body<'tcx>(
    ctx: super::Context<'tcx, '_>,
    ir_body: ir::Body,
    rs_body: &rustc_middle::mir::Body<'tcx>,
) -> ir::Body {
    let mut ctx = CodegenCtx {
        ctx,
        ir_body,
        rs_body,
        variables: HashMap::new(),
    };

    for (idx, local) in rs_body.local_decls.iter_enumerated() {
        let var = if (1..rs_body.arg_count + 1).contains(&idx.index()) {
            // An argument
            Some(ir::VariableId(idx.index() as u32 - 1))
        } else {
            ctx.convert_ty(local.ty)
                .map(|ty| ctx.ir_body.declare_var(ty, None))
        };

        if let Some(var) = var {
            ctx.variables.insert(idx, var);
        }
    }

    for info in &rs_body.var_debug_info {
        use rustc_middle::mir::VarDebugInfoContents as VDIC;
        match info.value {
            VDIC::Place(place) => {
                let var = ctx.ir_body.var_mut(ctx.variables[&place.local]);
                if !place.projection.is_empty() && var.name.is_some() {
                    continue;
                }
                var.name = Some(info.name.to_string());
            }
            VDIC::Const(..) => (),
        }
    }

    for _ in rs_body.basic_blocks.indices() {
        ctx.ir_body.alloc_label(Some("bb".to_owned()));
    }

    let blocks = rs_body.basic_blocks.reverse_postorder();
    let mut prev = None;
    for (idx, &block) in blocks.iter().enumerate() {
        let next = blocks.get(idx + 1).copied();
        ctx.codegen_block(block, prev, next);
        prev = Some(block);
    }

    ctx.ir_body
}

/// Codegen a single function by key, inserting it's body into the module
pub fn cg_function(ctx: super::Context, key: rustc_hir::def_id::DefId) {
    let functions = ctx.module.functions.pin();
    let path = ctx.convert_path(key);
    let function = functions
        .get(&path)
        .unwrap_or_else(|| panic!("trying to define an undeclared function {path}"));
    let ir_body = body(ctx, function.create_def(), ctx.tcx.optimized_mir(key));
    function
        .body
        .set(ir_body)
        .unwrap_or_else(|_| panic!("trying to define function {path} twice"));
}

/// Codegen all the functions using the backend provided.
/// See [`crate::declare`]
pub fn codegen(tcx: TyCtxt, module: &orco::Module, items: &rustc_middle::hir::ModuleItems) {
    let module = rustc_data_structures::sync::IntoDynSyncSend(module);
    items
        .par_items(|item| {
            let item = tcx.hir_item(item);
            let ctx = super::Context {
                tcx,
                module: *module,
            };
            let key = item.owner_id.def_id;

            use rustc_hir::ItemKind as IK;
            match item.kind {
                IK::Static(..) => (),
                IK::Const(..) => (),
                IK::Fn { .. } => cg_function(ctx, key.to_def_id()),
                IK::GlobalAsm { .. } => todo!("global_asm!"),
                IK::Impl(impl_) if let Some(trait_) = impl_.of_trait => {
                    let Some(_trait_key) = trait_.trait_ref.trait_def_id() else {
                        panic!("[bug?] trait impl of a non-trait?!");
                    };

                    // // TODO: Generics
                    // let map = tcx.impl_item_implementor_ids(key);
                    // for item in tcx.associated_items(trait_key).in_definition_order() {
                    //     let (impl_key, is_default_impl) = map
                    //         .get(&item.def_id)
                    //         .map_or((item.def_id, true), |key| (*key, false));
                    //     let mut name = crate::names::convert_path(tcx, item.def_id);
                    //     let trait_name = name.as_str().into();

                    //     let self_ty = crate::types::convert(
                    //         tcx,
                    //         backend,
                    //         tcx.type_of(key).instantiate_identity().skip_norm_wip(),
                    //         crate::types::GenericMap::default(),
                    //     );
                    //     if let Some(ty) = &self_ty {
                    //         name.push('_');
                    //         name.push_str(&ty.hashable_name());
                    //     }

                    //     let trait_generic_args = self_ty.into_iter().collect::<Vec<_>>();
                    //     backend.invoke_macro(trait_name, &trait_generic_args);
                    //     let map = if is_default_impl {
                    //         crate::types::GenericMap(1, &trait_generic_args)
                    //     } else {
                    //         crate::types::GenericMap::default()
                    //     };

                    //     body(
                    //         tcx,
                    //         backend,
                    //         backend.cg_function(name.into()),
                    //         tcx.optimized_mir(impl_key),
                    //         map,
                    //     );
                    // }
                }
                IK::Impl(impl_) => {
                    for item in impl_.items {
                        cg_function(ctx, item.owner_id.to_def_id());
                    }
                }
                _ => (),
            };
            Ok(())
        })
        .unwrap();
}
