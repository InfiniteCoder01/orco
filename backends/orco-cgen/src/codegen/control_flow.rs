use super::{Codegen, oc};
use std::fmt::Write as _;

impl oc::AcfCodegen for &mut Codegen<'_, '_> {
    fn alloc_label(&mut self) -> oc::Label {
        self.next_label_id += 1;
        oc::Label(self.next_label_id - 1)
    }

    fn label(&mut self, label: oc::Label) {
        writeln!(&mut self.body, "label{}:", label.0).unwrap();
    }

    fn jump(&mut self, label: oc::Label) {
        self.line(format_args!("goto label{};", label.0));
    }

    fn cjump(&mut self, condition: oc::Value, label: oc::Label) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) goto label{};", label.0));
    }
}

impl oc::BcfCodegen for &mut Codegen<'_, '_> {
    fn if_(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) {{"));
        self.indent += 1;
    }

    fn else_(&mut self) {
        self.indent -= 1;
        self.line(format_args!("}} else {{"));
        self.indent += 1;
    }

    fn end(&mut self) {
        self.indent -= 1;
        self.line(format_args!("}}"));
    }

    fn loop_(&mut self) {
        self.line(format_args!("while (true) {{"));
        self.indent += 1;
    }

    fn break_(&mut self) {
        self.line(format_args!("break;"));
    }

    fn continue_(&mut self) {
        self.line(format_args!("continue;"));
    }

    // This is very unnecessary, but I think it looks cleaner :)
    fn cbreak(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) break;"));
    }

    fn ccontinue(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) continue;"));
    }
}
