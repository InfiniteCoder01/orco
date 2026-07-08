use super::{Codegen, ir, oc};

impl oc::AcfCodegen for &mut Codegen<'_> {
    fn alloc_label(&mut self) -> oc::Label {
        self.body.labels.push(0);
        oc::Label(self.body.labels.len() - 1)
    }

    fn label(&mut self, label: oc::Label) {
        self.body.labels[label.0] = self.body.statements.len();
    }

    fn jump(&mut self, label: oc::Label) {
        self.body
            .statements
            .push(ir::Statement::Acf(ir::AcfStatement::Jump(label)));
    }

    fn cjump(&mut self, condition: oc::Value, label: oc::Label) {
        let condition = self.use_value(condition);
        self.body
            .statements
            .push(ir::Statement::Acf(ir::AcfStatement::Cjump(
                condition, label,
            )));
    }
}

impl oc::BcfCodegen for &mut Codegen<'_> {
    fn if_(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition);
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::If(condition)));
    }

    fn else_(&mut self) {
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::Else));
    }

    fn end(&mut self) {
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::End));
    }

    fn loop_(&mut self) {
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::Loop));
    }

    fn break_(&mut self) {
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::Break));
    }

    fn continue_(&mut self) {
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::Continue));
    }

    fn cbreak(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition);
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::Cbreak(condition)));
    }

    fn ccontinue(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition);
        self.body
            .statements
            .push(ir::Statement::Bcf(ir::BcfStatement::Ccontinue(condition)));
    }
}
