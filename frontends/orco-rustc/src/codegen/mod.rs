use crate::TyCtxt;
use ir::Instruction as Instr;
use orco::ir;
use std::collections::HashMap;

mod operand;

struct CodegenCtx<'tcx, 'a> {
    ctx: super::Context<'tcx, 'a>,
    ir_body: ir::Body,
    rs_body: &'a rustc_middle::mir::Body<'tcx>,
    variables: HashMap<rustc_middle::mir::Local, ir::VariableId>,
    // labels: HashMap<rustc_middle::mir::BasicBlock, oc::Label>,
}

impl<'tcx, 'a> std::ops::Deref for CodegenCtx<'tcx, 'a> {
    type Target = super::Context<'tcx, 'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'tcx> CodegenCtx<'tcx, '_> {
    fn instr(&mut self, instr: Instr) {
        self.ir_body.instructions.push(instr);
    }

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

    fn codegen_block(&mut self, block: rustc_middle::mir::BasicBlock) {
        // self.codegen.acf().label(self.labels[&block]);
        let block = &self.rs_body[block];

        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }

        // self.codegen.comment(&format!("{:#?}", block.terminator()));
        use rustc_middle::mir::TerminatorKind;
        match &block.terminator().kind {
            TerminatorKind::Goto { target } => {
                //self.codegen.acf().jump(self.labels[target]),
            }
            TerminatorKind::SwitchInt { discr, targets } => {
                //         use oc::Intrinsics as _;
                //         for (value, target) in targets.iter() {
                //             let discr = self.op(discr).expect("SwitchInt on unit discriminant");
                //             let value = match self.codegen.type_of(discr.0) {
                //                 orco::Type::Integer(is) => self.codegen.iconst(value as _, is),
                //                 orco::Type::Unsigned(is) => self.codegen.uconst(value as _, is),
                //                 orco::Type::Bool => {
                //                     assert!(
                //                         [0, 1].contains(&value),
                //                         "invalid bool branch in SwitchInt: {value} (expected 0 or 1)"
                //                     );
                //                     self.codegen.bconst(value != 0)
                //                 }
                //                 orco::Type::Symbol(name) => {
                //                     todo!("symbol discriminant type in SwitchInt ({name})")
                //                 }
                //                 ty => panic!("invalid discriminant type in SwitchInt: {ty}"),
                //             };
                //             let condition = self.codegen.intrinsics().eq(discr, value);
                //             self.codegen.acf().cjump(condition, self.labels[&target]);
                //         }
                //         self.codegen.acf().jump(self.labels[&targets.otherwise()]);
            }
            TerminatorKind::UnwindResume => (),
            TerminatorKind::UnwindTerminate(..) => todo!(),
            TerminatorKind::Return => {
                let value = self
                    .variables
                    .get(&rustc_middle::mir::RETURN_PLACE)
                    .copied();
                self.instr(Instr::Return(value.is_some()));
                if let Some(value) = value {
                    self.instr(Instr::Var(value));
                }
            }
            TerminatorKind::Unreachable => todo!(),
            TerminatorKind::Drop { target, .. } => {
                // self.codegen.acf().jump(self.labels[target]);
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
                // if let Some(target) = target {
                //     self.codegen.acf().jump(oc::Label(target.index()));
                // }
            }
            TerminatorKind::TailCall { func, args, .. } => {
                // let func = self.op(func).expect("trying to call a unit value");
                // let args = args.iter().filter_map(|arg| self.op(&arg.node)).collect();
                // let retval = self.codegen.call(func, args);
                // self.codegen.return_(retval);
            }
            TerminatorKind::Assert { target, .. } => {
                // self.codegen.acf().jump(self.labels[target]);
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

    let mut local_names = HashMap::new();
    for info in &rs_body.var_debug_info {
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

    for (idx, local) in rs_body.local_decls.iter_enumerated() {
        let var = if (1..rs_body.arg_count + 1).contains(&idx.index()) {
            // An argument
            Some(ir::VariableId(idx.index() as u32 - 1))
        } else {
            ctx.convert_ty(local.ty).map(|ty| {
                let id = ctx.ir_body.declare_var(ty);
                ctx.ir_body.var_mut(id).name =
                    local_names.get(&idx).map(rustc_span::Symbol::to_string);
                id
            })
        };

        if let Some(var) = var {
            ctx.variables.insert(idx, var);
        }
    }

    // for idx in rs_body.basic_blocks.indices() {
    //     ctx.labels.insert(idx, ctx.codegen.acf().alloc_label());
    // }

    for block in rs_body.basic_blocks.reverse_postorder() {
        ctx.codegen_block(*block);
    }

    ctx.ir_body
}

pub fn cg_function(ctx: super::Context, key: rustc_hir::def_id::DefId) {
    let functions = ctx.module.functions.pin();
    let path = ctx.convert_path(key);
    let function = functions
        .get(&path)
        .unwrap_or_else(|| panic!("trying to define an undeclared function {path}"));
    let ir_body = body(ctx, function.create_def(), ctx.tcx.optimized_mir(key));
    function.body.set(ir_body);
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
