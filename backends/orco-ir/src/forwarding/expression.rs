use super::{ir, oc};

impl<CG: oc::BodyCodegen> super::FwdCtx<'_, CG> {
    #[inline]
    pub fn var(&self, var: oc::Variable) -> oc::Variable {
        self.variable_map[var.0]
    }

    /// Convert [`ir::Place`] into [`oc::Place`],
    /// while generating code for inner expressions using
    /// [`Self::expr`]
    pub fn place(&mut self, place: &ir::Place) -> oc::Place {
        match place {
            ir::Place::Variable(variable) => self.var(*variable).into(),
            ir::Place::Global(symbol, generics) => oc::Place::Global(
                *symbol,
                generics
                    .iter()
                    .map(|ty| ty.copy_instantiate(&self.type_map))
                    .collect(),
            ),
            ir::Place::Deref(expr) => oc::Place::Deref(self.expr(expr)),
            ir::Place::Field(place, idx) => self.place(place).field(*idx),
        }
    }

    /// Codegen [`ir::Expression`] into another [`oc::BodyCodegen`]
    pub fn expr(&mut self, expr: &ir::Expression) -> oc::Value {
        match expr {
            ir::Expression::IConst(value, size) => self.cg.iconst(*value, *size),
            ir::Expression::UConst(value, size) => self.cg.uconst(*value, *size),
            ir::Expression::FConst(value, size) => self.cg.fconst(*value, *size),
            ir::Expression::BConst(value) => self.cg.bconst(*value),
            ir::Expression::Read(place) => {
                let place = self.place(place);
                self.cg.read(place)
            }
            ir::Expression::Reference(place, mutable) => {
                let place = self.place(place);
                self.cg.reference(place, *mutable)
            }
            ir::Expression::Call(func, args) => {
                let func = self.expr(func);
                let args = args.iter().map(|arg| self.expr(arg)).collect();
                self.cg
                    .call(func, args)
                    .unwrap_or_else(|| panic!("trying to use value from calling a void function"))
            }

            ir::Expression::Intrinsic(intrinsic) => {
                use crate::ir::Intrinsic as I;
                use oc::Intrinsics as IT;
                match intrinsic {
                    I::Add(a, b) => {
                        let a = self.expr(a);
                        let b = self.expr(b);
                        self.cg.intrinsics().add(a, b)
                    }
                    I::Mul(a, b) => {
                        let a = self.expr(a);
                        let b = self.expr(b);
                        self.cg.intrinsics().mul(a, b)
                    }
                    I::Eq(a, b) => {
                        let a = self.expr(a);
                        let b = self.expr(b);
                        self.cg.intrinsics().eq(a, b)
                    }
                    I::Not(a) => {
                        let a = self.expr(a);
                        self.cg.intrinsics().not(a)
                    }
                }
            }
        }
    }
}
