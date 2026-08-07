/// Id of a label (index into labels list).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LabelId(pub u32);

impl std::fmt::Display for LabelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl super::Body {
    /// Allocate a label to be placed in the source code later,
    /// returns the newly-allocated ID. ID value order guaranteed
    pub fn alloc_label(&mut self, name: Option<String>) -> LabelId {
        let id = LabelId(self.label_names.len() as _);
        self.label_names.push(name);
        id
    }

    /// Get a string used to identify the label in debug output
    pub fn label_debug_name(&self, id: LabelId) -> String {
        format!(
            "{}{id}",
            self.label_names
                .get(id.0 as usize)
                .unwrap_or_else(|| panic!("invalid label id {id}"))
                .as_deref()
                .unwrap_or("_")
        )
    }
}
