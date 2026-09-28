use ir::{Instr, Intrinsic};
use orco::ir;
use rustc_public::CrateDef as _;
use rustc_public::mir::BasicBlockIdx;

mod operand;

struct CodegenCtx<'a> {
    module: &'a orco::Module,
    body: ir::Body,
    /// Variable mapping
    variables: Vec<ir::VariableId>,
    /// Basic block predecessor indices
    predecessors: Vec<Vec<BasicBlockIdx>>,
}

impl CodegenCtx<'_> {
    fn instr(&mut self, instr: impl Into<Instr>) {
        self.body.instructions.push(instr.into());
    }

    fn codegen_statement(&mut self, stmt: &rustc_public::mir::Statement) {
        use rustc_public::mir::StatementKind;
        let (place, rvalue) = match &stmt.kind {
            StatementKind::Assign(place, rvalue) => (place, rvalue),
            StatementKind::Intrinsic(..) => todo!(),
            _ => return,
        };

        use rustc_public::mir::Rvalue;
        match rvalue {
            Rvalue::AddressOf(..) => todo!(),
            Rvalue::Aggregate(kind, fields) => {
                use rustc_public::mir::AggregateKind as AK;
                match kind {
                    AK::Array(..) => todo!(),
                    AK::Tuple | AK::Adt(..) => {
                        for (idx, op) in fields.iter().enumerate() {
                            self.instr(Instr::Assign);
                            self.instr(Instr::Field(idx as _));
                            self.place(&place);
                            self.op(op);
                        }
                    }
                    AK::Closure(..) => todo!(),
                    AK::Coroutine(..) => todo!(),
                    AK::CoroutineClosure(..) => todo!(),
                    AK::RawPtr(..) => todo!(),
                }
            }
            Rvalue::BinaryOp(op, a, b) | Rvalue::CheckedBinaryOp(op, a, b) => {
                let checked = matches!(rvalue, Rvalue::CheckedBinaryOp(..));
                if checked {
                    self.instr(Instr::Assign);
                    self.instr(Instr::Field(1));
                    self.place(place);
                    self.instr(Instr::BConst(false)); // TODO: Actually check it...
                }

                self.instr(Instr::Assign);
                if checked {
                    self.instr(Instr::Field(0))
                }
                self.place(place);

                use rustc_public::mir::BinOp;
                let intrinsic = match op {
                    BinOp::Add | BinOp::AddUnchecked => Intrinsic::Add,
                    BinOp::Sub | BinOp::SubUnchecked => Intrinsic::Sub,
                    BinOp::Mul | BinOp::MulUnchecked => Intrinsic::Mul,
                    BinOp::Div => Intrinsic::Div,
                    BinOp::Rem => Intrinsic::Rem,
                    BinOp::BitXor => Intrinsic::Xor,
                    BinOp::BitAnd => Intrinsic::And,
                    BinOp::BitOr => Intrinsic::Not,
                    BinOp::Shl | BinOp::ShlUnchecked => Intrinsic::Shl,
                    BinOp::Shr | BinOp::ShrUnchecked => Intrinsic::Shr,
                    BinOp::Eq => Intrinsic::Eq,
                    BinOp::Lt => Intrinsic::Lt,
                    BinOp::Le => Intrinsic::Le,
                    BinOp::Ne => Intrinsic::Ne,
                    BinOp::Ge => Intrinsic::Ge,
                    BinOp::Gt => Intrinsic::Gt,
                    BinOp::Cmp => todo!("<=>"),
                    BinOp::Offset => todo!("ptr.offset"),
                };

                self.instr(Instr::Intrinsic(intrinsic));
                self.op(a);
                self.op(b);
            }
            Rvalue::Cast(..) => todo!(),
            Rvalue::CopyForDeref(..) => todo!(),
            Rvalue::Discriminant(..) => todo!(),
            Rvalue::Len(..) => todo!(),
            Rvalue::Ref(..) => todo!(),
            Rvalue::Repeat(..) => todo!(),
            Rvalue::ThreadLocalRef(..) => todo!(),
            Rvalue::UnaryOp(..) => todo!(),
            Rvalue::Use(op, _) => {
                self.instr(Instr::Assign);
                self.place(place);
                self.op(op);
            }
            Rvalue::Reborrow(..) => todo!(),
        }
    }

    /// Codegen a basic block, inserting a label to it.
    /// Index in the list helps skip generating unnecessary jumps.
    fn codegen_block(&mut self, block: &rustc_public::mir::BasicBlock, index: BasicBlockIdx) {
        if self.predecessors[index] != index.checked_sub(1).as_slice() {
            self.instr(Instr::AcfLabel(ir::LabelId(index as _)));
        }

        for stmt in &block.statements {
            self.codegen_statement(stmt);
        }

        let next_block = move |this: &mut Self, block: BasicBlockIdx| {
            if block != index + 1 {
                this.instr(Instr::AcfJump(ir::LabelId(block as _)));
            }
        };

        use rustc_public::mir::TerminatorKind;
        match &block.terminator.kind {
            TerminatorKind::Goto { target } => next_block(self, *target),
            TerminatorKind::SwitchInt { discr, targets } => {
                for (value, target) in targets.branches() {
                    self.instr(Instr::AcfCJump(ir::LabelId(target as _)));
                    self.instr(Intrinsic::Eq);

                    let idx = self.body.instructions.len();
                    self.op(discr);
                    match self.body.value_ty(self.module, idx) {
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
            TerminatorKind::Resume => (),
            TerminatorKind::Abort => todo!(),
            TerminatorKind::Return => {
                self.instr(Instr::Return);
                self.instr(Instr::Var(self.variables[0]));
            }
            TerminatorKind::Unreachable => todo!(),
            TerminatorKind::Drop { target, .. } => {
                // TODO
                next_block(self, *target);
            }
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                ..
            } => {
                self.instr(Instr::Assign);
                self.place(destination);
                self.instr(Instr::Call(args.len() as _));
                self.op(func);
                for arg in args {
                    self.op(&arg);
                }

                if let Some(target) = target {
                    next_block(self, *target);
                }
            }
            TerminatorKind::Assert { target, .. } => {
                // TODO
                next_block(self, *target);
            }
            TerminatorKind::InlineAsm { .. } => todo!(),
        }
    }
}

/// Codegen a body
/// Note: Generates dirty code, not meant to be human-readable
pub fn body(
    ir_body: ir::Body,
    rs_body: &rustc_public::mir::Body,
    module: &orco::Module,
) -> ir::Body {
    let mut ctx = CodegenCtx {
        module,
        body: ir_body,
        variables: Vec::with_capacity(rs_body.locals().len()),
        predecessors: vec![Vec::new(); rs_body.locals().len()],
    };

    // Fill in variables
    for (idx, local) in rs_body.local_decls() {
        let var = if (1..rs_body.arg_locals().len() + 1).contains(&idx) {
            // An argument
            ir::VariableId(idx as u32 - 1)
        } else {
            ctx.body.declare_var(crate::ty::convert(local.ty), None)
        };

        ctx.variables.push(var);
    }

    for info in &rs_body.var_debug_info {
        use rustc_public::mir::VarDebugInfoContents as VDIC;
        match &info.value {
            VDIC::Place(place) => {
                let var = ctx.body.var_mut(ctx.variables[place.local]);
                if !place.projection.is_empty() && var.name.is_some() {
                    continue;
                }
                var.name = Some(info.name.to_string());
            }
            VDIC::Const(..) => (),
        }
    }

    // Fill in the blocks
    for (idx, block) in rs_body.blocks.iter().enumerate() {
        ctx.body.alloc_label(Some("bb".to_owned()));
        for successor in block.terminator.successors() {
            ctx.predecessors[successor].push(idx);
        }
    }

    for (idx, block) in rs_body.blocks.iter().enumerate() {
        ctx.codegen_block(block, idx);
    }

    ctx.body
}

/// Codegen all the functions using the backend provided.
/// See [`crate::declare`]
pub fn codegen(crate_: rustc_public::Crate, module: &orco::Module) {
    for func in crate_.fn_defs() {
        let Some(rs_body) = func.body() else {
            continue;
        };

        let path: orco::Symbol = func.name().into();
        let functions = module.functions.pin();
        let mut function = functions
            .get(&path)
            .unwrap_or_else(|| panic!("undelcared function {path}"))
            .write()
            .unwrap();

        let ir_body = body(function.create_def(), &rs_body, module);
        function
            .body
            .replace(ir_body)
            .map(|_| panic!("trying to define function {path} twice"));
    }
}
