use crate::{ob, ra};
use ra::def::hir::{Expr, ExprId};

pub struct CodegenCtx<'a, 'b, CG: ob::Codegen<'a>> {
    pub codegen: &'b mut CG,
    pub store: &'a ra::def::expr_store::ExpressionStore,
    pub inference: triomphe::Arc<ra::ty::InferenceResult>,
}

pub enum Value<Value> {
    Value(Value),
    Unit,
    Never,
}

impl<'a, 'b, CG: ob::Codegen<'a>> CodegenCtx<'a, 'b, CG> {
    pub fn expr_ty(&self, id: ExprId) -> &ra::ty::Ty {
        self.inference
            .type_of_expr_or_pat(ra::def::hir::ExprOrPatId::ExprId(id))
            .expect("type inference did not provide the type")
    }

    pub fn build_expr(&mut self, id: ExprId) -> Value<CG::Value> {
        match &self.store[id] {
            Expr::Missing => panic!("missing expression"),
            Expr::Path(..) => todo!(),
            Expr::If { .. } => todo!(),
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
                            if matches!(value, Value::Never) {
                                return Value::Never;
                            }
                        }
                        Statement::Item(..) => (),
                    }
                }
                if let Some(tail) = tail {
                    self.build_expr(*tail)
                } else {
                    Value::Unit
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
            Expr::Literal(lit) => Value::Value(self.build_literal(id, &lit)),
            Expr::Underscore => todo!(),
            Expr::OffsetOf(..) => todo!(),
            Expr::InlineAsm(..) => todo!(),
        }
    }

    pub fn build_literal(&mut self, id: ExprId, lit: &ra::def::hir::Literal) -> CG::Value {
        use ra::def::hir::Literal;
        let ty = || super::types::convert(self.codegen.pts(), self.expr_ty(id));

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
