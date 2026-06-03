use crate::codegen as cg;
use cg::{AcfCodegen as _, Intrinsics as _};

/// BCF block types
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BlockType {
    If { end: cg::Label },
    Else { end: cg::Label },
    Loop { start: cg::Label, end: cg::Label },
}

/// Convert block-based control flow to ACF.
/// To use this, add it as a field to your codegen and return
/// ```
/// BcfToAcf::bcf(self, |this| &mut this.acf_to_bcf)
/// ```
/// in your [`cg::BodyCodegen::bcf`] implementation
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BcfToAcf {
    stack: Vec<BlockType>,
}

impl BcfToAcf {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the implementation of [`cg::BcfCodegen`],
    /// referencing `codegen`. You must also supply a getter
    /// for the [BcfToAcf] instance
    pub fn bcf<'a, CG: cg::BodyCodegen>(
        codegen: &'a mut CG,
        getter: fn(&mut CG) -> &mut BcfToAcf,
    ) -> impl cg::BcfCodegen + 'a {
        Wrapper { codegen, getter }
    }

    fn last_loop(&self) -> Option<(cg::Label, cg::Label)> {
        for block in self.stack.iter().rev() {
            let BlockType::Loop { start, end } = block else {
                continue;
            };

            return Some((*start, *end));
        }

        None
    }
}

struct Wrapper<'a, CG: cg::BodyCodegen> {
    codegen: &'a mut CG,
    getter: fn(&mut CG) -> &mut BcfToAcf,
}

impl<CG: cg::BodyCodegen> Wrapper<'_, CG> {
    fn state(&mut self) -> &mut BcfToAcf {
        (self.getter)(self.codegen)
    }
}

impl<CG: cg::BodyCodegen> cg::BcfCodegen for Wrapper<'_, CG> {
    fn if_(&mut self, condition: cg::Value) {
        let end = self.codegen.acf().alloc_label();
        let uncondition = self.codegen.intrinsics().not(condition);
        self.codegen.acf().cjump(uncondition, end);
        self.state().stack.push(BlockType::If { end });
    }

    fn else_(&mut self) {
        match self.state().stack.pop() {
            Some(BlockType::If { end }) => {
                let end2 = self.codegen.acf().alloc_label();
                self.codegen.acf().jump(end2);
                self.codegen.acf().label(end);
                self.state().stack.push(BlockType::Else { end: end2 });
            }
            block => {
                panic!("expected last block to be `if` while generating else, but it was {block:?}")
            }
        }
    }

    fn end(&mut self) {
        let Some(block) = self.state().stack.pop() else {
            panic!("calling end() on an empty stack");
        };
        match block {
            BlockType::If { end } => self.codegen.acf().label(end),
            BlockType::Else { end } => self.codegen.acf().label(end),
            BlockType::Loop { start, end } => {
                self.codegen.acf().jump(start);
                self.codegen.acf().label(end)
            }
        }
    }

    fn loop_(&mut self) {
        let start = self.codegen.acf().alloc_label();
        let end = self.codegen.acf().alloc_label();
        self.codegen.acf().label(start);
        self.state().stack.push(BlockType::Loop { start, end });
    }

    fn break_(&mut self) {
        let Some((_, end)) = self.state().last_loop() else {
            panic!("can't break() here, no loop blocks are open")
        };

        self.codegen.acf().jump(end);
    }

    fn continue_(&mut self) {
        let Some((start, _)) = self.state().last_loop() else {
            panic!("can't continue() here, no loop blocks are open")
        };

        self.codegen.acf().jump(start);
    }

    fn cbreak(&mut self, condition: cg::Value) {
        let Some((_, end)) = self.state().last_loop() else {
            panic!("can't cbreak() here, no loop blocks are open")
        };

        self.codegen.acf().cjump(condition, end);
    }

    fn ccontinue(&mut self, condition: cg::Value) {
        let Some((start, _)) = self.state().last_loop() else {
            panic!("can't ccontinue() here, no loop blocks are open")
        };

        self.codegen.acf().cjump(condition, start);
    }
}
