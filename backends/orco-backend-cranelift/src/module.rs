use super::*;

impl Object {
    /// Declare all symbols in a module, prefixing all it's symbols by path
    pub fn declare_module(&mut self, module: &orco::ir::Module, path: &Path) {
        for symbol in module.symbols.values() {
            let symbol = symbol.try_read().unwrap();
            if let Some(value) = &symbol.evaluated {
                match symbol.value.get_type() {
                    orco::ir::Type::Function => {
                        let function = value.as_ref::<orco::ir::expression::Function>();
                        self.declare_function(
                            path.extend(symbol.name.clone()),
                            None,
                            cranelift_module::Linkage::Export,
                            &function.signature,
                        );
                    }
                    orco::ir::Type::ExternFunction => {
                        let function = value.as_ref::<orco::ir::expression::ExternFunction>();
                        self.declare_function(
                            path.extend(symbol.name.clone()),
                            Some(function.name.as_ref()),
                            cranelift_module::Linkage::Import,
                            &function.signature,
                        );
                    }
                    orco::ir::Type::Module => {
                        let module = value.as_ref::<orco::ir::Module>();
                        self.declare_module(module, &path.extend(symbol.name.clone()));
                    }
                    _ => (),
                }
            }
        }
    }

    /// Build a module, prefixing all it's symbols by path
    pub fn build_module(&mut self, module: &orco::ir::Module, path: &Path) {
        for symbol in module.symbols.values() {
            let symbol = symbol.try_read().unwrap();
            if let Some(value) = &symbol.evaluated {
                match symbol.value.get_type() {
                    orco::ir::Type::Function => {
                        let function = value.as_ref::<orco::ir::expression::Function>();
                        self.build_function(&path.extend(symbol.name.clone()), function);
                    }
                    orco::ir::Type::Module => {
                        let module = value.as_ref::<orco::ir::Module>();
                        self.build_module(module, &path.extend(symbol.name.clone()));
                    }
                    _ => (),
                }
            }
        }
    }
}
