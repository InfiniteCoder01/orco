use crate::{ob, ra};

use ra::def::DefWithBodyId;
use ra::def::hir::{Expr, ExprId};
use ra::hir::db::HirDatabase;
use triomphe::Arc;

mod value;
pub use value::AType;

/// Central code generation context
pub struct CodegenCtx<'a, 'b, CG, DB>
where
    CG: ob::Codegen<'a>,
    DB: HirDatabase,
{
    codegen: &'b mut CG,
    db: &'a DB,
    def: DefWithBodyId,
    body: Arc<ra::def::expr_store::Body>,
    inference: Arc<ra::ty::InferenceResult>,
    variables: std::collections::HashMap<ra::hir::Name, ob::Symbol>,
}

impl<'a, 'b, CG, DB> CodegenCtx<'a, 'b, CG, DB>
where
    CG: ob::Codegen<'a>,
    DB: HirDatabase,
{
    pub fn new(codegen: &'b mut CG, db: &'a DB, def: DefWithBodyId) -> Self {
        let mut ctx = Self {
            codegen,
            db,
            def,
            body: db.body(def),
            inference: db.infer(def),
            variables: std::collections::HashMap::new(),
        };

        // Insert params
        if let Some(param) = ctx.body.self_param {
            ctx.variables
                .insert(ctx.body[param].name.clone(), ctx.codegen.param(0));
        }
        for (idx, pat) in ctx.body.clone().params.iter().enumerate() {
            let symbol = ctx
                .codegen
                .param(idx - ctx.body.self_param.is_some() as usize);
            ctx.generate_bindings(*pat, symbol);
        }
        ctx
    }

    // Getters
    pub fn codegen(&mut self) -> &mut CG {
        self.codegen
    }

    pub fn db(&self) -> &DB {
        self.db
    }

    pub fn def(&self) -> DefWithBodyId {
        self.def
    }

    pub fn body(&self) -> &ra::def::expr_store::Body {
        &self.body
    }

    // Functions
    pub fn expr_ty(&self, id: ExprId) -> &ra::ty::Ty {
        self.inference
            .type_of_expr_or_pat(ra::def::hir::ExprOrPatId::ExprId(id))
            .expect("type inference did not provide the type")
    }

    /// Generate bindings as if we were destructuring sybmol
    fn generate_bindings(&mut self, pat: ra::def::hir::PatId, symbol: ob::Symbol) {
        use ra::def::hir::Pat;
        match self.body[pat] {
            Pat::Bind { id, subpat } => {
                let binding = &self.body[id];
                // TODO: handle binding modes
                self.variables.insert(binding.name.clone(), symbol);
                if let Some(_pat) = subpat {
                    todo!()
                }
            }
            _ => todo!(),
        }
    }

    pub fn build_expr(&mut self, id: ExprId) -> AType<ob::Value> {
        match &self.body.clone()[id] {
            Expr::Missing => panic!("missing expression"),
            Expr::Path(path) => {
                use ra::def::resolver::HasResolver as _;
                let mut resolver = self.def.resolver(self.db);
                let g = resolver.update_to_inner_scope(self.db, self.def, id);
                let hygiene = self.body.expr_path_hygiene(id);
                let value_ns = resolver
                    .resolve_path_in_value_ns_fully(self.db, path, hygiene)
                    .unwrap_or_else(|| panic!("unresolved path '{path:?}'"));
                resolver.reset_to_guard(g);

                use ra::def::resolver::ValueNs;
                match value_ns {
                    ValueNs::ImplSelf(..) => todo!(),
                    ValueNs::LocalBinding(id) => {
                        let binding = self.body[id].clone();
                        let &symbol = self.variables.get(&binding.name).unwrap_or_else(|| {
                            panic!("binding '{}' not found", binding.name.as_str())
                        });
                        AType::Value(self.codegen.variable(symbol))
                    }
                    ValueNs::FunctionId(..) => todo!(),
                    ValueNs::ConstId(..) => todo!(),
                    ValueNs::StaticId(..) => todo!(),
                    ValueNs::StructId(..) => todo!(),
                    ValueNs::EnumVariantId(..) => todo!(),
                    ValueNs::GenericParam(..) => todo!(),
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let ty = self.expr_ty(id);
                let ty = match ty.kind(ra::ty::Interner) {
                    ra::ty::TyKind::Tuple(0, _) => AType::Unit,
                    ra::ty::TyKind::Never => AType::Never,
                    _ => AType::Value(super::types::convert(self.codegen.backend(), ty)),
                };

                let slot = ty.map(|ty| {
                    let slot = self.codegen.new_slot();
                    self.codegen.define_variable(slot, ty, true, None);
                    slot
                });

                let cond = self.build_expr(*condition);
                self.codegen.if_(cond.unwrap());

                if let AType::Value(value) = self.build_expr(*then_branch) {
                    self.codegen.assign_variable(slot.unwrap(), value);
                }

                if let Some(else_branch) = *else_branch {
                    self.codegen.else_();
                    if let AType::Value(value) = self.build_expr(else_branch) {
                        self.codegen.assign_variable(slot.unwrap(), value);
                    }
                }

                self.codegen.end();

                slot.map(|slot| self.codegen.variable(slot))
            }
            Expr::Let { .. } => todo!(),
            Expr::Block {
                statements, tail, ..
            } => {
                for stmt in statements {
                    use ra::def::hir::Statement;
                    match stmt {
                        Statement::Let { .. } => todo!(),
                        Statement::Expr { expr, .. } => {
                            let value = self.build_expr(*expr);
                            if matches!(value, AType::Never) {
                                return AType::Never;
                            }
                        }
                        Statement::Item(..) => (),
                    }
                }
                if let Some(tail) = tail {
                    self.build_expr(*tail)
                } else {
                    AType::Unit
                }
            }
            Expr::Async { .. } => todo!(),
            Expr::Const(..) => todo!(),
            Expr::Unsafe { .. } => todo!(),
            Expr::Loop { .. } => todo!(),
            Expr::Call { .. } => todo!(),
            Expr::MethodCall { .. } => todo!(),
            Expr::Match { .. } => todo!(),
            Expr::Continue { .. } => todo!(),
            Expr::Break { .. } => todo!(),
            Expr::Return { .. } => todo!(),
            Expr::Become { .. } => todo!(),
            Expr::Yield { .. } => todo!(),
            Expr::Yeet { .. } => todo!(),
            Expr::RecordLit { .. } => todo!(),
            Expr::Field { .. } => todo!(),
            Expr::Await { .. } => todo!(),
            Expr::Cast { .. } => todo!(),
            Expr::Ref { .. } => todo!(),
            Expr::Box { .. } => todo!(),
            Expr::UnaryOp { .. } => todo!(),
            Expr::BinaryOp { .. } => todo!(),
            Expr::Assignment { .. } => todo!(),
            Expr::Range { .. } => todo!(),
            Expr::Index { .. } => todo!(),
            Expr::Closure { .. } => todo!(),
            Expr::Tuple { .. } => todo!(),
            Expr::Array(..) => todo!(),
            Expr::Literal(lit) => AType::Value(self.build_literal(id, lit)),
            Expr::Underscore => todo!(),
            Expr::OffsetOf(..) => todo!(),
            Expr::InlineAsm(..) => todo!(),
        }
    }

    fn build_literal(&mut self, id: ExprId, lit: &ra::def::hir::Literal) -> ob::Value {
        use ra::def::hir::Literal;
        let ty = || super::types::convert(self.codegen.backend(), self.expr_ty(id));

        match lit {
            Literal::String(..) => todo!(),
            Literal::ByteString(..) => todo!(),
            Literal::CString(..) => todo!(),
            Literal::Char(..) => todo!(),
            Literal::Bool(..) => todo!(),
            Literal::Int(value, ..) => self.codegen.iconst(ty(), *value),
            Literal::Uint(value, ..) => self.codegen.uconst(ty(), *value),
            Literal::Float(..) => todo!(),
        }
    }
}
