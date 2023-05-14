use inkwell::{module::Module, values::BasicMetadataValueEnum};

pub fn add_metadata_to_kernel_functions(module: &Module) -> Result<(), String> {
    let ctx = module.get_context();

    for func in module.get_functions() {
        // Continue if the function does not start with "kernel"
        if let Ok(false) = func
            .get_name()
            .to_str()
            .map(|name| name.starts_with("kernel"))
        {
            continue;
        }

        let metadata = ctx.metadata_node(&[
            BasicMetadataValueEnum::PointerValue(func.as_global_value().as_pointer_value()),
            BasicMetadataValueEnum::MetadataValue(ctx.metadata_string("kernel")),
            BasicMetadataValueEnum::IntValue(ctx.i32_type().const_int(1, true)),
        ]);

        module.add_global_metadata("nvvm.annotations", &metadata)?;
    }

    Ok(())
}
