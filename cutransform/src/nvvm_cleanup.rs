use inkwell::module::Module;

pub fn replace_external_void_functions(module: &Module) -> Result<(), String> {
    for func in module.get_functions() {
        let func_name = func.get_name().to_str().unwrap_or("<Invalid UTF8>");

        let returns_void = func.get_type().get_return_type().is_none();
        if func.count_basic_blocks() == 0 && func.get_intrinsic_id() == 0 && returns_void {
            eprintln!(
                "Warning: replacing unresolved external function `{}` with noop",
                func_name
            );

            let ctx = module.get_context();
            let block = ctx.append_basic_block(func, "noop");
            let builder = ctx.create_builder();
            builder.position_at_end(block);
            builder.build_return(None);
        }
    }

    Ok(())
}
