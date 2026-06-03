use crate::codegen as cg;
use cg::{BcfCodegen as _, Intrinsics as _};

/// Convert arbitrary control flow to BCF using one big state machine.
/// To use this, add it as a field to your codegen and return
/// ```
/// AcfToBcfStateMachine::acf(self, |this| &mut this.acf_to_bcf)
/// ```
/// in your [`cg::BodyCodegen::acf`] implementation. Also, add this
/// before you wrap up your code generation:
/// ```
/// AcfToBcfStateMachine::finish(self, |this| &mut this.acf_to_bcf);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AcfToBcfStateMachine {
    state_var: Option<cg::Variable>,
    next_label_id: usize,
}

impl AcfToBcfStateMachine {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the implementation of [`cg::AcfCodegen`],
    /// referencing `codegen`. You must also supply a getter
    /// for the [`AcfToBcfStateMachine`] instance
    pub fn acf<CG: cg::BodyCodegen>(
        codegen: &mut CG,
        getter: fn(&mut CG) -> &mut AcfToBcfStateMachine,
    ) -> impl cg::AcfCodegen {
        Wrapper { codegen, getter }
    }

    /// Call this at the very end.
    pub fn finish<CG: cg::BodyCodegen>(
        codegen: &mut CG,
        getter: fn(&mut CG) -> &mut AcfToBcfStateMachine,
    ) {
        if getter(codegen).state_var.is_some() {
            codegen.bcf().end();
            codegen.bcf().end();
        }
    }
}

struct Wrapper<'a, CG: cg::BodyCodegen> {
    codegen: &'a mut CG,
    getter: fn(&mut CG) -> &mut AcfToBcfStateMachine,
}

impl<CG: cg::BodyCodegen> Wrapper<'_, CG> {
    fn state(&mut self) -> &mut AcfToBcfStateMachine {
        (self.getter)(self.codegen)
    }

    fn change_state(&mut self, state_var: cg::Variable, label: cg::Label) {
        let value = self
            .codegen
            .uconst(label.0 as _, crate::types::IntegerSize::Size);
        self.codegen.assign(state_var.into(), value);
        if self.state().state_var.is_some() {
            self.codegen.bcf().continue_();
        }
    }

    fn begin_state(&mut self, state_var: cg::Variable, label: cg::Label) {
        let var = self.codegen.read(state_var.into());
        let value = self
            .codegen
            .uconst(label.0 as _, crate::types::IntegerSize::Size);
        let condition = self.codegen.intrinsics().eq(var, value);
        if self.state().state_var.is_some() {
            self.codegen.bcf().end();
        }
        self.codegen.bcf().if_(condition);
    }

    fn ensure_sm(
        &mut self,
        initial_state: impl FnOnce(&mut Self) -> cg::Label,
    ) -> (cg::Variable, bool) {
        if let Some(state_var) = self.state().state_var {
            return (state_var, false);
        }

        let state_var = self.codegen.declare_var(
            crate::Type::Unsigned(crate::types::IntegerSize::Size),
            Some("state"),
        );

        let label0 = initial_state(self);
        self.change_state(state_var, label0);
        self.codegen.bcf().loop_();
        self.begin_state(state_var, label0);

        self.state().state_var = Some(state_var);
        (state_var, true)
    }
}

impl<CG: cg::BodyCodegen> cg::AcfCodegen for Wrapper<'_, CG> {
    fn alloc_label(&mut self) -> cg::Label {
        let state = self.state();
        state.next_label_id += 1;
        cg::Label(state.next_label_id - 1)
    }

    fn label(&mut self, label: cg::Label) {
        let (state_var, initial) = self.ensure_sm(|_| label);
        if !initial {
            self.begin_state(state_var, label);
        }
    }

    fn jump(&mut self, label: cg::Label) {
        let (state_var, _) = self.ensure_sm(Self::alloc_label);
        self.change_state(state_var, label);
    }

    fn cjump(&mut self, condition: cg::Value, label: cg::Label) {
        let (state_var, _) = self.ensure_sm(Self::alloc_label);
        self.codegen.bcf().if_(condition);
        self.change_state(state_var, label);
        self.codegen.bcf().end();
    }
}
