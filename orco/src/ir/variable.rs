/// Id of a variable (index into variables list).
/// It is known that all function arguments have sequential IDs, starting from index 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableId(pub u32);

impl std::fmt::Display for VariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Info about one variable in a body.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableInfo {
    /// Type of this variable.
    pub ty: crate::Type,
    /// Wether this variable comes from function arguments.
    pub arg: bool,
    /// Debug name.
    pub name: Option<String>,
}

impl super::Body {
    /// Declare a variable with a set type and optional debug name,
    /// which can later be set using [`Self::var_mut`]
    /// Returns the newly-allocated ID to be used with [`Self::var`].
    /// ID value order guaranteed, see note on [`VariableId`].
    pub fn declare_var(&mut self, ty: crate::Type, name: Option<String>) -> VariableId {
        let id = VariableId(self.variables.len() as _);
        self.variables.push(VariableInfo {
            ty,
            arg: false,
            name,
        });
        id
    }

    /// Get variable info by ID.
    pub fn var(&self, id: VariableId) -> &VariableInfo {
        self.variables
            .get(id.0 as usize)
            .unwrap_or_else(|| panic!("invalid variable id {id}"))
    }

    /// Mutable version of [`Self::var`].
    pub fn var_mut(&mut self, id: VariableId) -> &mut VariableInfo {
        self.variables
            .get_mut(id.0 as usize)
            .unwrap_or_else(|| panic!("invalid variable id {id}"))
    }

    /// Get a string used to identify the variable in debug output.
    pub fn var_debug_name(&self, id: VariableId) -> String {
        format!("{}{id}", self.var(id).name.as_deref().unwrap_or("_"))
    }
}
