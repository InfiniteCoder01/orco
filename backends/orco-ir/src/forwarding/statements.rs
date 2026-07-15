use super::{ir, oc};

impl<CG: oc::BodyCodegen> super::FwdCtx<'_, CG> {
    #[inline]
    pub fn label(&self, label: oc::Label) -> oc::Label {
        self.label_map[label.0]
    }

    /// Codegen [`ir::Statement`] into another [`oc::BodyCodegen`]
    pub fn stmt(&mut self, stmt: &ir::Statement) {
        match stmt {
            ir::Statement::Comment(comment) => self.cg.comment(comment),
            ir::Statement::Assign(place, expr) => {
                let place = self.place(place);
                let expr = self.expr(expr);
                self.cg.assign(place, expr)
            }
            ir::Statement::Call(func, args) => {
                let func = self.expr(func);
                let args = args.iter().map(|arg| self.expr(arg)).collect();
                if let Some(value) = self.cg.call(func, args) {
                    self.cg.mk_tmp(value);
                }
            }
            ir::Statement::Return(expr) => {
                let value = expr.as_ref().map(|expr| self.expr(expr));
                self.cg.return_(value)
            }

            ir::Statement::Acf(acf) => self.acf(acf),
            ir::Statement::Bcf(bcf) => self.bcf(bcf),
        }
    }

    /// Codegen [`ir::AcfStatement`] into another [`oc::BodyCodegen`]
    fn acf(&mut self, stmt: &ir::AcfStatement) {
        use oc::AcfCodegen as _;
        match stmt {
            ir::AcfStatement::Jump(label) => {
                let label = self.label(*label);
                self.cg.acf().jump(label)
            }
            ir::AcfStatement::Cjump(expr, label) => {
                let expr = self.expr(expr);
                let label = self.label(*label);
                self.cg.acf().cjump(expr, label)
            }
        }
    }

    /// Codegen this statement into another [`oc::BodyCodegen`],
    /// mapping all variables and labels (ACF)
    fn bcf(&mut self, stmt: &ir::BcfStatement) {
        use oc::BcfCodegen as _;
        match stmt {
            ir::BcfStatement::If(expr) => {
                let expr = self.expr(expr);
                self.cg.bcf().if_(expr)
            }
            ir::BcfStatement::Else => self.cg.bcf().else_(),
            ir::BcfStatement::End => self.cg.bcf().end(),
            ir::BcfStatement::Loop => self.cg.bcf().loop_(),
            ir::BcfStatement::Break => self.cg.bcf().break_(),
            ir::BcfStatement::Continue => self.cg.bcf().continue_(),
            ir::BcfStatement::Cbreak(expr) => {
                let expr = self.expr(expr);
                self.cg.bcf().cbreak(expr)
            }
            ir::BcfStatement::Ccontinue(expr) => {
                let expr = self.expr(expr);
                self.cg.bcf().ccontinue(expr)
            }
        }
    }
}
