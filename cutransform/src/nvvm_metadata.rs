use inkwell::{module::Module, values::BasicMetadataValueEnum};

pub fn add_metadata_to_kernel_functions(module: &Module) -> Result<(), String> {
    let ctx = module.get_context();

    for func in module.get_functions() {
        if let Ok(func_name) = func.get_name().to_str() {
            // Skip the function if it does not start with the word "kernel"
            if !func_name.starts_with("kernel") {
                continue;
            }

            // Make sure that the function has a return type of `void`
            if let Some(return_type) = func.get_type().get_return_type() {
                return Err(format!(
                    "Kernel function `{}` has return type {}, when it should be void!",
                    func_name,
                    return_type
                        .print_to_string()
                        .to_str()
                        .unwrap_or("<Invalid UTF8>")
                ));
            }

            let metadata = ctx.metadata_node(&[
                BasicMetadataValueEnum::PointerValue(func.as_global_value().as_pointer_value()),
                BasicMetadataValueEnum::MetadataValue(ctx.metadata_string("kernel")),
                BasicMetadataValueEnum::IntValue(ctx.i32_type().const_int(1, true)),
            ]);
            module.add_global_metadata("nvvm.annotations", &metadata)?;
        }
    }

    Ok(())
}
