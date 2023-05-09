use inkwell::{context::Context, module::Module, values::BasicMetadataValueEnum};
use std::path::Path;

fn main() -> Result<(), String> {
    let file_path_str = std::env::args().nth(1).unwrap_or("kernel.bc".into());
    let file_path = Path::new(&file_path_str);

    let ctx = Context::create();
    let module = Module::parse_bitcode_from_path(file_path, &ctx).map_err(|e| e.to_string())?;

    // Add metadata to all kernel functions
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

    replace_stub_fn_with_intrinsic(&module, "threadIdxX", "llvm.nvvm.read.ptx.sreg.tid.x")?;
    replace_stub_fn_with_intrinsic(&module, "threadIdxY", "llvm.nvvm.read.ptx.sreg.tid.y")?;
    replace_stub_fn_with_intrinsic(&module, "threadIdxZ", "llvm.nvvm.read.ptx.sreg.tid.z")?;

    module.write_bitcode_to_path(file_path);

    Ok(())
}

fn replace_stub_fn_with_intrinsic(
    module: &Module,
    stub_fn_name: &str,
    instrinsic_name: &str,
) -> Result<(), String> {
    if let Some(func) = module.get_function(stub_fn_name) {
        let intrinsic_func = module.add_function(instrinsic_name, func.get_type(), None);
        func.replace_all_uses_with(intrinsic_func);
    }

    Ok(())
}
