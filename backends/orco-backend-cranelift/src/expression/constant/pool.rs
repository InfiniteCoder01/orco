use cranelift_codegen::ir::InstBuilder;
use cranelift_module::Module;

impl crate::Object<'_> {
    pub fn add_constant_to_pool(
        &mut self,
        builder: &mut cranelift_frontend::FunctionBuilder,
        value: &[u8],
    ) -> cranelift_codegen::ir::Value {
        let (id, pool) = self.constant_data.get_or_insert_with(|| {
            let id = self
                .object
                .declare_data("constants", cranelift_module::Linkage::Hidden, false, false)
                .unwrap();
            (id, Vec::new())
        });
        let offset = pool.len();
        pool.extend_from_slice(value);

        let pointer_type = self.object.target_config().pointer_type();
        let local_id = self.object.declare_data_in_func(*id, builder.func);
        let local_symbol = builder.ins().symbol_value(pointer_type, local_id);
        let offset = builder.ins().iconst(pointer_type, offset as i64);
        builder.ins().iadd(local_symbol, offset)
    }
}
