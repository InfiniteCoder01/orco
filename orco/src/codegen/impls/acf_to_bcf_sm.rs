use crate::codegen as cg;
use cg::{BcfCodegen as _, Intrinsics as _};

/// Convert arbitrary control flow to BCF using one big state machine
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AcfToBcfStateMachine {
    state_var: Option<cg::Variable>,
    next_label_id: usize,
}

impl AcfToBcfStateMachine {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the implementation of [`cg::AcfCodegen`],
    /// referencing `codegen`. You must also supply a getter
    /// for the [AcfToBcfStateMachine] instance
    pub fn acf<'a, CG: cg::BodyCodegen>(
        &self,
        codegen: &mut CG,
        getter: fn(&mut CG) -> &mut AcfToBcfStateMachine,
    ) -> impl cg::AcfCodegen {
        Wrapper { codegen, getter }
    }
}

struct Wrapper<'a, CG: cg::BodyCodegen> {
    codegen: &'a mut CG,
    getter: fn(&mut CG) -> &mut AcfToBcfStateMachine,
}

impl<CG: cg::BodyCodegen> Wrapper<'_, CG> {
    fn state(&mut self) -> &mut AcfToBcfStateMachine {
        if (self.getter)(self.codegen).state_var.is_none() {
            let state_var = self.codegen.declare_var(
                crate::Type::Unsigned(crate::types::IntegerSize::Size),
                Some("state"),
            );
            self.codegen.bcf().loop_();

            let state = (self.getter)(self.codegen);
            state.state_var = Some(state_var);
            return state;
        }
        (self.getter)(self.codegen)
    }
}

impl<CG: cg::BodyCodegen> cg::AcfCodegen for Wrapper<'_, CG> {
    fn alloc_label(&mut self) -> cg::Label {
        todo!()
    }

    fn label(&mut self, label: cg::Label) {
        todo!()
    }

    fn jump(&mut self, label: cg::Label) {
        todo!()
    }

    fn cjump(&mut self, condition: cg::Value, label: cg::Label) {
        todo!()
    }
}
