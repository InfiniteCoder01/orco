use super::{CodegenCtx, Instr};
use rustc_public::ty::{RigidTy, TyKind};

impl CodegenCtx<'_> {
    /// Codegen a place access into the body.
    pub(super) fn place(&mut self, place: &rustc_public::mir::Place) {
        for proj in place.projection.iter().rev() {
            use rustc_public::mir::ProjectionElem as PE;
            match proj {
                PE::Deref => todo!(),
                PE::Field(field, _) => self.instr(Instr::Field(*field as _)),
                PE::Index(_) => todo!(),
                PE::ConstantIndex { .. } => todo!(),
                PE::Subslice { .. } => todo!(),
                PE::Downcast(..) => todo!(),
                PE::OpaqueCast(..) => todo!(),
            }
        }

        self.instr(Instr::Var(self.variables[place.local]));
    }

    /// Codegen a constant value from [`rustc_public::ty::Allocation`].
    fn constant(&mut self, alloc: &rustc_public::ty::Allocation, ty: rustc_public::ty::Ty) {
        use orco::Type;
        match crate::ty::convert(ty) {
            Type::Integer(sz) => self.ir_body.int_literal(alloc.read_int().unwrap(), sz),
            Type::Unsigned(sz) => self.ir_body.uint_literal(alloc.read_uint().unwrap(), sz),
            Type::Float(_) => todo!("float const"),
            Type::Bool => self.instr(Instr::BConst(alloc.read_bool().unwrap())),
            Type::Char(_) => todo!("char const"),
            ty => panic!("complex (currently unsupported) const type {ty}"),
        }
    }

    /// Codegen a zero-sized constant.
    fn zero_sized(&mut self, ty: rustc_public::ty::Ty) {
        match ty.kind() {
            // TODO: We might need to do more
            TyKind::RigidTy(RigidTy::FnDef(func, generics)) => {
                let symbol = self.ir_body.use_symbol(
                    func.0.name().into(),
                    crate::ty::convert_generic_args(&generics),
                    self.module,
                );
                self.instr(Instr::Global(symbol));
            }
            _ => {
                let var = self
                    .ir_body
                    .declare_var(crate::ty::convert(ty), Some("zst".to_owned()));
                self.instr(Instr::Var(var));
            }
        }
    }

    /// Codegen [`rustc_public::mir::Operand`] into the body.
    pub(super) fn op(&mut self, op: &rustc_public::mir::Operand) {
        use rustc_public::mir::Operand;
        use rustc_public::ty::ConstantKind;
        match op {
            Operand::Copy(place) | Operand::Move(place) => {
                self.place(place);
            }
            Operand::Constant(value) => match value.const_.kind() {
                ConstantKind::Ty(..) => todo!(),
                ConstantKind::Allocated(allocation) => self.constant(allocation, value.ty()),
                ConstantKind::Unevaluated(uc) => {
                    panic!("unevaluated const encountered ({uc:?})")
                }
                ConstantKind::Param(..) => todo!(),
                ConstantKind::ZeroSized => self.zero_sized(value.ty()),
            },
            Operand::RuntimeChecks(..) => todo!(),
        }
    }
}
